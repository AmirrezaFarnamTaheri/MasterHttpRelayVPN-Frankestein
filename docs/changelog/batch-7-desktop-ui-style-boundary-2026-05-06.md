# Batch 7 Desktop UI Style Boundary - 2026-05-06

## Summary

- Added `src/bin/ui_style.rs` as the canonical Desktop UI style/helper module.
- Moved visual tokens, theme application, section frames, help text helpers,
  callout/goal cards, primary buttons, and compact form rows out of
  `src/bin/ui.rs`.
- Kept domain-specific renderers in `src/bin/ui.rs`; this batch only extracts
  shared visual primitives.
- Extended `tools/check-desktop-ui-modularization.py` so style primitives cannot
  drift back into the Desktop UI monolith.
- Updated tooling source-map documentation for the new Desktop style boundary.

## Parity

- Desktop behavior is intended to be unchanged: the same color values, spacing,
  shadows, theme settings, form-row layout, and primary button styling are now
  imported from `ui_style.rs`.
- Android, backend helpers, config schema, examples, readiness, Doctor/status,
  support bundles, runtime proxy logic, tunnel-node, and release workflow
  behavior are not changed by this slice.

## Cleanup

- Removed duplicate style constants and helper functions from `src/bin/ui.rs`.
- Left no second copy of the Desktop visual token palette.
- Kept the next extraction path clean: XHTTP rendering and other panels can now
  import style helpers from one shared module instead of depending on local
  `ui.rs` definitions.

## Verification

- `cargo fmt --check`
- `cargo check --features ui --bin mhrv-f-ui`
- `cargo test --features ui --bin mhrv-f-ui ui_style`
- `python tools/check-desktop-ui-modularization.py`
- `python tools/check-tooling-source-map.py`
- `python tools/check-doc-links.py`
- `python tools/check-doc-anchors.py`
- `python tools/check-changelog-headings.py`
- `python tools/generate-changelog-index.py -Check`
- `python tools/check-ci-local-sanity-parity.py`
- `python tools/run-repo-sanity.py --skip-node`
- `python tools/check-repo-cleanliness.py`
