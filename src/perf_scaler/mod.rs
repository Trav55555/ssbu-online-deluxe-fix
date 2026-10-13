use std::sync::atomic::{AtomicU8, Ordering};
use ultelier::sync_guest::{self, ResolutionLevel};

mod common;
mod sephiroth;

const MAX_FIGHTERS: usize = 8;

use crate::render::profile::RENDER_PROFILE_MANAGER;

static ENTRY_BASE_RESOLUTION: AtomicU8 = AtomicU8::new(ResolutionLevel::Res1920x1080 as u8);

pub(in crate::perf_scaler) fn push_dynamic_res_report(entry_id: u32) {
    let base_res_level =
        ResolutionLevel::from_u32(ENTRY_BASE_RESOLUTION.load(Ordering::SeqCst) as u32).unwrap();
    let target_res = if base_res_level >= ResolutionLevel::Res1024x576 {
        ResolutionLevel::Res854x480
    } else if base_res_level >= ResolutionLevel::Res1280x720 {
        ResolutionLevel::Res1024x576
    } else {
        ResolutionLevel::Res1280x720
    };

    let _result = sync_guest::push_dynamic_res_report(target_res);
    crate::logging::event!("dynamic_resolution", "action" => "push",
        "entry_id" => entry_id, "resolution" => target_res as u32, "result" => _result);
    let _ = entry_id;
}

pub(in crate::perf_scaler) fn pop_dynamic_res_report(entry_id: u32) {
    let base_res_level =
        ResolutionLevel::from_u32(ENTRY_BASE_RESOLUTION.load(Ordering::SeqCst) as u32).unwrap();
    let target_res = if base_res_level >= ResolutionLevel::Res1024x576 {
        ResolutionLevel::Res854x480
    } else if base_res_level >= ResolutionLevel::Res1280x720 {
        ResolutionLevel::Res1024x576
    } else {
        ResolutionLevel::Res1280x720
    };

    let _result = sync_guest::pop_dynamic_res_report(target_res);
    crate::logging::event!("dynamic_resolution", "action" => "pop",
        "entry_id" => entry_id, "resolution" => target_res as u32, "result" => _result);
    let _ = entry_id;
}

pub(crate) fn match_init() {
    let _result = sync_guest::clear_all_dynamic_res_report();
    crate::logging::event!("dynamic_resolution", "action" => "reset_init", "result" => _result);
    common::init();
    sephiroth::init();
    let rps = RENDER_PROFILE_MANAGER.active_render_profile_settings();
    let base_res_level = rps.default_resolution_level();
    ENTRY_BASE_RESOLUTION.store(base_res_level as u8, Ordering::SeqCst);
}

pub(crate) fn match_cleanup() {
    let _result = sync_guest::clear_all_dynamic_res_report();
    crate::logging::event!("dynamic_resolution", "action" => "reset_cleanup", "result" => _result);
    common::init();
    sephiroth::init();
}

pub fn install() {
    common::install();
    sephiroth::install();
}
