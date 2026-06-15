mod utils;

use ssbu_pia_interface::{NetworkStability, StationConnectionManager};
use utils::{PaneExt, TextBoxExt};

use skyline::nn::ui2d::{HorizontalPosition, Pane, PaneFlag, TextBoxFlag, VerticalPosition};

use crate::{
    input_poll::{InputSnapshot, PollerContext, POLLER},
    net::{latency_slider::LATENCY_SLIDER_MANAGER, pia::StationExt},
    render::profile::RENDER_PROFILE_MANAGER,
};

static NATIVE_UI_POLLER_CONTEXT: PollerContext =
    PollerContext::new(std::time::Duration::from_millis(167));

fn poll_station_cycle(input_snapshot: &InputSnapshot) -> i8 {
    let station_cycle_poll = input_snapshot.check_buttons_pressed(&[
        ninput::Buttons::ZL | ninput::Buttons::ZR | ninput::Buttons::LEFT,
        ninput::Buttons::ZL | ninput::Buttons::ZR | ninput::Buttons::RIGHT,
        ninput::Buttons::L | ninput::Buttons::R | ninput::Buttons::LEFT,
        ninput::Buttons::L | ninput::Buttons::R | ninput::Buttons::RIGHT,
    ]);
    if station_cycle_poll.intersects(ninput::Buttons::LEFT) {
        return -1;
    } else if station_cycle_poll.intersects(ninput::Buttons::RIGHT) {
        return 1;
    }
    0
}

pub unsafe fn update_css_ui(banner_pane1_ptr: *mut Pane) {
    if banner_pane1_ptr.is_null() {
        return;
    }

    static mut CURR_STATION_INDEX: usize = 0;
    const CELL_WIDTH: usize = 15;
    if !crate::ui::overlay::is_window_interactable() {
        let input_snapshot = POLLER.snapshot(&NATIVE_UI_POLLER_CONTEXT);
        let direction = poll_station_cycle(&input_snapshot);
        LATENCY_SLIDER_MANAGER.poll(
            &input_snapshot,
            ninput::Buttons::LEFT,
            ninput::Buttons::RIGHT,
        );
        RENDER_PROFILE_MANAGER.poll(&input_snapshot, ninput::Buttons::DOWN, ninput::Buttons::UP);
        let num_stations = StationConnectionManager::num_connected_stations();
        if num_stations > 0 {
            CURR_STATION_INDEX = (CURR_STATION_INDEX as isize + direction as isize)
                .rem_euclid(num_stations as isize) as usize;
        } else {
            CURR_STATION_INDEX = 0;
        }
    }
    let latency = LATENCY_SLIDER_MANAGER.selected_latency();
    let rp = RENDER_PROFILE_MANAGER.selected_render_profile();
    let rp_is_auto = RENDER_PROFILE_MANAGER.is_auto_mode();

    let (station_line_str, r, g, b, a) = if let Some(station) =
        StationConnectionManager::get_connected_station(CURR_STATION_INDEX)
    {
        let station_ping = station.get_rtt();
        let station_latency = station.get_latency();
        let station_rp = station.get_render_profile();
        let station_network_stability = station.get_network_stability();
        let station_latency_str = station_latency
            .map(|l| l.to_string())
            .unwrap_or_else(|| String::from("---"));
        let station_ping_str = station_ping
            .map(|rtt| format!("{}ms", rtt))
            .unwrap_or_else(|| String::from("---"));
        let station_rp_str = station_rp
            .map(|p| p.to_string())
            .unwrap_or_else(|| String::from("---"));
        let opp_cell_1 = format!("{:^width$}", station_rp_str, width = CELL_WIDTH);
        let opp_cell_2 = format!(
            "{:^width$}",
            format!("{} ({})", station_latency_str, station_ping_str),
            width = CELL_WIDTH
        );
        let (r, g, b, a) = match station_network_stability {
            NetworkStability::Stable => (0, 255, 0, 255),
            NetworkStability::Inconsistent => (255, 255, 0, 255),
            NetworkStability::Unstable => (255, 0, 0, 255),
        };
        let opp_name = station.get_main_account_name().unwrap_or_else(|| "???");
        (
            format!("\n{}: {} | {}", opp_name, opp_cell_1, opp_cell_2),
            r,
            g,
            b,
            a,
        )
    } else {
        (format!(""), 255, 255, 255, 255)
    };

    let local_rp_str = if rp_is_auto {
        format!("Auto({})", rp)
    } else {
        rp.to_string()
    };
    let local_cell_1 = format!("{:^width$}", local_rp_str, width = CELL_WIDTH);
    let local_cell_2 = format!("{:^width$}", latency.to_string(), width = CELL_WIDTH);
    let local_line_str = format!("You: {} | {}", local_cell_1, local_cell_2);

    let banner_display_str = format!("{}{}\0", local_line_str, station_line_str);

    let Some(banner_pane1_bg) = (*banner_pane1_ptr)
        .parent()
        .and_then(|pane| pane.traverse_backward(2))
    else {
        return;
    };
    banner_pane1_bg.set_visible(false);

    let Some(banner_pane2) = banner_pane1_bg
        .traverse_upward(2)
        .and_then(|pane| pane.prev())
    else {
        return;
    };
    let banner_pane2_ptr = banner_pane2 as *mut Pane;
    for pane_ptr in [banner_pane1_ptr, banner_pane2_ptr] {
        let pane = &mut *pane_ptr;
        let pane_tb = pane.as_textbox();

        pane_tb.set_text_string(&banner_display_str);

        pane_tb.font_size_x = 25.48;
        pane_tb.font_size_y = 51.0;
        pane_tb.line_space = -8.0;
        pane_tb.char_space = 1.0;

        pane_tb.set_default_material_colors();
        pane_tb.set_color(r, g, b, a);
        pane_tb.set_text_alignment(HorizontalPosition::Center, VerticalPosition::Center);

        pane_tb.bits |= 1 << TextBoxFlag::IsPTDirty as u16;
    }
}

