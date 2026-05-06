# Batch 7 - Desktop UI Trust Tab Extraction (2026-05-07)

## Summary

- Moved the Desktop Trust tab wrapper from `src/bin/ui.rs` into
  `src/bin/ui_trust.rs`.
- Kept the tab behavior unchanged: read-only trust snapshot, Install CA,
  Remove CA, Check CA, support-bundle preview, and local docs links all still
  render from the same shared state and command channel.
- Promoted `Cmd` and `FormState` to crate-visible Desktop UI boundary types so
  extracted UI modules can own complete tab renderers without duplicating state
  or command definitions.

## Parity

- The Trust tab still calls the same `FormState::to_config()` path and the same
  serialized CA commands (`InstallCa`, `RemoveCa`, `CheckCaTrusted`).
- The Trust Center snapshot and support-bundle preview remain sourced from
  shared Rust trust/support-bundle helpers, not duplicated UI-side probes.
- The docs links remain local repository paths for Trust Center, safety,
  Android signing, and Doctor references.

## Cleanup

- `src/bin/ui.rs` no longer contains `trust_center_tab`,
  `support_bundle_preview`, or Trust Center snapshot implementation details.
- The Desktop UI modularization guard now rejects Trust tab renderer drift back
  into the monolith and requires the tab wrapper plus command wiring in
  `src/bin/ui_trust.rs`.
- Tooling source-map documentation now describes `ui_trust.rs` as the owner of
  the Trust tab wrapper, snapshot renderer, and support-bundle preview.

## Verification

- `cargo fmt --check`
- `cargo check --features ui --bin mhrv-f-ui`
- `cargo test --features ui --bin mhrv-f-ui ui_trust`
- `python tools/check-desktop-ui-modularization.py`
