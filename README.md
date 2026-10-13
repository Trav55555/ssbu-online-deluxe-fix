# SSBU Online Deluxe

SSBU Online Deluxe is a work-in-progress mod for **Super Smash Bros. Ultimate**. It adds online latency controls, render-profile controls, and an optional overlay for connection and frame-time info.

This is a fork of saad-script's original SSBU Online Deluxe work, with additional compatibility and validation changes.

Use it at your own risk. I have tested it online without major issues, but there is always some risk when using mods online. The overclocks are intentionally conservative, but hardware damage, crashes, bans, or other account issues are still your responsibility.

## Compatibility

The next release candidate, **v1.1.0**, targets **SSBU 13.0.5 only**. It is not published or runtime-validated yet. The published [v1.0.1 release](https://github.com/Trav55555/ssbu-online-deluxe-fix/releases/tag/v1.0.1) remains available for **SSBU 13.0.4**.

Use the release matching your installed game version. See [version support and validation gates](docs/version-support.md).

- Nintendo Switch hardware
- Eden emulator for startup and LDN smoke testing
- Other emulators are untested
- HDR is untested

Eden does **not** support Nintendo's official Online services, so it cannot validate Quickplay or Elite Smash matchmaking. Use console hardware for that testing.

## Installation

1. Install the [matching runtime prerequisites](docs/version-support.md#install-the-1305-candidate) for the 13.0.5 candidate. This fork's ZIP contains only the Deluxe plugin and documentation.
2. Remove any old latency-slider, vsync, or less-lag mods.
3. Download the release ZIP matching your SSBU version and extract it.
4. Copy the `atmosphere` folder to the root of your SD card, or to the emulator `sdmc` directory. Keep the prerequisite plugins installed.
5. Fully restart the Switch or emulator.

### Eden notes

Eden currently needs a little extra setup:

- In Eden, right-click SSBU and open `Configure Game`.
- Go to `System`.
- Enable `RNG Seed` and set it to `00000000`.
- Use a complete, known-good Skyline ExeFS set for SSBU: `exefs/subsdk9` and `exefs/main.npdm`.
- Use SSBU `13.0.5` for the v1.1.0 candidate, or `13.0.4` for v1.0.1. Base `1.0.0` is not supported.

On Eden, the dynamic-resolution perf-scaler hooks are skipped. Those Smashline fighter-frame hooks froze Eden during testing. Latency controls, render-profile controls, the overlay, and online-mode hooks still load.

Eden Local Wireless reaches Eden's LDN service path, but room and match validation are still unfinished.

If you are setting up SSBU on emulator, [ssbu-emu-optimizer](https://github.com/saad-script/ssbu-emu-optimizer/releases) may save time.

## Controls

### Native online UI

On the online character-select screen or in an online arena:

- `D-pad Left / Right`: change network latency
- `D-pad Up / Down`: change render profile

On character select with more than one opponent:

- `ZL + ZR + D-pad Left / Right`: choose which opponent's network info is shown

### Overlay

- `ZL + ZR + D-pad Down`: cycle the overlay
- `L + R + D-pad Down`: alternate overlay shortcut

Overlay modes:

1. Hidden
2. Full Info
3. Performance Info / frame view

While the overlay is in Full Info mode:

- `D-pad Up / Down`: select a row
- `D-pad Left / Right`: change the selected value

Hold `Plus` or `Minus` while toggling from Full Info to open the hidden Debug view.

On Eden's default keyboard mapping, `ZL + ZR + D-pad Down` is usually `R + T + Down Arrow`.

The overlay can be opened after the plugin loads. Latency and render settings can be selected on supported online pre-match screens: Online Arena and Local Online. The fork also retains experimental Quickplay/Elite paths, but their runtime behavior is unverified. During a match, the overlay is read-only.

## Features

### Online info

The mod shows opponent connection info in supported online modes, including:

- RTT / ping
- connection quality
- green/yellow/red stability coloring

If both players have the mod, it can also show extended opponent info:

- selected latency
- selected render profile

### Latency controls

Intended for Online Arena and Local Online pre-match screens. Quickplay/Elite latency behavior has not been console-tested.

Latency options:

- `Auto`: use SSBU's normal latency calculation
- `0f` through `25f`: force a manual delay-frame value

Manual latency is usually best set from the opponent's ping and stability.

### Render profiles

Intended for Online Arena and Local Online pre-match screens. The matching SsbuSync runtime restricts render optimizations in official Elite Smash; this fork's mode detection does not bypass that restriction.

Profiles:

- `Auto`: pick a recommended profile for the platform and opponent count
- `Vanilla`: stock render settings
- `LessLag`: cuts 3 frames of native input delay
- `LLUltra`: cuts 4 frames of native input delay
- `LLDoubles`: cuts 2 frames of native input delay; intended for doubles or heavier scenes

`LLUltra` also works on console, but it lowers the game resolution to avoid stutter. Some UI elements may look off, especially fighter cut-ins and the match-start countdown.

If you are unsure, leave the profile on `Auto`.

Suggested profiles:

- Console: `LessLag` or `LLUltra`
- Emulator: `LLUltra`
- Doubles: `LLDoubles`

The selected render profile is applied when entering a valid online match and cleaned up after leaving it.

## Testing

See [the testing harness](docs/testing.md) for CI checks, diagnostic builds, and isolated runtime preflight. Host checks do not prove game compatibility.

## Notes and contributing

The dynamic-resolution logic currently handles zoom-in moments, such as final-hit or critical-hit camera effects, and Sephiroth's Gigaflare.

Contributions are welcome, especially for moves that still cause stutter. Start with `src/perf_scaler` if you want to add another move-specific optimization.

## License

This fork follows the repository license: AGPL-3.0. If you distribute builds, include the license and provide the corresponding source for the build.

Do not include game files, update/DLC packages, keys, or saves in release packages.

## Credits

- **saad-script** — original SSBU Online Deluxe work.
- **Bludev** — SSBU render-system research and the original less-lag and latency-slider work.
- **BlankMauser** — SsbuSync and smash-ultelier. This mod uses that work to modify SSBU's render system.
- **Kinnay** and NintendoClients contributors — network-service documentation and implementation guidance.
- **Coolsonickirby** — imgui-smash, which makes the overlay possible.
- **HDR team** — smashline.
- **Skyline developers** — the Switch modding runtime used by this project.
