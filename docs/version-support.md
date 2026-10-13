# Version-specific releases

Each release targets one SSBU version. `SSBU_VERSION` is the build's game-version authority. `build.rs` embeds it in the plugin, and the plugin checks the installed game version before installing Deluxe hooks. There is no multi-version hook selection.

| Mod release | Required SSBU version | Status |
| --- | --- | --- |
| v1.0.1 | 13.0.4 | Published legacy release; Eden startup tested |
| v1.1.0 | 13.0.5 | Local candidate; runtime testing pending |

The existing v1.0.1 tag and assets stay unchanged. New assets include both versions in their names:

```text
ssbu-online-deluxe-fix-v1.1.0-ssbu13.0.5.zip
ssbu-online-deluxe-fix-v1.1.0-ssbu13.0.5.zip.sha256
```

## Install the 13.0.5 candidate

This fork's ZIP contains only the Deluxe plugin and documentation. Install the matching runtime dependencies first. Do not mix the 13.0.4 installation with this candidate.

1. Confirm SSBU displays version **13.0.5**.
2. Back up your existing mod installation.
3. Install the runtime files from [upstream Deluxe v1.4.3](https://github.com/saad-script/ssbu-online-deluxe/releases/tag/v1.4.3). Its ZIP includes Skyline ExeFS, SsbuSync, and the overclock plugin/service. Use the bundled Skyline and SsbuSync rather than unrelated latest releases.
4. Install the additional prerequisites below. The upstream ZIP does not contain these plugins.
5. Remove conflicting standalone latency-slider, vsync, and less-lag mods.
6. Copy this fork's `atmosphere` folder to your SD root or emulator `sdmc`. Replace upstream's `libssbu_online_deluxe.nro`, keeping the prerequisite plugins installed.
7. Fully restart the Switch or emulator. Restart Atmosphere after installing the overclock service.

Additional prerequisites for this candidate:

| Plugin | Release |
| --- | --- |
| `libssbu_pia_manager.nro` | [PIA manager v1.3.0](https://github.com/project-ultelier/ssbu-pia-interface/releases/tag/v1.3.0) |
| `libimgui_smash.nro` | [imgui-smash v1.0.0](https://github.com/Coolsonickirby/imgui-smash/releases/tag/v1.0.0) |
| `libsmashline_plugin.nro` | [Smashline v1.6.7](https://github.com/HDR-Development/smashline/releases/tag/v1.6.7) |
| `libnro_hook.nro` | [nro-hook v0.4.0](https://github.com/ultimate-research/nro-hook-plugin/releases/tag/v0.4.0) |
| `libarcropolis.nro` | [ARCropolis v4.1.1](https://github.com/Raytwo/ARCropolis/releases/tag/v4.1.1) |

These are candidate prerequisites, not a runtime-tested combination. `release/runtime-dependencies.json` records download links and the inspected upstream/PIA hashes. Packaging copies this record into `release.json`.

The build interfaces expect SsbuSync API 2 and PIA manager API 2. Old runtime plugins are not interchangeable with the updated Rust interfaces.

## Build and package

Use Python 3.11 or newer, rustup, cargo-skyline, and the Skyline-patched `skyline-v3` toolchain. A normal Rust standard library does not support the Skyline target.

```sh
cargo fmt -- --check
RUSTC_BOOTSTRAP=1 cargo check --lib --locked
python3 scripts/run_host_tests.py
python3 -m unittest discover -s scripts -p 'test_*.py' -v
python3 scripts/package_release.py
```

The packaging command puts rustup's proxies ahead of system Rust in the build process's PATH, selects `skyline-v3`, and builds before reading the output. For a standard rustup installation, the equivalent manual command is:

```sh
PATH="$HOME/.cargo/bin:$PATH" rustup run skyline-v3 cargo skyline build --release -- --locked
```

Use `--toolchain NAME` for another Skyline-patched toolchain. Use `--output-dir PATH` to write a revised candidate without replacing an earlier artifact.

Packaging verifies the NRO header, declared size, and embedded version guard. It includes the plugin hash, Cargo.lock hash, resolved git dependencies, and source revision/dirty status in `release.json`. Identical reruns leave assets unchanged. A different artifact at the same path causes an error instead of overwriting it. Packages are always labelled as candidates; the script does not publish releases or certify runtime support.

## Compatibility changes

The backport follows [upstream v1.4.3](https://github.com/saad-script/ssbu-online-deluxe/tree/25f67c99468ff6b8fad56fb5107c959fdc09b087), rather than applying only the initial 13.0.5 patch.

| Location | 13.0.5 target | Reference |
| --- | --- | --- |
| `src/net/mod.rs`, main-menu hook | `0x235aaa0` | [Initial version update](https://github.com/saad-script/ssbu-online-deluxe/commit/f1b65915fb1155fb01cc02f891a2c2f82ebf21f0) |
| `src/net/mod.rs`, arena-menu hook | `0x22d9b50` | [Upstream hook cleanup](https://github.com/saad-script/ssbu-online-deluxe/commit/e5472d8da1985daa3df7e339d5b75f87ca6d5da7) |
| `src/net/mod.rs`, stage presetup | `0x25d9320`, stage base in `x24` | [Later stage_config fix](https://github.com/saad-script/ssbu-online-deluxe/commit/25f67c99468ff6b8fad56fb5107c959fdc09b087) |
| `src/net/latency_slider/mod.rs` | `0x16ccab8` | [Corrected latency hook](https://github.com/saad-script/ssbu-online-deluxe/commit/3cc959a1d924b611d7609b258ec4b33b2bfec4f3) |
| `src/ui/native/utils.rs`, text setter | `0x37a28a0` | Initial version update |

Other active network hook offsets match upstream v1.4.3. Following upstream's approach, this backport removes the old render main-loop hook. Settings now use SsbuSync's public API, with environment flags applied together. This avoids carrying an unaudited 13.0.4 render-loop offset into the new build.

`Cargo.lock` pins all 14 resolved git dependencies to upstream v1.4.3's revisions, including git `nnsdk` 0.6.1 at `ca3f1e3f`. The backport preserves the fork's Quickplay/Elite detection, conservative overclock settings, and emulator perf-scaler skip. The overlay uses imgui's default font; it no longer embeds the repository's unverified font asset.

### Release-readiness follow-up

- Active rendering and outgoing in-match metadata now read SsbuSync's runtime getters instead of a cache of requested settings. Unavailable getters fall back to conservative vanilla values. This does not prove the runtime is present or bypass Elite restrictions.
- The plugin still sends protocol v2 metadata so older fork builds can read it. It accepts upstream v3 metadata after removing v3's FPS-boost and reserved high bits. Those bits do not represent this fork's v2 overclock field. FPS++ information is not displayed. Malformed or unsupported packets invalidate the cached peer metadata.
- Match tracking distinguishes inactive, singles, doubles, and training/waiting-room stages. Each active stage forces initialization, including rematches. The actual fighter count selects the in-match Auto profile, allowing doubles with one peer console. Training and waiting stages reset active rendering without discarding the next-match selection.
- Rendering defaults apply again on the first NRO-load callback. The unsupported-version return precedes callback registration and hook installation.
- Dynamic-resolution reports and fighter latches reset at match initialization and cleanup. Critical-hit slowdowns and Gigaflare use per-fighter state for eight entries. The critical-hit hook `0x06a7100` and Gigaflare charge threshold `>= 3` follow upstream v1.4.3. These callbacks remain disabled on emulators.

`python3 scripts/run_host_tests.py` runs the production packet codec and match-state module directly. It also tests the production render manager with simulated SsbuSync and network APIs. It checks rejected requests, external runtime changes, unavailable getters, doubles selection, and training resets. It does not load Skyline or test real FFI, NRO events, fighter callbacks, peer connections, or official matchmaking. Those checks still require a game process.

The revised local artifact is packaged under `dist/readiness-2/`. The earlier candidate and legacy ZIPs are preserved. Rebuild it with:

```sh
python3 scripts/package_release.py --output-dir dist/readiness-2
```

## Runtime and publication gates

Compilation and packaging do not prove game compatibility. Before publishing a supported release, record the plugin/dependency hashes and complete these checks on 13.0.5:

- Eden startup reaches the main menu, loads all required plugins, and opens/closes the overlay. Verify Auto/manual latency displays and render-profile selections in Local Wireless. Complete a room/match/exit cycle with a second peer.
- Console Arenas show ping, accept pre-match latency/profile selections, start/end matches, and restore vanilla rendering on exit.
- Official Quickplay/Elite enters matchmaking, starts a match, handles rematches, and cleans up on exit. Check actual runtime settings, not just selected values shown by this fork's UI.
- Repeat Arena/Local Wireless transitions and test with `stage_config`. HDR is not claimed supported by this fork merely because the hook fix came from upstream.
- On a separate compatible test setup, verify the version guard refuses a non-13.0.5 game before Deluxe hook installation. This does not make other dependency plugins safe on unsupported games.

Eden cannot test Nintendo's official matchmaking. The upstream SsbuSync binary contains an Elite-mode restriction that forces vanilla rendering outside its supported modes. Retaining this fork's Elite detection does not bypass that restriction. Do not advertise official Elite render controls as working. Latency and transition behavior remain unverified.

Before publication, also resolve the static `libimgui_smash.a` license compatibility and corresponding-source obligations described in `THIRD_PARTY_NOTICES.md`. Do not publish an uncommitted candidate as if its source matched an existing tag. Commit and tag approved source, regenerate its candidate, and review the evidence before creating a GitHub release.