pub unsafe fn update_local_online_ui(pane_handle: *mut Pane) {
    if pane_handle.is_null() {
        return;
    }

    if !crate::ui::overlay::is_window_interactable() {
        let input_snapshot = POLLER.snapshot(&NATIVE_UI_POLLER_CONTEXT);
        LATENCY_SLIDER_MANAGER.poll(
            &input_snapshot,
            ninput::Buttons::LEFT,
            ninput::Buttons::RIGHT,
        );
    }
    let delay_str = LATENCY_SLIDER_MANAGER.selected_latency().to_string();
    (*pane_handle)
        .as_textbox()
        .set_text_string(&format!("{}", delay_str));
}

pub unsafe fn update_online_arena_ui(pane_handle: *mut Pane, room_id: String) {
    if pane_handle.is_null() {
        return;
    }

    if !crate::ui::overlay::is_window_interactable() {
        let input_snapshot = POLLER.snapshot(&NATIVE_UI_POLLER_CONTEXT);
        LATENCY_SLIDER_MANAGER.poll(
            &input_snapshot,
            ninput::Buttons::LEFT,
            ninput::Buttons::RIGHT,
        );
        RENDER_PROFILE_MANAGER.poll(&input_snapshot, ninput::Buttons::DOWN, ninput::Buttons::UP);
    }

    let mut station_lines = String::new();
    let stations = StationConnectionManager::get_connected_stations();
    for station in stations.iter() {
        let station_ping = station.get_rtt();
        let station_latency = station.get_latency();
        let station_rp = station.get_render_profile();
        let station_latency_str = station_latency
            .map(|l| l.to_string())
            .unwrap_or_else(|| String::from("---"));
        let station_ping_str = station_ping
            .map(|rtt| format!("{}ms", rtt))
            .unwrap_or_else(|| String::from("---"));
        let station_rp_str = station_rp
            .map(|p| p.to_string())
            .unwrap_or_else(|| String::from("---"));
        let opp_name = station.get_main_account_name().unwrap_or_else(|| "???");
        let line = format!(
            "\n{}: {} | {} ({})",
            opp_name, station_rp_str, station_latency_str, station_ping_str
        );
        station_lines.push_str(line.as_str());
    }

    let latency = LATENCY_SLIDER_MANAGER.selected_latency();
    let rp = RENDER_PROFILE_MANAGER.selected_render_profile();
    let rp_is_auto = RENDER_PROFILE_MANAGER.is_auto_mode();
    let local_rp_str = if rp_is_auto {
        format!("Auto({})", rp)
    } else {
        rp.to_string()
    };
    let local_line_str = format!("\nYou: {} | {}", local_rp_str, latency.to_string());

    let room_id_line = format!("ROOM ID: {}", room_id);

    let (r, g, b, a) = match StationConnectionManager::get_network_stability() {
        NetworkStability::Stable => (0, 255, 0, 255),
        NetworkStability::Inconsistent => (255, 255, 0, 255),
        NetworkStability::Unstable => (255, 0, 0, 255),
    };

    let tb = (*pane_handle).as_textbox();
    tb.pos_y = -115.0 - (22.5 * stations.len() as f32);
    tb.flags |= 1 << PaneFlag::IsGlobalMatrixDirty as u8;
    tb.bits |= 1 << TextBoxFlag::IsPTDirty as u8;
    tb.set_default_material_colors();
    tb.set_color(r, g, b, a);
    tb.set_text_string(&format!(
        "{}{}{}",
        room_id_line, local_line_str, station_lines
    ));
}

