# Testing the 13.0.5 candidate

The harness currently provides CI checks, read-only runtime preflight, and opt-in diagnostic events. It does not launch Eden, drive matches, or certify game compatibility. These instructions require a source checkout.

## Run host checks

```sh
cargo fmt -- --check
RUSTC_BOOTSTRAP=1 cargo check --lib --locked
RUSTC_BOOTSTRAP=1 cargo check --lib --locked --features runtime_trace
python3 scripts/run_host_tests.py
python3 -m unittest discover -s scripts -p 'test_*.py' -v
```

Host tests use the production packet codec, match-state model, and event encoder. Render-manager tests simulate the external APIs. These tests cannot validate actual FFI, hook addresses, dependency loading, rendering, or peer transport.

`.github/workflows/checks.yml` runs these checks with Rust 1.96.0 and Python 3.11. A separate job builds normal and diagnostic plugins. Its installer pins cargo-skyline 3.5.0, nightly-2026-02-14, and Skyline's patched Rust source at `fabcf8e5bebc0d31e33717778a89df97d2b0e443`. Only the required backtrace submodule is initialized. The installer refuses non-CI execution and never calls the mutable `update-std` command.

Actions use immutable revisions and read-only permissions. CI does not upload NROs, publish releases, access consoles, or include game assets. A local check does not prove that the GitHub workflow has run.

## Build diagnostic instrumentation

Normal builds omit structured diagnostics. Build an instrumented NRO with:

```sh
PATH="$HOME/.cargo/bin:$PATH" rustup run skyline-v3 cargo skyline build --release -- --locked --features runtime_trace
```

The output is `target/aarch64-skyline-switch/release/libssbu_online_deluxe.nro`. Keep this diagnostic binary separate from normal release candidates. Subsequent builds replace the target output. Diagnostics can affect performance; do not use them to make latency or frame-time claims.

Each diagnostic record starts with `DELUXE_EVENT ` followed by one JSON object:

```json
{"schema":1,"sequence":7,"event":"render_apply","data":{"requested_bits":10246,"resolution_result":false,"flags":null}}
```

This abbreviated example distinguishes a rejected request from an unavailable getter. Sequence numbers are local to the process. Concurrent writers can deliver records out of sequence.

Events cover boot/version refusal, dependency API availability, first NRO load, hook installation, scene/stage/match transitions, rendering application and raw state, peer connections/metadata, and dynamic-resolution push/pop/reset responses. Peer IDs and player names are not logged. Identical consecutive peer metadata is suppressed per station.

- `null` means unavailable, unrecognized, or unqueried. It is not false, zero, or vanilla.
- Rendering state records use raw getters, not the UI's conservative fallback.
- `flags_result` is the API's returned flags value, not an acceptance boolean. Check the separate observed `flags` field.
- Wire-valid peer metadata means a supported version and packet layout. It does not prove every render bit is meaningful.
- A successful reset response does not independently prove that the runtime's report count is zero.
- A scene-hook event does not prove the screen finished rendering.

Records use the existing Skyline/stdout and plugin file logger. Keep logs private until checked for sensitive information. API responses and event ordering need game-process validation.

## Prepare an isolated runtime profile

1. Create a dedicated Eden profile outside this repository and the default Eden directories. Do not reuse or modify the existing 13.0.4 installation.
2. Configure that instance to use legal game/update dumps and SSBU 13.0.5. The preflight cannot inspect installed updates or prove ownership.
3. Install the matching prerequisites from [version support](version-support.md). Stage the diagnostic NRO and runtime files inside the dedicated profile.
4. Copy `tests/runtime/preflight.example.json` to an ignored local configuration, such as `.pi/runtime.json`.
5. Replace every path with an absolute path and every checksum with an independently recorded SHA256. Example paths do not prescribe Eden's mod layout. The Skyline roles refer to ExeFS `subsdk9` and `main.npdm`.
6. Run preflight into a new evidence directory:

```sh
python3 scripts/runtime_preflight.py --config .pi/runtime.json --output-dir dist/test-runs/preflight-001
```

Preflight only reads the configuration and test files. It checks profile isolation, file presence, configured checksums, NRO headers, the plugin's version guard, and its diagnostic marker. Runtime files are limited to 64 MiB each. Missing files/checksums, wrong declared versions, and non-diagnostic builds block preparation. Hash mismatches and profile escapes fail it.

Exit codes are 0 for passed preparation, 1 for failure, and 2 for blocked preparation. A pass does not prove launchability or game compatibility. The results always mark runtime validation blocked.

Each invocation writes `run.json`, `results.json`, and an empty `events.jsonl`. Existing evidence directories are never overwritten. Evidence cannot be written into the inspected profile or default Eden directories. No game starts and no installation files change.

## Next runtime milestone

Add a bounded runner for one Local Wireless room → match → rematch → exit scenario. It must own its process, wait for observed states with deadlines, reject missing evidence, and preserve logs/screenshots alongside plugin and dependency hashes. Do not count navigation suggestions or absence of a crash as a passing assertion.

Official Quickplay/Elite checks remain operator-run on a console. Eden cannot test them and skips performance-scaler callbacks. Console checks must cover critical-hit/Gigaflare behavior, multiple fighters, and truthful reporting under Elite's vanilla restrictions. Runtime and licensing gates in [version support](version-support.md) remain open.
