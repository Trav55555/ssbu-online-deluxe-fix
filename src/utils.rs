use skyline::nn::os::Tick;
use std::time::Duration;

extern "C" {
    #[link_name = "\u{1}_ZN2nn2os22GetSystemTickFrequencyEv"]
    fn get_tick_freq_internal() -> u64;
}

#[inline]
pub fn get_tick_frequency() -> u64 {
    unsafe { get_tick_freq_internal() }
}

#[inline]
pub fn duration_since_tick(tick: Tick) -> Duration {
    unsafe {
        let elapsed_ticks = skyline::nn::os::GetSystemTick() - tick;
        Duration::from_secs_f64(elapsed_ticks as f64 / get_tick_freq_internal() as f64)
    }
}

#[inline]
pub fn text_region_base() -> u64 {
    unsafe { skyline::hooks::getRegionAddress(skyline::hooks::Region::Text) as u64 }
}

#[inline]
pub fn is_emulator() -> bool {
    let base_address = text_region_base();

    // Real Switch processes use high 0x71... text mappings. Eden/Yuzu-style
    // emulators map NSO text in low 32-bit-ish address space, and the exact
    // base changes with game/update/loader state (for example SSBU 13.0.3 in
    // Eden maps at 0x80bde000). Use the address range instead of pinning one
    // update-specific base.
    base_address < 0x1_0000_0000
}

#[inline]
pub fn lookup_symbol_addr(symbol: &'static [u8]) -> Option<usize> {
    let mut addr = 0usize;
    unsafe {
        if skyline::nn::ro::LookupSymbol(&mut addr, symbol.as_ptr()) == 0 && addr != 0 {
            Some(addr)
        } else {
            None
        }
    }
}
