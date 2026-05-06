# Batch 6 - Android Support Snapshot Doctor Summary - 2026-05-06

## Summary

Android's copied Trust Center support snapshot now includes the latest in-app
Doctor result as a redacted summary when that result belongs to the current
config. This closes the support-sharing gap left after the Android Doctor card:
users can run Doctor, copy the mobile support snapshot, and share enough
structured diagnostic context without exposing raw Doctor details.

## Changed

- Extended `SupportRedaction.kt` so `androidSupportSnapshot` accepts optional
  Doctor JSON.
- Kept the Doctor JSON contract as the only diagnostic source of truth.
- Added privacy-safe Doctor fields to the copied snapshot:
  - `doctor_available`;
  - `doctor_ok`;
  - `doctor_items_total`;
  - `doctor_items_ok`;
  - `doctor_items_warn`;
  - `doctor_items_fail`;
  - `doctor_problem_ids`.
- Deliberately excluded Doctor titles, details, fixes, endpoint URLs, and raw
  JSON from the copied snapshot.
- Updated `HomeScreen.kt` to keep the latest raw Doctor JSON only for support
  snapshot summarization.
- Cleared cached Doctor report/support JSON whenever config is edited, avoiding
  stale diagnostics after a save.
- Updated Android support-redaction tests and static drift gate markers.
- Updated Android, Persian Android, Trust Center, and tools docs.
- Regenerated the Android hard-coded copy inventory after the technical
  support-snapshot literals changed.

## Parity

- Desktop and CLI support bundles already include full `doctor.json` as a
  reviewed support artifact.
- Android remains intentionally lighter: it copies only a summary, not a bundle
  and not raw Doctor JSON.
- The same Doctor JSON contract powers desktop, CLI, Android UI, and Android
  support-snapshot summarization.
- No backend scripts, tunnel-node protocol, config examples, or runtime proxy
  behavior changed.

## Split-Brain / Concurrency Notes

- `SupportRedaction.kt` remains the only Android support-snapshot owner.
- `HomeScreen.kt` remains a caller and state holder; it does not define masking
  or snapshot formatting helpers.
- The Doctor result is tied to a `cfg.toJson()` snapshot and is ignored if the
  config changes before JNI returns.
- Config edits clear cached Doctor report/support JSON so copied support text
  cannot describe a previous config.

## Cleanup

- No legacy UI-local support snapshot logic was reintroduced.
- No raw Doctor export path was added on Android.
- No new localized visible strings were required.

## Verification

- `python tools/check-android-support-redaction.py`
- `python tools/check-android-doctor-summary-ui.py`
- `python tools/check-android-doctor-jni-bridge.py`
- `python tools/check-android-string-resource-parity.py`
- `python tools/generate-android-hardcoded-copy-inventory.py -Check`
- `python tools/check-ci-local-sanity-parity.py`
- `python tools/check-doc-links.py`
- `cargo fmt --check`
- `cargo check --lib`
- `cargo check --features ui --bin mhrv-f-ui`
- `python tools/run-repo-sanity.py --skip-node`
- `python tools/check-repo-cleanliness.py`

No Gradle, Java, or Kotlin build processes were left running.
