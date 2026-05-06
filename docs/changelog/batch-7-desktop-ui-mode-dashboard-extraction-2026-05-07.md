# Batch 7 - Desktop UI Mode Dashboard Extraction (2026-05-07)

## Summary

- Added `src/bin/ui_mode.rs` for Desktop mode summary and readiness dashboard
  rendering.
- Moved mode summary copy, mode dashboard data construction, operational
  readiness augmentation, repair-route mapping, dashboard renderer, and chip
  button helpers out of `src/bin/ui.rs`.
- Moved the existing Desktop readiness/repair tests with the code they protect.

## Parity

- The extracted dashboard still consumes the same `FormState`, shared
  `readiness` IDs, repair targets, and Desktop tab enum.
- Repair buttons still route to the same Setup, Network, Advanced, Trust, and
  Help tabs.
- No Android, CLI, config schema, backend helper, runtime proxy, support-bundle,
  or release workflow behavior changed.

## Cleanup

- `src/bin/ui.rs` no longer owns `mode_summary`, `mode_dashboard`,
  `desktop_repair_action`, `mode_dashboard_panel`, `info_chip`, or
  `ghost_action`.
- The Desktop UI modularization guard now rejects mode-dashboard drift back into
  `src/bin/ui.rs` and requires the moved tests in `src/bin/ui_mode.rs`.
- Tooling source-map documentation and tools README now record `ui_mode.rs`
  ownership.

## Verification

- `cargo fmt`
- `cargo fmt --check`
- `cargo check --features ui --bin mhrv-f-ui`
- `cargo test --features ui --bin mhrv-f-ui ui_mode`
- `python tools/check-desktop-ui-modularization.py`
