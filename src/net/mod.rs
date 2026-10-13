pub mod latency_slider;
pub mod ldn;
pub mod pia;

mod match_state;
use match_state::MatchState;
pub use match_state::MatchStatus;

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use crate::net::ldn::interface::{get_network_role, NetworkRole};
use crate::utils::is_emulator;
use skyline::hooks::InlineCtx;
use skyline::nn::ui2d::Pane;
use ssbu_pia_interface::StationConnectionManager;

static mut LOCAL_ONLINE_CSS_NUM_PANES_ADJUSTED: bool = false;
static mut CURRENT_ARENA_ID: String = String::new();

static ONLINE_ARENA_PANE_HANDLE: AtomicU64 = AtomicU64::new(0);
static LOCAL_ROOM_PANE_HANDLE: AtomicU64 = AtomicU64::new(0);

// Online quickplay/Elite Smash does not expose the arena/local-room pane handles
// used by the original valid-online-mode check. Track that scene separately so
// the same latency/render/UI paths can run there too.
static ONLINE_MELEE_MODE: AtomicBool = AtomicBool::new(false);

static MATCH_STATE: MatchState = MatchState::new();

extern "C" {
    #[link_name = "\u{1}_ZN3app9smashball16is_training_modeEv"]
    fn is_training_mode() -> bool;

    #[link_name = "\u{1}_ZN3app7fighter23get_fighter_entry_countEv"]
    fn get_fighter_entry_count() -> i32;
}

#[skyline::hook(offset = 0x22d9d10, inline)]
unsafe fn online_melee_any_scene_create(_: &InlineCtx) {
    println!("ONLINE ELITE INIT");
    crate::logging::info!("ONLINE ELITE INIT");
    crate::logging::event!("scene", "name" => "online_melee");
    LOCAL_ROOM_PANE_HANDLE.store(0, Ordering::SeqCst);
    ONLINE_ARENA_PANE_HANDLE.store(0, Ordering::SeqCst);
    ONLINE_MELEE_MODE.store(true, Ordering::SeqCst);
    update_match_status(MatchStatus::Inactive, false);
}

#[skyline::hook(offset = 0x22d9c40, inline)]
unsafe fn bg_matchmaking_seq(_: &InlineCtx) {
    println!("ONLINE BG MM INIT");
    crate::logging::info!("ONLINE BG MM INIT");
    crate::logging::event!("scene", "name" => "matchmaking");
    LOCAL_ROOM_PANE_HANDLE.store(0, Ordering::SeqCst);
    ONLINE_ARENA_PANE_HANDLE.store(0, Ordering::SeqCst);
    ONLINE_MELEE_MODE.store(true, Ordering::SeqCst);
    update_match_status(MatchStatus::Inactive, false);
}

#[skyline::hook(offset = 0x235aaa0, inline)]
unsafe fn main_menu(_: &InlineCtx) {
    println!("MAIN MENU INIT");
    crate::logging::info!("MAIN MENU INIT");
    crate::logging::event!("scene", "name" => "main_menu");
    LOCAL_ROOM_PANE_HANDLE.store(0, Ordering::SeqCst);
    ONLINE_ARENA_PANE_HANDLE.store(0, Ordering::SeqCst);
    ONLINE_MELEE_MODE.store(false, Ordering::SeqCst);
    update_match_status(MatchStatus::Inactive, false);
}

#[skyline::hook(offset = 0x22d9b50, inline)]
unsafe fn arena_seq(_: &InlineCtx) {
    crate::logging::event!("scene", "name" => "arena_menu");
    println!("ONLINE ARENA INIT");
    crate::logging::info!("ONLINE ARENA INIT");
    ONLINE_MELEE_MODE.store(false, Ordering::SeqCst);
    update_match_status(MatchStatus::Inactive, false);
}

#[skyline::hook(offset = 0x18881f0, inline)]
unsafe fn online_arena_update_room_hook(_: &skyline::hooks::InlineCtx) {
    let pane_handle = ONLINE_ARENA_PANE_HANDLE.load(Ordering::SeqCst) as *mut u64 as *mut Pane;
    if !pane_handle.is_null() {
        crate::ui::native::update_online_arena_ui(pane_handle, CURRENT_ARENA_ID.clone());
    }
}

#[skyline::hook(offset = 0x1887b1c, inline)]
unsafe fn online_arena_set_room_id(ctx: &skyline::hooks::InlineCtx) {
    crate::logging::event!("scene", "name" => "arena_room");
    println!("ONLINE ARENA INIT");
    crate::logging::info!("ONLINE ARENA INIT");
    ONLINE_MELEE_MODE.store(false, Ordering::SeqCst);
    let panel = *((*((ctx.registers[0].x() + 8) as *const u64) + 0x10) as *const u64);
    ONLINE_ARENA_PANE_HANDLE.store(panel, Ordering::SeqCst);
    CURRENT_ARENA_ID = String::from_utf16(std::slice::from_raw_parts(
        ctx.registers[3].x() as *const u16,
        5,
    ))
    .unwrap_or_else(|_| String::from("?????"));
    update_match_status(MatchStatus::Inactive, false);
}

// called on local online menu init
#[skyline::hook(offset = 0x1bd45e0, inline)]
unsafe fn store_local_menu_pane(ctx: &InlineCtx) {
    println!("LOCAL ONLINE INIT");
    crate::logging::info!("LOCAL ONLINE INIT");
    crate::logging::event!("scene", "name" => "local_wireless");
    ONLINE_MELEE_MODE.store(false, Ordering::SeqCst);
    LOCAL_ONLINE_CSS_NUM_PANES_ADJUSTED = false;
    let handle = *((*((ctx.registers[0].x() + 8) as *const u64) + 0x10) as *const u64);
    LOCAL_ROOM_PANE_HANDLE.store(handle, Ordering::SeqCst);
    update_match_status(MatchStatus::Inactive, false);
}

