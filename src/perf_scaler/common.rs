use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};

use smashline::{
    skyline_smash::{
        app::{
            self,
            lua_bind::{CameraModule, WorkModule},
        },
        lib::lua_const::FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID,
    },
    *,
};

use crate::perf_scaler::{pop_dynamic_res_report, push_dynamic_res_report};

static ZOOM_IN_ATTACK_LANDED: AtomicBool = AtomicBool::new(false);
static ZOOM_PENDING_FRAMES: AtomicI32 = AtomicI32::new(0);
const ZOOM_PENDING_TIMEOUT_FRAMES: i32 = 45;

extern "C" fn global_camera_zoom_state_fighter_frame(fighter: &mut L2CFighterCommon) {
    static mut ZOOM_ACTIVE: bool = false;
    static mut ZOOM_FINISH_COOLDOWN_FRAMES: i32 = 7;

    unsafe {
        let module_accessor =
            app::sv_system::battle_object_module_accessor(fighter.lua_state_agent);

        let camera_type = CameraModule::get_camera_type(module_accessor) as i32;

        if !ZOOM_IN_ATTACK_LANDED.load(Ordering::SeqCst) {
            return;
        }

        let entry_id = WorkModule::get_int(module_accessor, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID);
        if entry_id != 0 {
            return;
        }
        let camera_zoom_active_now = camera_type == 3;

        if camera_zoom_active_now {
            ZOOM_PENDING_FRAMES.store(ZOOM_PENDING_TIMEOUT_FRAMES, Ordering::SeqCst);
            ZOOM_FINISH_COOLDOWN_FRAMES = 7;
            if !ZOOM_ACTIVE {
                ZOOM_ACTIVE = true;
            }
        } else if ZOOM_ACTIVE {
            ZOOM_FINISH_COOLDOWN_FRAMES -= 1;
            if ZOOM_FINISH_COOLDOWN_FRAMES <= 0 {
                ZOOM_ACTIVE = false;
                pop_dynamic_res_report();
                ZOOM_IN_ATTACK_LANDED.store(false, Ordering::SeqCst);
                ZOOM_PENDING_FRAMES.store(0, Ordering::SeqCst);
            }
        } else if ZOOM_PENDING_FRAMES.fetch_sub(1, Ordering::SeqCst) <= 1 {
            pop_dynamic_res_report();
            ZOOM_IN_ATTACK_LANDED.store(false, Ordering::SeqCst);
            ZOOM_PENDING_FRAMES.store(0, Ordering::SeqCst);
        }
    }
}

#[skyline::hook(replace=app::sv_animcmd::EFFECT_GLOBAL_BACK_GROUND_CUT_IN_CENTER_POS)]
unsafe fn cut_in_center(lua_state: u64) {
    if !ZOOM_IN_ATTACK_LANDED.swap(true, Ordering::SeqCst) {
        push_dynamic_res_report();
    }
    ZOOM_PENDING_FRAMES.store(ZOOM_PENDING_TIMEOUT_FRAMES, Ordering::SeqCst);
    call_original!(lua_state);
}

pub fn install() {
    skyline::install_hooks!(cut_in_center);
    Agent::new("fighter")
        .on_line(Main, global_camera_zoom_state_fighter_frame)
        .install();
}
