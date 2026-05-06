# Batch 7 Desktop UI Modularization Prep - 2026-05-06

## UI / UX

- No visible Desktop UI behavior changed.
- Started reducing the `src/bin/ui.rs` monolith by extracting leaf utilities
  that do not own UI state, config serialization, rendering flow, or proxy
  runtime lifecycle.

## Config / Schema

- No config schema changed.
- `ConfigWire`, config registry, Android config, examples, and parity matrices
  were not changed.

## Backend Helpers

- No backend helper behavior changed.

## Security / Trust

- No trust/security behavior changed.
- File/resource opening behavior was moved as-is into `src/bin/ui_fs.rs`.

## Breaking / Cleanup

- Moved formatting helpers from `src/bin/ui.rs` to `src/bin/ui_format.rs`.
- Moved Desktop file/resource-opening helpers from `src/bin/ui.rs` to
  `src/bin/ui_fs.rs`.
- Added focused unit tests for duration/byte formatting and resource resolver
  behavior.
- Removed the old helper definitions from `src/bin/ui.rs`.

## Parity

- Added `tools/check-desktop-ui-modularization.py`.
- Wired the guard into:
  - `tools/run-repo-sanity.py`;
  - `tools/check-ci-local-sanity-parity.py`.
- Documented the guard in `tools/README.md`.
- Added the new Desktop UI helper boundaries to `docs/tooling-source-map.json`
  and `docs/tooling-source-map.md`.

## Race / Split-Brain Review

- Extraction was limited to pure/leaf helpers:
  - `fmt_duration`;
  - `fmt_bytes`;
  - `downloads_dir`;
  - `reveal_in_file_manager`;
  - `open_local_resource`.
- No background thread ownership, shared state, async runtime, config save/load,
  Doctor/status state, or proxy lifecycle logic moved.

## Verification

- `cargo fmt --check`
- `cargo check --features ui --bin mhrv-f-ui`
- `python tools/check-desktop-ui-modularization.py`
- `python tools/check-desktop-doctor-summary.py`
- `python tools/check-desktop-test-relay-mode-guard.py`
- `python tools/check-tooling-source-map.py`
- `python tools/check-ci-local-sanity-parity.py`

Full repo-sanity and cleanup were run as part of batch closeout.
