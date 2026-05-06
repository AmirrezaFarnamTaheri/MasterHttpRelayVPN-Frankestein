# Batch 7 Desktop UI Help Walkthrough Extraction - 2026-05-07

## Summary

- Moved the full Desktop **Help & walkthrough** renderer from `src/bin/ui.rs`
  into `src/bin/ui_help.rs`.
- Kept the visible Help content unchanged: first-time checklist, Trust Center
  preview, backend registry link, mode goal cards, mode requirements,
  split-brain warnings, advanced tuning notes, privacy/trust notes, Android
  companion note, and maintainer repository link.
- Left `src/bin/ui.rs` as the caller for the Help tab while `src/bin/ui_help.rs`
  now owns both walkthrough content and backend-tool catalog rows.

## Parity

- This is a Desktop presentation-boundary extraction only. Android Help,
  backend helpers, config schema, runtime behavior, examples, and docs targets
  are unchanged.
- The Help walkthrough still consumes the same `FormState`, Trust Center
  snapshot renderer, style helpers, and product/repository constants.
- `docs/tooling-source-map.json`, `docs/tooling-source-map.md`, and
  `tools/README.md` now describe `src/bin/ui_help.rs` as the Help walkthrough,
  backend-tool catalog, and row-renderer boundary.

## Cleanup

- Removed inline `help_walkthrough` from `src/bin/ui.rs`.
- Removed the stale `mode_goal_card` import from `src/bin/ui.rs`; the Help
  module now owns that helper use.
- Tightened `tools/check-desktop-ui-modularization.py` so `help_walkthrough`
  cannot drift back into the monolith and the Help module keeps the expected
  Trust Center, backend registry, and advanced-reference links.

## Verification

- `cargo fmt --check`
- `cargo check --features ui --bin mhrv-f-ui`
- `cargo test --features ui --bin mhrv-f-ui ui_help`
- `python tools/check-desktop-ui-modularization.py`
- `python tools/check-tooling-source-map.py`
- `python tools/check-doc-links.py`
- `python tools/check-doc-anchors.py`
- `python tools/check-changelog-headings.py`
- `python tools/generate-changelog-index.py -Check`
- `python tools/check-ci-local-sanity-parity.py`
- `python tools/run-repo-sanity.py --skip-node`
- `python tools/check-repo-cleanliness.py`
