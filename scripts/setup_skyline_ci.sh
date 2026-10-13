#!/usr/bin/env bash
# CI-only setup. Never run cargo skyline update-std against a mutable branch.
set -euo pipefail

: "${RUNNER_TEMP:?This installer requires a disposable GitHub Actions RUNNER_TEMP}"
: "${GITHUB_ACTIONS:?This installer is for GitHub Actions only}"
[[ "$GITHUB_ACTIONS" == "true" ]] || { echo "GitHub Actions only" >&2; exit 1; }
[[ "$(uname -sm)" == "Linux x86_64" ]] || { echo "Linux x86_64 required" >&2; exit 1; }

export PATH="$HOME/.cargo/bin:$PATH"
base=nightly-2026-02-14
std_revision=fabcf8e5bebc0d31e33717778a89df97d2b0e443
rustup toolchain install "$base" --profile minimal
root=$(mktemp -d "$RUNNER_TEMP/ssbu-skyline.XXXXXX")
sysroot=$(rustup run "$base" rustc --print sysroot)
cp -a "$sysroot/." "$root/"
source_dir="$root/lib/rustlib/src/rust"
# Minimal rustup installation has no rust-src component.
[[ ! -e "$source_dir" ]] || { echo "Unexpected pre-existing rust-src" >&2; exit 1; }
mkdir -p "$source_dir"
git -C "$source_dir" init -q
git -C "$source_dir" remote add origin https://github.com/skyline-rs/rust-src
git -C "$source_dir" fetch --depth 1 origin "$std_revision"
git -C "$source_dir" checkout --detach FETCH_HEAD
[[ "$(git -C "$source_dir" rev-parse HEAD)" == "$std_revision" ]]
# build-std needs backtrace, not LLVM or compiler documentation submodules.
git -C "$source_dir" submodule update --init --depth 1 library/backtrace
rustup toolchain link skyline-v3 "$root"
rustup run skyline-v3 rustc -Vv
rustup run "$base" cargo install cargo-skyline --version 3.5.0 --locked
