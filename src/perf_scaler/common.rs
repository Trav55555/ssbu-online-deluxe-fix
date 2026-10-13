use std::sync::atomic::{AtomicBool, AtomicI8, Ordering};

use smashline::{
    skyline_smash::{
        app::{
            self,
            lua_bind::{SlowModule, WorkModule},
        },
        lib::lua_const::FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID,
    },
    *,
};

use crate::perf_scaler::{pop_dynamic_res_report, push_dynamic_res_report, MAX_FIGHTERS};

const FINISH_COOLDOWN_FRAMES: i8 = 7;
static CRITICAL_HIT_ACTIVE: [AtomicBool; MAX_FIGHTERS] =
    [const { AtomicBool::new(false) }; MAX_FIGHTERS];
static CRITICAL_HIT_COOLDOWN: [AtomicI8; MAX_FIGHTERS] =
    [const { AtomicI8::new(FINISH_COOLDOWN_FRAMES) }; MAX_FIGHTERS];

pub(super) fn init() {
    for (active, cooldown) in CRITICAL_HIT_ACTIVE.iter().zip(&CRITICAL_HIT_COOLDOWN) {
        active.store(false, Ordering::SeqCst);
        cooldown.store(FINISH_COOLDOWN_FRAMES, Ordering::SeqCst);
    }
}

extern "C" fn global_critical_hit_fighter_frame(fighter: &mut L2CFighterCommon) {
    unsafe {
        let module_accessor = fighter.module_accessor;
        if module_accessor.is_null() {
            return;
        }
        let entry_id = WorkModule::get_int(module_accessor, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID);
        let Some(active) = CRITICAL_HIT_ACTIVE.get(entry_id as usize) else {
            return;
        };
        if !active.load(Ordering::SeqCst) {
            return;
        }
        let cooldown = &CRITICAL_HIT_COOLDOWN[entry_id as usize];
        if SlowModule::is_slow(module_accessor) {
            cooldown.store(FINISH_COOLDOWN_FRAMES, Ordering::SeqCst);
        } else if cooldown.fetch_sub(1, Ordering::SeqCst) <= 1 {
            if active.swap(false, Ordering::SeqCst) {
                pop_dynamic_res_report();
            }
        }
    }
}

#[skyline::hook(offset = 0x06a7100)]
unsafe fn fighter_cut_in_start(
    manager: u64,
    boma: *mut app::BattleObjectModuleAccessor,
    transactor: u64,
    cut_in_type: u32,
    data: u64,
    priority: u32,
) {
    call_original!(manager, boma, transactor, cut_in_type, data, priority);
    if boma.is_null() {
        return;
    }
    let entry_id = WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID);
    let Some(active) = CRITICAL_HIT_ACTIVE.get(entry_id as usize) else {
        return;
    };
    CRITICAL_HIT_COOLDOWN[entry_id as usize].store(FINISH_COOLDOWN_FRAMES, Ordering::SeqCst);
    if !active.swap(true, Ordering::SeqCst) {
        push_dynamic_res_report();
    }
}

pub fn install() {
    skyline::install_hooks!(fighter_cut_in_start);
    Agent::new("fighter")
        .on_line(Main, global_critical_hit_fighter_frame)
        .install();
}
