# Project progress

Last updated: 2026-06-15

## Current state

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

## Recommended next steps

1. Test on console with SSBU `13.0.4` and official Online access.
2. Validate Quickplay and Elite Smash entry, in-match behavior, and exit cleanup.
3. Re-test Arenas and Local Online after the Quickplay changes.
4. Test alongside `stage_config` to confirm the hook conflict is actually resolved in practice.
5. Do a final license/release audit before promoting the binary more widely.
6. Replace or remove the bundled font if redistribution rights are unclear.
