use std::sync::{
    atomic::{AtomicBool, AtomicU16, Ordering},
    LazyLock,
};

use ssbu_pia_interface::StationConnectionManager;
use ultelier::sync_guest::{
    self, profile::DockedProfile, BufferMode, IndexMode, OverclockProfile, ResolutionLevel,
};

use crate::{input_poll::InputSnapshot, utils::is_emulator};

#[derive(Debug, Clone)]
pub struct RenderProfileSettings {
    buffer_mode: BufferMode,
    index_mode: IndexMode,
    default_resolution_level: ResolutionLevel,
    dynamic_res_enabled: bool,
    vsync_enabled: bool,
    render_opts_enabled: bool,
    overclock_profile: DockedProfile,
}

impl RenderProfileSettings {
    pub fn vanilla() -> Self {
        RenderProfileSettings {
            buffer_mode: BufferMode::Triple,
            index_mode: IndexMode::TwoBehind,
            default_resolution_level: ResolutionLevel::Res1920x1080,
            dynamic_res_enabled: false,
            vsync_enabled: true,
            render_opts_enabled: false,
            overclock_profile: DockedProfile::Rest,
        }
    }
    pub fn less_lag() -> Self {
        RenderProfileSettings {
            buffer_mode: BufferMode::Double,
            index_mode: IndexMode::OneBehind,
            default_resolution_level: ResolutionLevel::Res1920x1080,
            dynamic_res_enabled: !is_emulator(),
            vsync_enabled: false,
            render_opts_enabled: true,
            overclock_profile: DockedProfile::Singles,
        }
    }
    pub fn less_lag_ultra() -> Self {
        let (default_res, dynamic_res_enabled) = match is_emulator() {
            true => (ResolutionLevel::Res1920x1080, false),
            false => (ResolutionLevel::Res1024x576, true),
        };
        RenderProfileSettings {
            buffer_mode: BufferMode::Double,
            index_mode: IndexMode::Immediate,
            default_resolution_level: default_res,
            dynamic_res_enabled: dynamic_res_enabled,
            vsync_enabled: false,
            render_opts_enabled: true,
            overclock_profile: DockedProfile::Singles,
        }
    }
    pub fn less_lag_doubles() -> Self {
        RenderProfileSettings {
            buffer_mode: BufferMode::Triple,
            index_mode: IndexMode::OneBehind,
            default_resolution_level: ResolutionLevel::Res1920x1080,
            dynamic_res_enabled: true,
            vsync_enabled: false,
            render_opts_enabled: true,
            overclock_profile: DockedProfile::Ffa,
        }
    }
    pub fn buffer_mode(&self) -> BufferMode {
        self.buffer_mode
    }
    pub fn index_mode(&self) -> IndexMode {
        self.index_mode
    }
    pub fn default_resolution_level(&self) -> ResolutionLevel {
        self.default_resolution_level
    }
    pub fn dynamic_res_enabled(&self) -> bool {
        self.dynamic_res_enabled
    }
    pub fn vsync_enabled(&self) -> bool {
        self.vsync_enabled
    }
    pub fn render_opts_enabled(&self) -> bool {
        self.render_opts_enabled
    }
    pub fn overclock_profile(&self) -> DockedProfile {
        self.overclock_profile
    }
    pub fn from_render_profile(rp: RenderProfile) -> Self {
        match rp {
            RenderProfile::Vanilla => RenderProfileSettings::vanilla(),
            RenderProfile::LessLag => RenderProfileSettings::less_lag(),
            RenderProfile::LessLagUltra => RenderProfileSettings::less_lag_ultra(),
            RenderProfile::LessLagDoubles => RenderProfileSettings::less_lag_doubles(),
            _ => panic!("Must specific a valid render profile"),
        }
    }
    pub fn from_env() -> Self {
        Self {
            buffer_mode: sync_guest::buffer_mode()
                .flatten()
                .unwrap_or(BufferMode::Triple),
            index_mode: sync_guest::index_mode()
                .flatten()
                .unwrap_or(IndexMode::TwoBehind),
            default_resolution_level: sync_guest::default_game_resolution_level()
                .flatten()
                .unwrap_or(ResolutionLevel::Res1920x1080),
            dynamic_res_enabled: sync_guest::dynamic_resolution_enabled().unwrap_or(false),
            vsync_enabled: sync_guest::vsync_enabled().unwrap_or(true),
            render_opts_enabled: sync_guest::render_opts_enabled().unwrap_or(false),
            overclock_profile: match sync_guest::current_overclock_profile().flatten() {
                Some(OverclockProfile::PerformanceSingles) => DockedProfile::Singles,
                Some(OverclockProfile::PerformanceFfa) => DockedProfile::Ffa,
                _ => DockedProfile::Rest,
            },
        }
    }
    pub fn to_bits(&self) -> u16 {
        (self.buffer_mode as u16)
            | ((self.index_mode as u16) << 2)
            | ((self.default_resolution_level as u16) << 4)
            | ((self.dynamic_res_enabled as u16) << 11)
            | ((self.vsync_enabled as u16) << 12)
            | ((self.render_opts_enabled as u16) << 13)
            | ((self.overclock_profile as u16) << 14)
    }
    pub fn from_bits(bits: u16) -> Option<Self> {
        Some(Self {
            buffer_mode: BufferMode::from_u32((bits & 0b11) as u32)?,
            index_mode: IndexMode::from_u32(((bits >> 2) & 0b11) as u32)?,
            default_resolution_level: ResolutionLevel::from_u32(((bits >> 4) & 0x7f) as u32)?,
            dynamic_res_enabled: ((bits >> 11) & 1) != 0,
            vsync_enabled: ((bits >> 12) & 1) != 0,
            render_opts_enabled: ((bits >> 13) & 1) != 0,
            overclock_profile: match (bits >> 14) & 0b11 {
                0 => DockedProfile::Rest,
                1 => DockedProfile::Singles,
                2 => DockedProfile::Ffa,
                _ => return None,
            },
        })
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RenderProfile {
    Vanilla = 0,
    Custom = 1,
    LessLagDoubles = 2,
    LessLag = 3,
    LessLagUltra = 4,
}

impl RenderProfile {
    pub fn from_settings(rps: &RenderProfileSettings) -> Self {
        match (
            rps.buffer_mode,
            rps.index_mode,
            rps.vsync_enabled,
            rps.render_opts_enabled,
        ) {
            (BufferMode::Triple, IndexMode::TwoBehind, true, false) => RenderProfile::Vanilla,
            (BufferMode::Double, IndexMode::OneBehind, false, true) => RenderProfile::LessLag,
            (BufferMode::Double, IndexMode::Immediate, false, true) => RenderProfile::LessLagUltra,
            (BufferMode::Triple, IndexMode::OneBehind, false, true) => {
                RenderProfile::LessLagDoubles
            }
            _ => RenderProfile::Custom,
        }
    }
}

impl std::fmt::Display for RenderProfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            RenderProfile::Vanilla => "Vanilla",
            RenderProfile::LessLag => "LessLag",
            RenderProfile::LessLagUltra => "LLUltra",
            RenderProfile::LessLagDoubles => "LLDoubles",
            RenderProfile::Custom => "Custom",
        };
        write!(f, "{}", s)
    }
}

