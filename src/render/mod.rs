use ultelier::sync_guest::SsbuSyncConfig;

use crate::utils::is_emulator;

pub mod profile;

extern "C" {
    fn nx_over_configure_nstuff_oc() -> u32;
}

pub(super) fn on_nro_load() {
    profile::init();
}

pub fn install() {
    let is_emulator = is_emulator();

    let mut config = SsbuSyncConfig::vanilla();
    config.overclocker = !is_emulator;
    if !is_emulator {
        unsafe { nx_over_configure_nstuff_oc() };
    }

    ultelier::sync_guest::install(config);
    profile::init();
}
