# Batch 7 Desktop UI Doctor Summary Extraction - 2026-05-06

## Summary

- Extracted the Desktop Monitor Doctor summary card from `src/bin/ui.rs` into
  `src/bin/ui_doctor.rs`.
- Moved the Doctor level-label helper into the same module so background Doctor
  logging and Monitor rendering use one label vocabulary.
- Kept Doctor summary behavior unchanged: empty state, OK / warning / failure
  counts, timestamp age, prioritized non-OK items, fix text, and overflow count
  remain the same.
- Added focused unit tests for stable Doctor labels, level counting, and status
  token colors.

## Parity

- Desktop keeps consuming the typed Rust `DoctorReport` directly while Android
  continues to consume the guarded Doctor JSON contract through JNI.
- The extraction does not change backend diagnostics, support-bundle Doctor
  JSON, Android Doctor UI, readiness behavior, config schema, or examples.
- `docs/tooling-source-map.json`, `docs/tooling-source-map.md`, and
  `tools/README.md` now name `src/bin/ui_doctor.rs` as the guarded source for
  this Desktop UI boundary.

## Cleanup

- Removed the inline Doctor summary helpers from `src/bin/ui.rs`.
- Extended `tools/check-desktop-ui-modularization.py` so future edits cannot
  reintroduce Doctor summary rendering, Doctor counts, or Doctor color helpers
  into the monolith.
- Updated `tools/check-desktop-doctor-summary.py` so the older Doctor summary
  guard verifies the extracted renderer in `src/bin/ui_doctor.rs` instead of
  requiring the helpers to stay inline in `src/bin/ui.rs`.
- Kept the new module narrowly scoped to Doctor summary rendering only; no
  backend, Android, or support-bundle logic was duplicated.

## Verification

- `cargo fmt --check`
- `cargo check --features ui --bin mhrv-f-ui`
- `cargo test --features ui --bin mhrv-f-ui ui_doctor`
- `python tools/check-desktop-ui-modularization.py`
- `python tools/check-tooling-source-map.py`
- `python tools/check-doc-links.py`
- `python tools/check-doc-anchors.py`
- `python tools/check-changelog-headings.py`
- `python tools/generate-changelog-index.py -Check`
- `python tools/check-ci-local-sanity-parity.py`
- `python tools/run-repo-sanity.py --skip-node`
- `python tools/check-repo-cleanliness.py`
