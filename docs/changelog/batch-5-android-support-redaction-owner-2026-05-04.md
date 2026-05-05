# Batch 5 - Android support redaction owner (2026-05-04)

## Summary

Android's redacted support snapshot is no longer owned by the Compose
`HomeScreen`. The snapshot generator and deployment-ID masking helper now live
in a package-level utility so Android support redaction has a stable owner,
matching the Rust-side redaction consolidation.

## User-facing changes

- No visible behavior change is intended.
- The Trust Center card still copies the same redacted Android support snapshot.
- Deployment IDs remain masked.
- Auth keys, serverless auth keys, LAN tokens, upstream SOCKS5, and raw
  preserved JSON remain omitted.

## Implementation details

- Added `android/app/src/main/java/com/farnam/mhrvf/SupportRedaction.kt`.
- Moved Android support snapshot generation out of
  `android/app/src/main/java/com/farnam/mhrvf/ui/HomeScreen.kt`.
- Added `android/app/src/test/java/com/farnam/mhrvf/SupportRedactionTest.kt`.
- Updated Android, Persian Android, and Trust Center docs to point at the new
  owner.

## Parity

- Rust has `src/redaction.rs` for CLI/Desktop/support-bundle policy.
- Android has `SupportRedaction.kt` for the smaller mobile copied snapshot.
- The split is documented until a future JNI/shared support-bundle exporter is
  justified.

## Verification

- `python tools/check-json-xml-android-stale.py`
- `python tools/check-doc-links.py`
- `python tools/run-repo-sanity.py`
- Static Kotlin brace-balance check for:
  - `android/app/src/main/java/com/farnam/mhrvf/SupportRedaction.kt`
  - `android/app/src/main/java/com/farnam/mhrvf/ui/HomeScreen.kt`
  - `android/app/src/test/java/com/farnam/mhrvf/SupportRedactionTest.kt`
- Static stale-helper scan confirmed `HomeScreen.kt` no longer owns
  `maskedDeploymentId`, `androidSupportSnapshot`, or a local `yesNo` helper.
- `python tools/check-repo-cleanliness.py`
- No Gradle download/install/run was performed; JVM execution remains CI or
  approved pre-provisioned environment only.
