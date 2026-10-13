use std::sync::{
    atomic::{AtomicBool, AtomicU16, AtomicU8, Ordering},
    Arc, LazyLock, Mutex,
};

use arc_swap::ArcSwap;
#[cfg(feature = "runtime_trace")]
use std::sync::atomic::AtomicU64;

mod packet;
use packet::PiaCustomNetPacket;

use ssbu_pia_interface::{
    self, ConnectedStation, ConnectionChangedEvent, StationConnectionManager,
};

use crate::{
    net::{
        is_in_real_game,
        latency_slider::{Latency, LATENCY_SLIDER_MANAGER},
    },
    render::profile::{RenderProfile, RenderProfileSettings, RENDER_PROFILE_MANAGER},
};

static CONNECTED_STATION_TABLE_SYNCED_INTERNAL: LazyLock<Mutex<Vec<StationNetInfo>>> =
    LazyLock::new(|| Mutex::new(Vec::with_capacity(8)));
static CONNECTED_STATION_TABLE_ATOMIC_VIEW: LazyLock<ArcSwap<Vec<StationNetInfo>>> =
    LazyLock::new(|| ArcSwap::from_pointee(Vec::with_capacity(8)));

#[derive(Debug)]
struct StationNetInfo {
    id: u64,
    is_valid_comms: AtomicBool,
    latency_bits: AtomicU8,
    render_profile_settings_bits: AtomicU16,
    #[cfg(feature = "runtime_trace")]
    last_trace: AtomicU64,
}

impl Clone for StationNetInfo {
    fn clone(&self) -> Self {
        StationNetInfo {
            id: self.id,
            is_valid_comms: AtomicBool::new(self.is_valid_comms.load(Ordering::SeqCst)),
            latency_bits: AtomicU8::new(self.latency_bits.load(Ordering::SeqCst)),
            render_profile_settings_bits: AtomicU16::new(
                self.render_profile_settings_bits.load(Ordering::SeqCst),
            ),
            #[cfg(feature = "runtime_trace")]
            last_trace: AtomicU64::new(self.last_trace.load(Ordering::SeqCst)),
        }
    }
}

#[cfg(feature = "runtime_trace")]
impl StationNetInfo {
    fn trace_metadata(&self, bytes: &[u8], packet: Option<PiaCustomNetPacket>) {
        let version = bytes.first().map(|v| *v as u32);
        let signature = version.unwrap_or(256) as u64
            | ((packet.is_some() as u64) << 9)
            | ((packet.map_or(0, |p| p.latency_bits) as u64) << 10)
            | ((packet.map_or(0, |p| p.render_profile_settings_bits) as u64) << 18);
        if self.last_trace.swap(signature, Ordering::SeqCst) != signature {
            crate::logging::event!("peer_metadata",
                "version" => version, "valid" => packet.is_some(),
                "latency_bits" => packet.map(|p| p.latency_bits as u32),
                "render_bits" => packet.map(|p| p.render_profile_settings_bits as u32),
            );
        }
    }
}

pub trait StationExt {
    fn get_latency(&self) -> Option<Latency>;
    fn get_render_profile_settings(&self) -> Option<RenderProfileSettings>;
    fn get_render_profile(&self) -> Option<RenderProfile>;
}

impl StationExt for ConnectedStation {
    fn get_latency(&self) -> Option<Latency> {
        let id = self.get_id();
        let stations_table = CONNECTED_STATION_TABLE_ATOMIC_VIEW.load();
        stations_table
            .iter()
            .find(|s| s.id == id && s.is_valid_comms.load(Ordering::SeqCst))
            .map(|s| Latency::from_bits(s.latency_bits.load(Ordering::SeqCst)))
    }
    fn get_render_profile_settings(&self) -> Option<RenderProfileSettings> {
        if !self.is_modded() {
            return None;
        }
        let id = self.get_id();
        let stations_table = CONNECTED_STATION_TABLE_ATOMIC_VIEW.load();
        stations_table
            .iter()
            .find(|s| s.id == id && s.is_valid_comms.load(Ordering::SeqCst))
            .and_then(|s| {
                RenderProfileSettings::from_bits(
                    s.render_profile_settings_bits.load(Ordering::SeqCst),
                )
            })
    }
    fn get_render_profile(&self) -> Option<RenderProfile> {
        self.get_render_profile_settings()
            .map(|rps| RenderProfile::from_settings(&rps))
    }
}