#[skyline::hook(offset = 0x1bd7a80, inline)]
unsafe fn update_local_menu(_: &InlineCtx) {
    let pane_handle = LOCAL_ROOM_PANE_HANDLE.load(Ordering::SeqCst) as *mut u64 as *mut Pane;
    if !pane_handle.is_null() {
        crate::ui::native::update_local_online_ui(pane_handle);
    }
}

#[skyline::hook(offset = 0x1a26200)]
unsafe fn css_player_pane_num_changed(param_1: i64, prev_num: i32, changed_by_player: u32) {
    if is_local_online_mode()
        && !LOCAL_ONLINE_CSS_NUM_PANES_ADJUSTED
        && changed_by_player == 0
        && get_network_role() == NetworkRole::Host
    {
        LOCAL_ONLINE_CSS_NUM_PANES_ADJUSTED = true;
        *((param_1 + 0x160) as *mut i32) = 2;
    }
    update_match_status(MatchStatus::Inactive, false);
    call_original!(param_1, prev_num, changed_by_player);
}

#[skyline::hook(offset = 0x1a12f60)]
unsafe fn update_css(arg: u64) {
    if is_valid_online_mode() && arg != 0 {
        let pane_holder = *((arg + 0xe58) as *const u64);
        if pane_holder != 0 {
            let banner_pane1_ptr = *((pane_holder + 0x10) as *const u64) as *mut Pane;
            if !banner_pane1_ptr.is_null() {
                crate::ui::native::update_css_ui(banner_pane1_ptr);
            }
        }
    }
    call_original!(arg);
}

fn update_match_status(status: MatchStatus, force_update: bool) {
    if MATCH_STATE.set(status, force_update) {
        crate::logging::event!("match_status",
            "status" => format!("{status:?}").as_str(),
            "forced" => force_update,
            "mode" => online_mode_name(),
        );
        println!("UPDATE MATCH STATUS: {:?}", status);
        crate::logging::info!("UPDATE MATCH STATUS: {:?}", status);
        if status != MatchStatus::Inactive {
            crate::render::profile::match_init();
            if !is_emulator() {
                crate::perf_scaler::match_init();
            }
        } else {
            latency_slider::match_cleanup();
            if !is_emulator() {
                crate::perf_scaler::match_cleanup();
            }
            crate::render::profile::match_cleanup();
        }
    }
}

#[inline]
pub fn is_local_online_mode() -> bool {
    return LOCAL_ROOM_PANE_HANDLE.load(Ordering::SeqCst) > 0;
}

#[inline]
pub fn is_online_arena_mode() -> bool {
    return ONLINE_ARENA_PANE_HANDLE.load(Ordering::SeqCst) > 0;
}

#[inline]
pub fn is_online_melee_mode() -> bool {
    return ONLINE_MELEE_MODE.load(Ordering::SeqCst);
}

#[inline]
pub fn is_valid_online_mode() -> bool {
    #[cfg(feature = "dummy_connection")]
    return true;

    #[cfg(not(feature = "dummy_connection"))]
    return is_online_arena_mode() || is_local_online_mode() || is_online_melee_mode();
}

#[inline]
pub fn online_mode_name() -> &'static str {
    if is_online_arena_mode() {
        "arena"
    } else if is_local_online_mode() {
        "local_online"
    } else if is_online_melee_mode() {
        "quickplay_elite"
    } else {
        "none"
    }
}

#[inline]
pub fn is_connected() -> bool {
    return StationConnectionManager::is_connected();
}

#[inline]
pub fn get_match_status() -> MatchStatus {
    MATCH_STATE.get()
}

#[inline]
pub fn is_in_game() -> bool {
    get_match_status() != MatchStatus::Inactive
}

#[inline]
pub fn is_in_real_game() -> bool {
    get_match_status().is_real_game()
}

#[inline]
pub fn is_in_valid_online_game() -> bool {
    is_valid_online_mode() && is_in_real_game() && is_connected()
}

#[skyline::hook(offset = 0x25d9320, inline)]
unsafe fn on_stage_presetup(ctx: &InlineCtx) {
    let stage_base = ctx.registers[24].x();
    let stage_id = *((stage_base + 8) as *mut u32);

    let training = is_training_mode();
    let fighter_count = if training || matches!(stage_id, 310 | 354 | 311) {
        None
    } else {
        Some(get_fighter_entry_count())
    };
    let status = MatchStatus::from_stage(stage_id, training, fighter_count.unwrap_or(0));
    crate::logging::event!("stage",
        "stage_id" => stage_id,
        "training" => training,
        "fighter_count" => fighter_count,
    );
    update_match_status(status, status != MatchStatus::Inactive);
    crate::logging::info!(
        "STAGE PRESETUP: stage={} training={} fighters={:?} status={:?}",
        stage_id,
        training,
        fighter_count,
        status
    );
}

pub fn install() {
    skyline::install_hooks!(
        online_melee_any_scene_create,
        bg_matchmaking_seq,
        main_menu,
        online_arena_set_room_id,
        online_arena_update_room_hook,
        store_local_menu_pane,
        update_local_menu,
        update_css,
        css_player_pane_num_changed,
        on_stage_presetup,
        arena_seq,
    );
    latency_slider::install();
    pia::install();
}