pub struct RenderProfileManager {
    auto_mode: AtomicBool,
    selected_profile_settings: AtomicU16,
}

impl RenderProfileManager {
    pub fn new() -> Self {
        RenderProfileManager {
            auto_mode: AtomicBool::new(true),
            selected_profile_settings: AtomicU16::new(RenderProfileSettings::vanilla().to_bits()),
        }
    }
    pub fn selected_render_profile_settings(&self) -> RenderProfileSettings {
        RenderProfileSettings::from_bits(self.selected_profile_settings.load(Ordering::SeqCst))
            .unwrap_or_else(RenderProfileSettings::vanilla)
    }
    pub fn active_render_profile_settings(&self) -> RenderProfileSettings {
        RenderProfileSettings::from_env()
    }
    pub fn selected_render_profile(&self) -> RenderProfile {
        RenderProfile::from_settings(&self.selected_render_profile_settings())
    }
    pub fn active_render_profile(&self) -> RenderProfile {
        RenderProfile::from_settings(&self.active_render_profile_settings())
    }
    fn select_render_profile_immediate(&self, rp: RenderProfile) {
        self.selected_profile_settings.store(
            RenderProfileSettings::from_render_profile(rp).to_bits(),
            Ordering::SeqCst,
        );
    }
    fn apply_profile_settings_immediate(&self, rps: &RenderProfileSettings) {
        let _resolution_result =
            sync_guest::set_default_game_resolution_level(rps.default_resolution_level);
        let _dynamic_result = sync_guest::set_dynamic_resolution_enabled(rps.dynamic_res_enabled);
        let mut _flags_result = None;
        if let Some(flags) = sync_guest::env_flags() {
            let flags = flags
                .with(
                    sync_guest::EnvironmentFlags::VSYNC_ENABLED,
                    rps.vsync_enabled,
                )
                .with(
                    sync_guest::EnvironmentFlags::RENDER_OPTS_ENABLED,
                    rps.render_opts_enabled,
                )
                .with(
                    sync_guest::EnvironmentFlags::TRIPLE_ENABLED,
                    rps.buffer_mode == BufferMode::Triple,
                )
                .with_value(
                    sync_guest::EnvironmentFlags::INDEX_MODE,
                    rps.index_mode as u32,
                );
            _flags_result = sync_guest::replace_env_flags(flags);
        }
        if !is_emulator() {
            sync_guest::profile::apply_docked_profile(rps.overclock_profile);
        }
        crate::logging::event!("render_apply",
            "requested_bits" => rps.to_bits(),
            "resolution_result" => _resolution_result,
            "dynamic_result" => _dynamic_result,
            "flags_result" => _flags_result.map(|flags| flags.bits()),
            "flags" => sync_guest::env_flags().map(|flags| flags.bits()),
            "resolution" => sync_guest::default_game_resolution_level().flatten().map(|level| level as u32),
            "dynamic" => sync_guest::dynamic_resolution_enabled(),
            "overclock" => sync_guest::current_overclock_profile().flatten().map(|profile| profile as u32),
        );
    }
    pub fn apply_selected_profile_settings(&self) {
        self.apply_profile_settings_immediate(&self.selected_render_profile_settings());
    }
    pub fn select_next_render_profile(&self) -> RenderProfile {
        let selected_rp = self.selected_render_profile();
        if self.is_auto_mode() {
            return selected_rp;
        }
        let next_rp = match self.selected_render_profile() {
            RenderProfile::Vanilla => RenderProfile::LessLag,
            RenderProfile::LessLag => RenderProfile::LessLagUltra,
            RenderProfile::LessLagUltra => RenderProfile::LessLagDoubles,
            RenderProfile::LessLagDoubles => RenderProfile::Vanilla,
            RenderProfile::Custom => RenderProfile::Vanilla,
        };
        let next_rps = RenderProfileSettings::from_render_profile(next_rp);
        self.selected_profile_settings
            .store(next_rps.to_bits(), Ordering::SeqCst);
        next_rp
    }
    pub fn select_prev_render_profile(&self) -> RenderProfile {
        let selected_rp = self.selected_render_profile();
        if self.is_auto_mode() {
            return selected_rp;
        }
        let prev_rp = match self.selected_render_profile() {
            RenderProfile::Vanilla => RenderProfile::LessLagDoubles,
            RenderProfile::LessLag => RenderProfile::Vanilla,
            RenderProfile::LessLagUltra => RenderProfile::LessLag,
            RenderProfile::LessLagDoubles => RenderProfile::LessLagUltra,
            RenderProfile::Custom => RenderProfile::Vanilla,
        };
        let prev_rps = RenderProfileSettings::from_render_profile(prev_rp);
        self.selected_profile_settings
            .store(prev_rps.to_bits(), Ordering::SeqCst);
        prev_rp
    }
    pub fn is_auto_mode(&self) -> bool {
        self.auto_mode.load(Ordering::SeqCst)
    }
    pub fn set_auto_mode(&self, auto_mode: bool) {
        self.auto_mode.store(auto_mode, Ordering::SeqCst);
        self.auto_select_profile(StationConnectionManager::num_connected_stations());
    }
    pub fn poll(
        &self,
        input_snapshot: &InputSnapshot,
        prev: ninput::Buttons,
        next: ninput::Buttons,
    ) -> bool {
        let pressed_buttons = input_snapshot.check_buttons_pressed(&[prev, next]);
        if pressed_buttons == prev {
            if self.is_auto_mode() {
                self.set_auto_mode(false);
                self.select_render_profile_immediate(RenderProfile::LessLagDoubles);
            } else {
                if self.selected_render_profile() == RenderProfile::Vanilla {
                    self.set_auto_mode(true);
                } else {
                    self.select_prev_render_profile();
                }
            }
            return true;
        } else if pressed_buttons == next {
            if self.is_auto_mode() {
                self.set_auto_mode(false);
                self.select_render_profile_immediate(RenderProfile::Vanilla);
            } else {
                if self.selected_render_profile() == RenderProfile::LessLagDoubles {
                    self.set_auto_mode(true);
                } else {
                    self.select_next_render_profile();
                }
            }
            return true;
        }
        false
    }
    pub fn auto_select_profile(&self, num_opponents: usize) {
        self.auto_select_profile_inner(num_opponents > 0, num_opponents > 2);
    }
    pub fn auto_select_match_profile(&self, is_doubles: bool) {
        self.auto_select_profile_inner(true, is_doubles);
    }
    fn auto_select_profile_inner(&self, is_connected: bool, is_doubles: bool) {
        if !self.is_auto_mode() {
            return;
        }
        if !is_connected {
            self.select_render_profile_immediate(RenderProfile::Vanilla);
            return;
        }
        match (is_emulator(), is_doubles) {
            (_, true) => self.select_render_profile_immediate(RenderProfile::LessLagDoubles),
            (true, _) => self.select_render_profile_immediate(RenderProfile::LessLagUltra),
            (false, _) => self.select_render_profile_immediate(RenderProfile::LessLag),
        };
    }
}

