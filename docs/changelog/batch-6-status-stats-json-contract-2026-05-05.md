# Batch 63 - Status/Stats JSON Contract

Date: 2026-05-05

## What Changed

- Added `status_api::stats_snapshot_json_value`, a shared renderer for
  `StatsSnapshot` JSON.
- Updated local `/status` rendering to use the shared stats renderer.
- Updated Android JNI `Native.statsJson(handle)` to call the same shared
  renderer instead of hand-maintaining its own stats field list.
- Preserved Android's legacy alias keys (`total_scripts`,
  `blacklisted_scripts`) while keeping canonical status keys
  (`scripts_total`, `scripts_blacklisted`) in the same object.
- Added focused Rust regression tests for canonical keys, Android aliases,
  degradation reason trimming, and `/status` envelope rendering.
- Added `tools/check-status-stats-json-contract.py` and wired it into
  repo-sanity so Android JNI cannot silently fork the stats schema again.
- Documented the status/stats JSON guard in `tools/README.md`.

## Parity / Cleanup

- Desktop, local `/status`, support-bundle snapshots, and Android JNI now share
  one stats JSON field source.
- Android frontend parsing remains compatible because the raw `statsJson`
  object still exposes `today_calls`, `today_bytes`, and `today_reset_secs`.
- No config schema, Apps Script, tunnel-node, Android UI layout, or backend
  routing behavior changed.
- No legacy/deprecated compatibility path was added; the only aliases retained
  are documented Android compatibility keys inside the single shared renderer.

## Verification

- `python tools/check-status-stats-json-contract.py`
- `cargo fmt --check`
- `cargo test status_api --lib`
- `python tools/check-ci-local-sanity-parity.py`

Full batch verification is recorded in `elevation_audit_roadmap_source.md`.
