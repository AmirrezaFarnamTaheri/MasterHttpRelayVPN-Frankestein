# Batch 64 - Android Stats Envelope Tolerance

Date: 2026-05-05

## What Changed

- Added `statsPayloadFromJson` in Android `HomeScreen.kt`.
- The Usage Today card now accepts both:
  - raw `Native.statsJson(handle)` stats objects;
  - local `/status`-style envelopes where stats live under `stats`.
- Extended `tools/check-status-stats-json-contract.py` so repo-sanity verifies
  the Android Usage Today card uses the envelope-tolerant parser.

## Parity / Cleanup

- This keeps Android compatible with the current raw JNI stats shape while
  preparing it for any later move toward a fuller status envelope.
- Android copy inventory remains at 3 dynamic/technical tokens and 0 localize
  candidates.
- No Rust runtime stats fields, config schema, Apps Script, tunnel-node,
  desktop UI, or backend routing behavior changed.

## Verification

- `python tools/check-status-stats-json-contract.py`
- `python tools/generate-android-hardcoded-copy-inventory.py -Check`
- `cargo fmt --check`
- `python tools/check-ci-local-sanity-parity.py`

Full batch verification is recorded in `elevation_audit_roadmap_source.md`.
