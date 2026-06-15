use smashline::{
    skyline_smash::{
        app::{
            self,
            lua_bind::{StatusModule, WorkModule},
        },
        lib::lua_const::{
            FIGHTER_EDGE_STATUS_KIND_SPECIAL_N_SHOOT,
            FIGHTER_EDGE_STATUS_SPECIAL_N_WORK_INT_CHARGE_KIND,
        },
    },
    *,
};

use crate::perf_scaler::{pop_dynamic_res_report, push_dynamic_res_report};

extern "C" fn sephiroth_fighter_frame(fighter: &mut L2CFighterCommon) {
    static mut GIGAFLARE_ACTIVE: bool = false;
    static mut PREV_STATUS: i32 = i32::MIN;
    static mut PREV_CHARGE_KIND: i32 = i32::MIN;
    static mut PREV_FULLY_CHARGED_SHOOT: bool = false;
    unsafe {
        let module_accessor =
            app::sv_system::battle_object_module_accessor(fighter.lua_state_agent);

        let status = StatusModule::status_kind(module_accessor);
        let charge_kind = WorkModule::get_int(
            module_accessor,
            *FIGHTER_EDGE_STATUS_SPECIAL_N_WORK_INT_CHARGE_KIND,
        );
        let fully_charged_shoot =
            status == *FIGHTER_EDGE_STATUS_KIND_SPECIAL_N_SHOOT && charge_kind >= 2;

        if PREV_STATUS != status
            || PREV_CHARGE_KIND != charge_kind
            || PREV_FULLY_CHARGED_SHOOT != fully_charged_shoot
        {
            PREV_STATUS = status;
            PREV_CHARGE_KIND = charge_kind;
            PREV_FULLY_CHARGED_SHOOT = fully_charged_shoot;
        }

        if fully_charged_shoot {
            if !GIGAFLARE_ACTIVE {
                GIGAFLARE_ACTIVE = true;
                push_dynamic_res_report();
            }
        } else if GIGAFLARE_ACTIVE {
            GIGAFLARE_ACTIVE = false;
            pop_dynamic_res_report();
        }
    }
}

pub fn install() {
    Agent::new("edge")
        .on_line(Main, sephiroth_fighter_frame)
        .install();
}
