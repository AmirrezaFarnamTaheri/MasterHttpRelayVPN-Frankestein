# Batch 7 - Desktop UI Setup Wizard Extraction (2026-05-07)

## Summary

- Added `src/bin/ui_setup.rs` for the Desktop first-run Setup wizard renderer.
- Moved the wizard stepper, mode selection, relay credential prompts, CA step,
  and diagnostics actions out of `src/bin/ui.rs`.
- Kept the visible wizard flow unchanged: Mode, Relay, CA, Diagnostics, plus
  the same Test relay, Doctor, Install CA, and Check CA command wiring.

## Parity

- The extracted wizard still mutates the same `FormState` fields and sends the
  same `Cmd` variants over the same Desktop command channel.
- Config validation for Test relay / Doctor still flows through
  `FormState::to_config()`.
- No runtime proxy, Android, CLI, config schema, backend helper, readiness,
  support-bundle, or release behavior changed.

## Cleanup

- `src/bin/ui.rs` no longer contains the `show_first_run_wizard` renderer.
- The Desktop UI modularization guard now requires the Setup wizard in
  `src/bin/ui_setup.rs` and rejects drift back into `src/bin/ui.rs`.
- Tooling source-map documentation and tools README now record `ui_setup.rs`
  ownership.

## Verification

- `cargo fmt`
- `cargo fmt --check`
- `cargo check --features ui --bin mhrv-f-ui`
- `cargo test --features ui --bin mhrv-f-ui ui_setup`
- `python tools/check-desktop-ui-modularization.py`