//unsafe fn set_arena_msg_pane_text(msg_pane_root: &mut Pane, italics: bool, msg: &str) {
//    let r = msg_pane_root.children().unwrap();
//    let normal_tb = r.children().unwrap();
//    normal_tb.set_visible(!italics);
//    let italics_tb = normal_tb.next().unwrap();
//    italics_tb.set_visible(italics);
//
//    let tb = match italics {
//        true => italics_tb,
//        false => normal_tb,
//    };
//    let tb = tb.as_textbox();
//    tb.set_visible(true);
//    tb.set_influenced_alpha(false);
//    tb.alpha = 255;
//    tb.global_alpha = 255;
//    tb.set_default_material_colors();
//    tb.set_color(255, 255, 255, 255);
//    tb.set_text_string(msg);
//}
//
//unsafe fn handle_arena_layout(root_pane: &Pane) {
//    let r = root_pane.find_child("set_parts_logw", true).unwrap();
//    let r = r.children().unwrap();
//    let r = r.children().unwrap();
//    let r = r.traverse_forward(3).unwrap();
//    let new_msg_pane_root = r.children().unwrap();
//    let curr_msg_pane_holder = new_msg_pane_root.next().unwrap();
//    let old_msg_pane_holder = curr_msg_pane_holder.next().unwrap();
//
//    for holder in [&curr_msg_pane_holder, &old_msg_pane_holder] {
//        let mut current = holder.children();
//        while let Some(msg_pane_root) = current {
//            set_arena_msg_pane_text(msg_pane_root, true, "TEST 01");
//            current = msg_pane_root.next();
//        }
//    }
//    set_arena_msg_pane_text(new_msg_pane_root, false, "TEST NEW");
//}
//
//#[skyline::hook(offset = 0x4b640, inline)]
//unsafe fn on_draw_ui2d(ctx: &InlineCtx) {
//    if !is_online() || is_in_game() {
//        return;
//    }
//    let layout = ctx.registers[0].x() as *mut Layout;
//    let layout_name = skyline::from_c_str((*layout).layout_name);
//    let root_pane = (*layout).root_pane;
//    let root_pane = &*root_pane;
//
//    if layout_name == "online_buddy" {
//        handle_arena_layout(root_pane);
//    }
//}
//
//pub fn install() {
//    skyline::install_hook!(on_draw_ui2d);
//}
