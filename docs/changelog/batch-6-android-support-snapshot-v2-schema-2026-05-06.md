# Batch 6 - Android Support Snapshot v2 Schema - 2026-05-06

## Summary

The Android copied support snapshot now declares
`schema: android-support-snapshot/v2`. The previous batch added Doctor summary
fields, so leaving the marker at `v1` would have made the copied artifact look
older than its actual shape.

## Changed

- Bumped the Android support snapshot marker from `v1` to `v2`.
- Updated `SupportRedactionTest.kt` and
  `tools/check-android-support-redaction.py` to require the new marker.
- Added `docs/android-support-snapshot.md`, a dedicated schema/redaction
  contract for the copied mobile support text.
- Linked the schema doc from `docs/index.md`.
- Updated Android, Persian Android, Trust Center, and tools docs to point at
  the schema page.

## Parity

- Android remains intentionally lighter than Desktop/CLI support bundles.
- Desktop/CLI still own full `support-bundle` artifacts.
- Android `v2` documents the mobile projection of the same Doctor/readiness
  vocabulary without exporting raw Doctor JSON.

## Cleanup

- Removed the stale `android-support-snapshot/v1` marker from active code,
  tests, and the static guard.
- No backward-compatibility branch was added for old copied text because this
  is a human-readable support artifact, not an imported config format.

## Verification

- `python tools/check-android-support-redaction.py`
- `python tools/check-doc-links.py`
- `python tools/check-ci-local-sanity-parity.py`
