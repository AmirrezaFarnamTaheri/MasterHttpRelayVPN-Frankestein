# Batch 3 - Android redacted support snapshot (2026-05-04)

## Summary

Android Trust Center now has a copyable, redacted support snapshot. This gives
mobile users a safer first diagnostic artifact without adding a new JNI support
bundle exporter or duplicating the desktop bundle pipeline.

## User-facing changes

- Added **Copy redacted support snapshot** to the Android Trust Center card.
- The copied snapshot includes:
  - selected mode and Android routing mode;
  - split-tunnel policy and selected-app count;
  - whether a local user CA is required and installed;
  - listener ports and LAN exposure state;
  - deployment count plus masked deployment IDs;
  - whether auth keys and serverless credentials are configured;
  - SNI/advanced tuning counts and booleans;
  - whether account groups, fronting groups, or unknown root fields are being
    preserved from a desktop/imported config.
- The snapshot explicitly omits:
  - `auth_key`;
  - serverless `AUTH_KEY`;
  - LAN token;
  - upstream SOCKS5 value;
  - raw unknown JSON;
  - unmasked deployment IDs.

## Parity and consistency

- Desktop and CLI remain the full support-bundle authority.
- Android now has a lightweight mobile-safe projection for first-line support.
- Trust Center docs and Android English/Persian guides were updated together.
- English and Persian Android strings were updated together.

## Cleanup

- Added no new native/JNI API.
- Added no new background job.
- Added no new file export surface, so no storage permission or race-prone file
  lifecycle was introduced.
- Kept the copy action local to the existing Trust Center card.

## Verification

Passed:

- `cargo fmt --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all-targets --features ui --quiet` (179 root tests + 5 UI/config tests)
- `python tools/run-repo-sanity.py`
- `python tools/check-doc-links.py`
- `cargo clippy --all-targets -- -D warnings` in `tunnel-node`
- `cargo test --all-targets --quiet` in `tunnel-node` (34 tests)

Not run:

- Gradle / Android build tasks, by maintainer instruction. Android coverage for
  this batch is static XML/string/config parity plus CI responsibility.