fn on_station_connection_changed(
    event: ConnectionChangedEvent,
    station: ConnectedStation,
    new_num_connected: usize,
) {
    let id = station.get_id();
    let mut stations_table = CONNECTED_STATION_TABLE_SYNCED_INTERNAL.lock().unwrap();
    if event == ConnectionChangedEvent::StationConnected {
        stations_table.retain(|station| station.id != id);
        stations_table.push(StationNetInfo {
            id,
            is_valid_comms: AtomicBool::new(false),
            latency_bits: AtomicU8::new(Latency::unknown().to_bits()),
            render_profile_settings_bits: AtomicU16::new(
                RenderProfileSettings::vanilla().to_bits(),
            ),
            #[cfg(feature = "runtime_trace")]
            last_trace: AtomicU64::new(u64::MAX),
        });
    } else if event == ConnectionChangedEvent::StationDisconnected {
        if let Some(i) = stations_table.iter().position(|s| s.id == id) {
            stations_table.remove(i);
        }
    }

    CONNECTED_STATION_TABLE_ATOMIC_VIEW.store(Arc::new(stations_table.clone()));

    crate::logging::event!("peer_connection",
        "connected" => event == ConnectionChangedEvent::StationConnected,
        "peer_count" => new_num_connected,
    );
    RENDER_PROFILE_MANAGER.auto_select_profile(new_num_connected);
}

fn send_pia_data_hook(_station: ConnectedStation, data: &mut [u8]) {
    let Some(target) = data.get_mut(..PiaCustomNetPacket::LEN) else {
        return;
    };

    let latency_bits = LATENCY_SLIDER_MANAGER
        .active_latency()
        .unwrap_or(LATENCY_SLIDER_MANAGER.selected_latency())
        .to_bits();
    let rps_bits = match is_in_real_game() {
        false => RENDER_PROFILE_MANAGER.selected_render_profile_settings(),
        true => RENDER_PROFILE_MANAGER.active_render_profile_settings(),
    }
    .to_bits();
    let payload = PiaCustomNetPacket {
        latency_bits,
        render_profile_settings_bits: rps_bits,
    };
    target.copy_from_slice(&payload.encode());
}

fn receive_pia_data_hook(station: ConnectedStation, data: &[u8]) {
    let id = station.get_id();
    let stations_table = CONNECTED_STATION_TABLE_ATOMIC_VIEW.load();
    if let Some(station) = stations_table.iter().find(|s| s.id == id) {
        let packet = PiaCustomNetPacket::decode(data);
        #[cfg(feature = "runtime_trace")]
        station.trace_metadata(data, packet);
        let Some(data) = packet else {
            station.is_valid_comms.store(false, Ordering::SeqCst);
            println!("Invalid or incompatible custom peer metadata");
            return;
        };
        station
            .latency_bits
            .store(data.latency_bits, Ordering::SeqCst);
        station
            .render_profile_settings_bits
            .store(data.render_profile_settings_bits, Ordering::SeqCst);
        station.is_valid_comms.store(true, Ordering::SeqCst);
    }
}

pub(super) fn install() {
    #[cfg(feature = "runtime_trace")]
    {
        let version = crate::utils::lookup_symbol_addr(b"ssbu_pia_manager_api_version\0").map(
            |address| unsafe {
                let get_version: unsafe extern "C" fn() -> u32 = std::mem::transmute(address);
                get_version()
            },
        );
        crate::logging::event!("dependency_apis", "pia" => version);
    }
    ssbu_pia_interface::install();
    StationConnectionManager::set_enabled(true);
    StationConnectionManager::register_station_connection_changed_callback(
        on_station_connection_changed,
    );
    StationConnectionManager::register_station_data_send_hook(send_pia_data_hook);
    StationConnectionManager::register_station_data_received_hook(receive_pia_data_hook);

    #[cfg(feature = "dummy_connection")]
    ssbu_pia_interface::setup_dummy_connection(3);
}
