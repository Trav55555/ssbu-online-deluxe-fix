//! Runs the production render manager against a controllable SsbuSync boundary.
//! This tests reporting logic, not the real game's FFI or rendering behavior.
#![allow(dead_code)]

extern crate self as ninput;
extern crate self as ssbu_pia_interface;
extern crate self as ultelier;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Buttons(u64);
impl Buttons {
    pub const LEFT: Self = Self(1);
    pub const RIGHT: Self = Self(2);
}

pub struct StationConnectionManager;
impl StationConnectionManager {
    pub fn num_connected_stations() -> usize {
        1
    }
}

mod input_poll {
    pub struct InputSnapshot;
    impl InputSnapshot {
        pub fn check_buttons_pressed(&self, _: &[crate::Buttons]) -> crate::Buttons {
            crate::Buttons(0)
        }
    }
}
mod utils {
    pub fn is_emulator() -> bool {
        false
    }
}
mod net {
    use std::cell::Cell;
    #[derive(Clone, Copy, PartialEq)]
    pub enum MatchStatus {
        Inactive,
        Singles,
        Doubles,
        Training,
    }
    thread_local! { static STATUS: Cell<MatchStatus> = const { Cell::new(MatchStatus::Singles) }; }
    pub fn set_match_status(status: MatchStatus) {
        STATUS.with(|s| s.set(status));
    }
    pub fn get_match_status() -> MatchStatus {
        STATUS.with(|s| s.get())
    }
    pub fn is_in_game() -> bool {
        get_match_status() != MatchStatus::Inactive
    }
    pub fn is_in_real_game() -> bool {
        matches!(
            get_match_status(),
            MatchStatus::Singles | MatchStatus::Doubles
        )
    }
    pub fn is_valid_online_mode() -> bool {
        true
    }
    pub fn is_connected() -> bool {
        true
    }
}
#[macro_export]
macro_rules! host_info { ($($args:tt)*) => { let _ = format_args!($($args)*); }; }
mod logging {
    pub use crate::host_info as info;
}

pub mod sync_guest {
    use std::cell::RefCell;

    macro_rules! wire_enum {
        ($name:ident { $($variant:ident = $value:literal),+ }) => {
            #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
            pub enum $name { $($variant = $value),+ }
            impl $name {
                pub fn from_u32(value: u32) -> Option<Self> {
                    match value { $($value => Some(Self::$variant),)+ _ => None }
                }
            }
        };
    }
    wire_enum!(BufferMode { Double = 2, Triple = 3 });
    wire_enum!(IndexMode { Immediate = 0, OneBehind = 1, TwoBehind = 2 });
    wire_enum!(ResolutionLevel {
        Res1920x1080 = 0, Res1600x900 = 1, Res1280x720 = 2,
        Res1024x576 = 3, Res854x480 = 4
    });
    wire_enum!(OverclockProfile { PerformanceSingles = 1, PerformanceFfa = 2, Rest = 3 });

    #[derive(Clone, Copy)]
    pub struct EnvironmentFlags(u32);
    impl EnvironmentFlags {
        pub const TRIPLE_ENABLED: u32 = 1 << 2;
        pub const INDEX_MODE: u32 = 3 << 3;
        pub const VSYNC_ENABLED: u32 = 1 << 5;
        pub const RENDER_OPTS_ENABLED: u32 = 1 << 6;
        pub fn with(self, mask: u32, value: bool) -> Self {
            Self(if value { self.0 | mask } else { self.0 & !mask })
        }
        pub fn with_value(self, mask: u32, value: u32) -> Self {
            Self((self.0 & !mask) | ((value << mask.trailing_zeros()) & mask))
        }
        fn vanilla() -> Self {
            Self(Self::TRIPLE_ENABLED | Self::VSYNC_ENABLED | (2 << 3))
        }
    }
    pub struct Backend {
        pub available: bool,
        pub restricted: bool,
        pub flags: EnvironmentFlags,
        pub resolution: ResolutionLevel,
        pub dynamic: bool,
        pub overclock: OverclockProfile,
    }
    impl Default for Backend {
        fn default() -> Self {
            Self {
                available: true,
                restricted: false,
                flags: EnvironmentFlags::vanilla(),
                resolution: ResolutionLevel::Res1920x1080,
                dynamic: false,
                overclock: OverclockProfile::Rest,
            }
        }
    }
    thread_local! { static BACKEND: RefCell<Backend> = RefCell::new(Backend::default()); }
    pub fn control<T>(f: impl FnOnce(&mut Backend) -> T) -> T {
        BACKEND.with(|b| f(&mut b.borrow_mut()))
    }
    fn read<T>(f: impl FnOnce(&Backend) -> T) -> Option<T> {
        control(|b| b.available.then(|| f(b)))
    }
    pub fn env_flags() -> Option<EnvironmentFlags> {
        read(|b| b.flags)
    }
    pub fn replace_env_flags(flags: EnvironmentFlags) -> Option<EnvironmentFlags> {
        control(|b| {
            if !b.available {
                return None;
            }
            b.flags = if b.restricted {
                EnvironmentFlags::vanilla()
            } else {
                flags
            };
            Some(b.flags)
        })
    }
    pub fn set_default_game_resolution_level(level: ResolutionLevel) -> Option<bool> {
        control(|b| {
            if !b.available {
                return None;
            }
            if !b.restricted {
                b.resolution = level;
            }
            Some(!b.restricted)
        })
    }
    pub fn set_dynamic_resolution_enabled(enabled: bool) -> Option<bool> {
        control(|b| {
            if !b.available {
                return None;
            }
            if !b.restricted {
                b.dynamic = enabled;
            }
            Some(!b.restricted)
        })
    }
    pub fn buffer_mode() -> Option<Option<BufferMode>> {
        read(|b| {
            Some(if b.flags.0 & EnvironmentFlags::TRIPLE_ENABLED != 0 {
                BufferMode::Triple
            } else {
                BufferMode::Double
            })
        })
    }
    pub fn index_mode() -> Option<Option<IndexMode>> {
        read(|b| IndexMode::from_u32((b.flags.0 & EnvironmentFlags::INDEX_MODE) >> 3))
    }
    pub fn default_game_resolution_level() -> Option<Option<ResolutionLevel>> {
        read(|b| Some(b.resolution))
    }
    pub fn dynamic_resolution_enabled() -> Option<bool> {
        read(|b| b.dynamic)
    }
    pub fn vsync_enabled() -> Option<bool> {
        read(|b| b.flags.0 & EnvironmentFlags::VSYNC_ENABLED != 0)
    }
    pub fn render_opts_enabled() -> Option<bool> {
        read(|b| b.flags.0 & EnvironmentFlags::RENDER_OPTS_ENABLED != 0)
    }
    pub fn current_overclock_profile() -> Option<Option<OverclockProfile>> {
        read(|b| Some(b.overclock))
    }
    pub mod profile {
        #[derive(Debug, Clone, Copy)]
        pub enum DockedProfile {
            Rest = 0,
            Singles = 1,
            Ffa = 2,
        }
        pub fn apply_docked_profile(state: DockedProfile) {
            super::control(|b| {
                b.overclock = match state {
                    DockedProfile::Rest => super::OverclockProfile::Rest,
                    DockedProfile::Singles => super::OverclockProfile::PerformanceSingles,
                    DockedProfile::Ffa => super::OverclockProfile::PerformanceFfa,
                }
            });
        }
        pub fn apply_singles() {
            apply_docked_profile(DockedProfile::Singles);
        }
    }
}

