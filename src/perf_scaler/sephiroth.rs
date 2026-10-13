use std::sync::atomic::{AtomicBool, Ordering};

use smashline::{
    skyline_smash::{
        app::lua_bind::{StatusModule, WorkModule},
        lib::lua_const::{
            FIGHTER_EDGE_STATUS_KIND_SPECIAL_N_SHOOT,
            FIGHTER_EDGE_STATUS_SPECIAL_N_WORK_INT_CHARGE_KIND,
            FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID,
        },
    },
    *,
};

use crate::perf_scaler::{pop_dynamic_res_report, push_dynamic_res_report, MAX_FIGHTERS};

static GIGAFLARE_ACTIVE: [AtomicBool; MAX_FIGHTERS] =
    [const { AtomicBool::new(false) }; MAX_FIGHTERS];

pub(super) fn init() {
    for active in &GIGAFLARE_ACTIVE {
        active.store(false, Ordering::SeqCst);
    }
}

extern "C" fn sephiroth_fighter_frame(fighter: &mut L2CFighterCommon) {
    unsafe {
        let module_accessor = fighter.module_accessor;
        if module_accessor.is_null() {
            return;
        }
        let entry_id = WorkModule::get_int(module_accessor, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID);
        let Some(active) = GIGAFLARE_ACTIVE.get(entry_id as usize) else {
            return;
        };
        let status = StatusModule::status_kind(module_accessor);
        let charge_kind = WorkModule::get_int(
            module_accessor,
            *FIGHTER_EDGE_STATUS_SPECIAL_N_WORK_INT_CHARGE_KIND,
        );
        let fully_charged_shoot =
            status == *FIGHTER_EDGE_STATUS_KIND_SPECIAL_N_SHOOT && charge_kind >= 3;
        let was_active = active.swap(fully_charged_shoot, Ordering::SeqCst);
        match (was_active, fully_charged_shoot) {
            (false, true) => push_dynamic_res_report(),
            (true, false) => pop_dynamic_res_report(),
            _ => {}
        }
    }
}

pub fn install() {
    Agent::new("edge")
        .on_line(Main, sephiroth_fighter_frame)
        .install();
}
