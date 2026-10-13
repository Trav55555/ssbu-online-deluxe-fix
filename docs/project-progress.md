# Project progress

Last updated: 2026-10-13

## SSBU 13.0.5 candidate

The local development target is **v1.1.0 for SSBU 13.0.5 only**. No new release has been published. The existing v1.0.1 tag and assets remain unchanged for 13.0.4.

Completed locally:

- Backported upstream's 13.0.5 main-menu and native-text offsets, corrected latency hook, arena-menu hook, and later stage_config fix with the stage base in register x24.
- Added a game-version check before installing Deluxe hooks. `SSBU_VERSION` controls the compiled check and package naming.
- Updated and locked the Rust interfaces to the upstream v1.4.3 revisions. Runtime prerequisites are recorded in `release/runtime-dependencies.json`.
- Replaced the old render-loop hook with immediate SsbuSync API calls and batched environment flags. Active-profile reporting now reads runtime getters instead of requested-settings caches.
- Added v3 peer-metadata receive compatibility while retaining v2 emission for older fork builds. FPS++ metadata is not supported.
- Added singles/doubles/training match states, forced per-stage initialization, and actual fighter-count Auto selection.
- Added first-NRO-load rendering initialization and per-fighter dynamic-resolution resets. Critical-hit and Gigaflare behavior follow upstream v1.4.3; emulator callbacks remain disabled.
- Preserved the fork's Quickplay/Elite mode detection and emulator perf-scaler skip.
- Removed unused custom-font initialization. The overlay uses imgui's default font.
- Added candidate packaging and tests for version labels, checksums, wrong-version/truncated binaries, failed builds, reruns, and preservation of existing assets.

Checks passed locally:

```sh
cargo fmt -- --check
RUSTC_BOOTSTRAP=1 cargo check --lib --locked
python3 scripts/run_host_tests.py
PATH="$HOME/.cargo/bin:$PATH" rustup run skyline-v3 cargo skyline build --release -- --locked
python3 -m unittest discover -s scripts -p 'test_*.py' -v
```

The system Homebrew Rust cannot build the Skyline standard library. The installed Skyline-patched `skyline-v3` toolchain builds successfully. Existing dead-code warnings and the binrw future-compatibility warning remain.

The revised candidate is under `dist/readiness-2/`; the earlier candidate is preserved. Host tests exercise production packet/match-state logic and the render manager with simulated dependency APIs. They do not validate the NRO callback, fighter hooks, real SsbuSync, or live peer interoperability.

No 13.0.5 runtime evidence is available yet. The local Eden setup is still on 13.0.4 and was not changed. The updated SsbuSync runtime restricts official Elite rendering optimizations, so preserving this fork's mode detection is not proof of working Elite controls. See [version support](version-support.md) for installation requirements and runtime/publication gates.

## Published 13.0.4 release

The fork is published as **SSBU Online Deluxe Fix**.

- Main branch: `main`
- Current public release: `v1.0.1`
- Release URL: <https://github.com/Trav55555/ssbu-online-deluxe-fix/releases/tag/v1.0.1>
- Release commit/tag: `80c2dd5`
- Packaged asset names:
  - `ssbu-online-deluxe-fix-v1.0.1.zip`
  - `ssbu-online-deluxe-fix-v1.0.1.zip.sha256`

The plugin binary is still named `libssbu_online_deluxe.nro` so the install path and Skyline plugin name stay compatible with the existing project layout.

## Completed work

- Prepared the fork for public sharing.
- Added repository license metadata in `Cargo.toml`.
- Added `THIRD_PARTY_NOTICES.md`.
- Rewrote the README in a more direct project style.
- Added a fork note crediting the original `saad-script/ssbu-online-deluxe` project.
- Updated `.gitignore` so local game dumps, archives, release zips, build outputs, tools, and local validation notes stay out of git.
- Moved local SSBU update dumps out of the repo.
- Built and packaged `v1.0.1` with the `deluxe-fix` release naming.
- Synced upstream's stage presetup hook offset fix for `stage_config` mod compatibility.
- Restored the upstream overlay font path after the temporary font-removal experiment.
- Updated the release notes to say Quickplay/Elite features are not tested yet.

## Validation so far

Build checks run successfully:

```sh
cargo fmt -- --check
RUSTC_BOOTSTRAP=1 cargo check --lib
RUSTC_BOOTSTRAP=1 cargo skyline build --release
```

Runtime smoke test on Eden with SSBU `13.0.4` reached the main menu and loaded the plugin:

```text
Booting game ... Super Smash Bros. Ultimate (64-bit) | 13.0.4
[PluginManager] Loaded libssbu_online_deluxe.nro
[ssbu_online_deluxe] ssbu-online-deluxe initialized
[ssbu_online_deluxe] PERF SCALER SKIP: emulator detected
MAIN MENU INIT
```

The current tested NRO hash was:

```text
f6dcebe03f0a7d79ff6424d01469d021d9094150057520287ba25296e2f89d6e
```

## Known gaps

- Quickplay/Elite features have **not** been tested yet.
- Eden cannot validate Nintendo official Online matchmaking, so Quickplay/Elite needs console validation.
- Real online-match behavior still needs testing for mode detection, ping display, latency controls, render profile switching, and match start/end transitions.
- The `stage_config` compatibility fix is synced from upstream, but the full mod combination should still be tested together.
- DLC/unlock coverage was not installed in the Eden test environment.
- Binary release compliance needs another pass, especially around:
  - `lib/libimgui_smash.a` / `imgui-smash` licensing,
  - git dependencies with missing Cargo license metadata,
  - `assets/fonts/default_font.otf`, which identifies as a Fontworks/Nintendo font.

## Files intentionally not included

Do not commit or package:

- game dumps,
- update/DLC packages,
- keys,
- saves,
- emulator runtime files,
- Eden start/stop scripts,
- local model/navigation tools.

## Next steps for the 13.0.5 candidate

1. Run the candidate with matching dependency plugins on a legal SSBU 13.0.5 test setup.
2. Validate console Arenas, Local Online, and official Quickplay/Elite transitions. Do not infer runtime settings from the selection UI alone.
3. Test the version refusal and `stage_config` compatibility.
4. Resolve remaining static-library license and corresponding-source questions before publication.
5. Commit, tag, and publish only after approval and the documented release gates.