#[path = "../src/render/profile.rs"]
mod production;

#[test]
fn rejected_profile_is_not_reported_as_active() {
    sync_guest::control(|b| b.restricted = true);
    let manager = production::RenderProfileManager::new();
    manager.set_auto_mode(false);
    manager.select_next_render_profile();
    manager.apply_selected_profile_settings();
    assert_eq!(
        manager.selected_render_profile(),
        production::RenderProfile::LessLag
    );
    assert_eq!(
        manager.active_render_profile(),
        production::RenderProfile::Vanilla
    );
}

#[test]
fn runtime_change_is_visible_without_a_local_profile_request() {
    let manager = production::RenderProfileManager::new();
    sync_guest::control(|b| {
        b.flags = b
            .flags
            .with(sync_guest::EnvironmentFlags::TRIPLE_ENABLED, false)
            .with(sync_guest::EnvironmentFlags::VSYNC_ENABLED, false)
            .with(sync_guest::EnvironmentFlags::RENDER_OPTS_ENABLED, true)
            .with_value(sync_guest::EnvironmentFlags::INDEX_MODE, 1);
    });
    assert_eq!(
        manager.active_render_profile(),
        production::RenderProfile::LessLag
    );
    assert_eq!(
        manager.selected_render_profile(),
        production::RenderProfile::Vanilla
    );
}

#[test]
fn accepted_request_reports_runtime_flags_and_resolution() {
    let manager = production::RenderProfileManager::new();
    manager.set_auto_mode(false);
    manager.select_next_render_profile();
    manager.select_next_render_profile();
    manager.apply_selected_profile_settings();
    let active = manager.active_render_profile_settings();
    assert_eq!(
        manager.active_render_profile(),
        production::RenderProfile::LessLagUltra
    );
    assert_eq!(
        active.default_resolution_level(),
        sync_guest::ResolutionLevel::Res1024x576
    );
    assert!(!active.vsync_enabled());
    assert!(active.render_opts_enabled());
}

#[test]
fn unavailable_runtime_does_not_report_requested_optimizations() {
    sync_guest::control(|b| b.available = false);
    let manager = production::RenderProfileManager::new();
    manager.set_auto_mode(false);
    manager.select_next_render_profile();
    manager.apply_selected_profile_settings();
    assert_eq!(
        manager.active_render_profile(),
        production::RenderProfile::Vanilla
    );
}

#[test]
fn doubles_profile_does_not_require_multiple_peer_consoles() {
    let manager = production::RenderProfileManager::new();
    assert_eq!(StationConnectionManager::num_connected_stations(), 1);
    manager.auto_select_match_profile(true);
    assert_eq!(
        manager.selected_render_profile(),
        production::RenderProfile::LessLagDoubles
    );
    manager.auto_select_match_profile(false);
    assert_eq!(
        manager.selected_render_profile(),
        production::RenderProfile::LessLag
    );
}

#[test]
fn training_resets_active_rendering_but_keeps_the_next_match_selection() {
    let manager = &production::RENDER_PROFILE_MANAGER;
    manager.set_auto_mode(false);
    manager.select_next_render_profile();
    manager.apply_selected_profile_settings();
    assert_eq!(
        manager.active_render_profile(),
        production::RenderProfile::LessLag
    );
    net::set_match_status(net::MatchStatus::Training);
    production::match_init();
    assert_eq!(
        manager.active_render_profile(),
        production::RenderProfile::Vanilla
    );
    assert_eq!(
        manager.selected_render_profile(),
        production::RenderProfile::LessLag
    );
}

#[test]
fn peer_metadata_reports_the_runtime_overclock_profile() {
    let manager = production::RenderProfileManager::new();
    sync_guest::control(|b| b.overclock = sync_guest::OverclockProfile::PerformanceFfa);
    assert_eq!(manager.active_render_profile_settings().to_bits() >> 14, 2);
}