pub static RENDER_PROFILE_MANAGER: LazyLock<RenderProfileManager> =
    LazyLock::new(|| RenderProfileManager::new());

pub(crate) fn match_init() {
    let num_connected_stations = StationConnectionManager::num_connected_stations();
    if crate::net::is_connected()
        && crate::net::is_valid_online_mode()
        && crate::net::is_in_real_game()
    {
        RENDER_PROFILE_MANAGER.auto_select_match_profile(
            crate::net::get_match_status() == crate::net::MatchStatus::Doubles,
        );
        RENDER_PROFILE_MANAGER.apply_selected_profile_settings();
    } else {
        RENDER_PROFILE_MANAGER.apply_profile_settings_immediate(&RenderProfileSettings::vanilla());
        if !is_emulator() && crate::net::is_in_game() {
            let _ = sync_guest::profile::apply_singles();
        }
    }
    crate::logging::event!("render_state",
        "context" => "match_init",
        "flags" => sync_guest::env_flags().map(|flags| flags.bits()),
        "resolution" => sync_guest::default_game_resolution_level().flatten().map(|level| level as u32),
        "dynamic" => sync_guest::dynamic_resolution_enabled(),
        "overclock" => sync_guest::current_overclock_profile().flatten().map(|profile| profile as u32),
    );
    crate::logging::info!(
        "RENDER PROFILE MATCH INIT: selected={} active={} stations={}",
        RENDER_PROFILE_MANAGER.selected_render_profile(),
        RENDER_PROFILE_MANAGER.active_render_profile(),
        num_connected_stations
    );
}

pub(crate) fn match_cleanup() {
    RENDER_PROFILE_MANAGER.apply_profile_settings_immediate(&RenderProfileSettings::vanilla());
    crate::logging::info!(
        "RENDER PROFILE MATCH CLEANUP: active={}",
        RENDER_PROFILE_MANAGER.active_render_profile()
    );
}

pub(super) fn init() {
    let _ = LazyLock::force(&RENDER_PROFILE_MANAGER);

    RENDER_PROFILE_MANAGER.apply_profile_settings_immediate(&RenderProfileSettings::vanilla());
}
