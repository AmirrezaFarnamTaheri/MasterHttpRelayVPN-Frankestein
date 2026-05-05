# Batch 5 - Rust redaction policy consolidation (2026-05-04)

## Summary

Rust diagnostic redaction now has a shared module instead of separate masking
rules inside support-bundle and Doctor code. This reduces split-brain risk for
support bundles, CLI diagnostics, future Desktop diagnostics, and status/support
exports.

## User-facing changes

- Support bundles keep the same privacy behavior:
  - Apps Script auth keys are replaced;
  - serverless auth keys are replaced;
  - LAN tokens are removed from config and scrubbed from logs;
  - deployment IDs are masked as prefix/suffix;
  - private CA keys are not included.
- Doctor tunnel-node health URLs continue to strip username/password
  credentials before display.
- Redaction behavior is now documented as a shared Rust policy rather than a
  support-bundle-only implementation detail.

## Implementation details

- Added `src/redaction.rs` with reusable helpers and tests:
  - `mask_deployment_id`;
  - `redact_url_credentials`;
  - `redact_config_secrets_in_text`.
- Exported the module from `src/lib.rs`.
- Rewired `src/support_bundle.rs` to use the shared helpers for:
  - sanitized config output;
  - recent-log text scrubbing.
- Rewired `src/doctor.rs` to use the shared URL credential redactor.
- Removed the old duplicate deployment-ID masking helper from
  `support_bundle.rs`.

## Parity

- CLI Doctor, support-bundle export, Desktop support-bundle manifest copy, and
  Trust Center docs now point at one Rust-side redaction vocabulary.
- Android still has a smaller Kotlin support snapshot; its masking rules remain
  documented separately until a JNI/shared export path is designed.

## Verification

- `cargo fmt --check`
- `cargo test redaction --quiet` (5 focused redaction tests)
- `cargo test support_bundle --quiet` (4 focused support-bundle tests)
- `cargo check --bin mhrv-f`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all-targets --features ui --quiet` (184 root tests + 5 UI/config tests)
- `python tools/run-repo-sanity.py`
- `python tools/check-doc-links.py`
- `cargo clippy --all-targets -- -D warnings` in `tunnel-node`
- `cargo test --all-targets --quiet` in `tunnel-node` (34 tests)
- No Gradle download/install/run was performed; Android remains static/CI-governed for this Rust-side batch.
