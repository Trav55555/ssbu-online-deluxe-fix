#![allow(static_mut_refs)]

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

#[skyline::main(name = "ssbu-online-deluxe")]
pub fn main() {
    setup_panic_hook();
    let _ = logging::init(log::LevelFilter::Info);
    logging::info!("ssbu-online-deluxe initialized");
    logging::info!(
        "TEXT REGION BASE: {:#x} emulator={}",
        utils::text_region_base(),
        utils::is_emulator()
    );

    render::install();
    net::install();
    ui::install();

    if utils::is_emulator() {
        logging::info!("PERF SCALER SKIP: emulator detected");
    } else {
        perf_scaler::install();
    }
}
