#![allow(static_mut_refs)]

use std::sync::atomic::{AtomicBool, Ordering};

mod input_poll;
mod logging;
mod net;
mod perf_scaler;
mod render;
mod ui;
mod utils;

fn setup_panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        let location = info
            .location()
            .map(|location| location.to_string())
            .unwrap_or_else(|| String::from("unknown location"));

        let msg = match info.payload().downcast_ref::<&'static str>() {
            Some(s) => *s,
            None => match info.payload().downcast_ref::<String>() {
                Some(s) => &s[..],
                None => "Box<Any>",
            },
        };

        let err_msg = format!("thread has panicked at '{}', {}\0", msg, location);
        skyline::error::show_error(
            69,
            "Skyline plugin has panicked! Please open the details and send a screenshot to the developer, then close the game.\0",
            err_msg.as_str()
        );
    }));
}

fn on_nro_load(_info: &skyline::nro::NroInfo) {
    static NRO_LOADED: AtomicBool = AtomicBool::new(false);
    if !NRO_LOADED.swap(true, Ordering::SeqCst) {
        logging::event!("nro_loaded");
        render::on_nro_load();
        logging::event!("dependency_apis", "ssbusync" => ultelier::sync_guest::api_version());
    }
}

#[skyline::main(name = "ssbu-online-deluxe")]
pub fn main() {
    setup_panic_hook();
    let _ = logging::init(log::LevelFilter::Info);
    let game_version = skyline::nn::oe::get_display_version();
    logging::event!("boot",
        "game_version" => game_version.as_str(),
        "required_version" => env!("SSBU_VERSION"),
        "mod_version" => env!("CARGO_PKG_VERSION"),
        "emulator" => utils::is_emulator(),
    );
    if game_version != env!("SSBU_VERSION") {
        logging::event!("version_refused", "game_version" => game_version.as_str());
        let detail = format!(
            "This build requires SSBU {}. Detected {}. Install the release matching your game version. No Deluxe hooks were installed.\0",
            env!("SSBU_VERSION"), game_version
        );
        logging::info!("Unsupported SSBU version: {}", game_version);
        skyline::error::show_error(71, "Unsupported SSBU version!\0", &detail);
        return;
    }
    logging::info!(
        "ssbu-online-deluxe v{} initialized for SSBU {}",
        env!("CARGO_PKG_VERSION"),
        game_version
    );
    logging::info!(
        "TEXT REGION BASE: {:#x} emulator={}",
        utils::text_region_base(),
        utils::is_emulator()
    );

    skyline::nro::add_hook(on_nro_load)
        .expect("Unable to add NRO load hook! Make sure nro hook plugin is present.");

    logging::event!("dependency_apis", "ssbusync" => ultelier::sync_guest::api_version());
    render::install();
    net::install();
    ui::install();

    if utils::is_emulator() {
        logging::info!("PERF SCALER SKIP: emulator detected");
    } else {
        perf_scaler::install();
    }
    logging::event!("hooks_installed", "perf_scaler" => !utils::is_emulator());
}
