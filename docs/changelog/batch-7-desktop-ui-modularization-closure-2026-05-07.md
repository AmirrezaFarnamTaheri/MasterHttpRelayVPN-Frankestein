# Batch 7 - Desktop UI Modularization Closure (2026-05-07)

## Summary

- Closed the current Desktop UI modularization run after the extracted leaf
  surfaces reached a stable boundary.
- Confirmed the remaining `src/bin/ui.rs` bulk is mostly stateful app shell,
  tab composition, form editing, background command handling, and shared
  runtime orchestration rather than another obvious low-risk leaf renderer.
- Kept the next extraction decision explicit instead of forcing a split that
  would create broad visibility churn without reducing behavior risk.

## Extracted Owners

- `src/bin/ui_format.rs`: formatting helpers.
- `src/bin/ui_fs.rs`: file/resource-opening helpers.
- `src/bin/ui_style.rs`: visual tokens, theme, sections, help text, buttons,
  and compact form rows.
- `src/bin/ui_xhttp.rs`: XHTTP defaults, VLESS link/deploy-note generation,
  cloud deploy polling, and XHTTP renderer.
- `src/bin/ui_doctor.rs`: Doctor summary card and Doctor level helpers.
- `src/bin/ui_help.rs`: Help walkthrough and backend-tool catalog.
- `src/bin/ui_trust.rs`: Trust tab, Trust Center snapshot, and support-bundle
  preview.
- `src/bin/ui_setup.rs`: first-run Setup wizard.
- `src/bin/ui_mode.rs`: mode summary, mode dashboard, readiness repair routing,
  and dashboard chips/actions.

## Cleanup

- The Desktop UI modularization guard rejects drift of the extracted helpers
  and renderers back into `src/bin/ui.rs`.
- Vocabulary and readiness guards now follow the extracted mode-dashboard owner
  instead of assuming all Desktop readiness copy lives in the monolith.
- Tooling source-map documentation records each extracted owner.

## Remaining

- Future Desktop work can still split the large stateful Setup, Network,
  Advanced, Monitor, and background-worker areas, but those are larger
  ownership changes and should be taken as separate behavior-preserving
  refactors with their own tests.

## Verification

- `cargo fmt --check`
- `cargo check --features ui --bin mhrv-f-ui`
- `cargo test --features ui --bin mhrv-f-ui ui_mode`
- `python tools/check-desktop-ui-modularization.py`
- `python tools/check-mode-vocabulary.py`
- `python tools/check-readiness-ui-contract.py`
- `python tools/run-repo-sanity.py --skip-node`
