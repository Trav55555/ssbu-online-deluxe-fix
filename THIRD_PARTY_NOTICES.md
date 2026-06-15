# Third-party notices

This project depends on several Rust crates and Smash modding libraries. This file is a best-effort notice list; check upstream repositories before publishing a release.

## Project license

This repository includes `LICENSE`, currently GNU Affero General Public License v3.0.

## Included source snippets

- `src/render/tests/mod.rs` contains test functions adapted from `alexheretic/spin-sleep` experiments and is noted in the file as Apache-2.0.

## Runtime/build dependencies

Cargo dependencies include crates under MIT, Apache-2.0, BSD-style, Zlib, and Unlicense/MIT licenses. The git dependencies used by Smash modding projects do not all declare Cargo license metadata.

Notable dependencies to verify before release:

- `skyline`, `skyline_macro`, `skyline_smash`, `nnsdk`: MIT in Cargo metadata.
- `imgui-smash` / `libimgui_smash.a`: upstream host library includes a GPL-2.0 license file. If distributing a binary linked with this library, verify compatibility with this project's AGPL-3.0 license and provide the required source/notices.
- `imgui-api`, `ninput`, `smashline`, `ssbu-pia-interface`, `smash-ultelier`: no license metadata was found in Cargo metadata at the revisions currently used. Verify upstream license/permission before publishing release binaries.

## Assets

Do not commit Nintendo game dumps, update/DLC packages, keys, saves, or proprietary game assets.

`assets/fonts/default_font.otf` identifies itself as `nintendoP_RodinNTLG-B003` with Fontworks copyright metadata. Verify redistribution rights before publishing source or binary packages that include or embed it.
