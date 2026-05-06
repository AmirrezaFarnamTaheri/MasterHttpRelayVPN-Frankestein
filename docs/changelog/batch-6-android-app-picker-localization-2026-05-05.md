# Batch 6 - Android App Picker Localization

Date: 2026-05-05

## What changed

- Moved the App Picker dialog's visible English copy out of Kotlin and into
  Android string resources.
- Added matching Persian translations for the App Picker title, search label,
  system-app toggle, and save action.
- Reused the existing shared `btn_cancel` resource for the dismiss action.
- Regenerated `docs/android-hardcoded-copy-inventory.md`.
- Updated the roadmap and changelog bookkeeping for the reduced hard-coded copy
  count.

## Current inventory after this batch

- Total visible hard-coded Android Kotlin literals: 25.
- Should move to string resources: 19.
- Dynamic or technical tokens to review separately: 6.

## Verification

- `python tools/generate-android-hardcoded-copy-inventory.py`
- `python tools/generate-android-hardcoded-copy-inventory.py -Check`
- `python tools/check-android-string-resource-parity.py`
- `python tools/check-ci-local-sanity-parity.py`
- `python tools/check-doc-links.py`
- `python tools/check-repo-cleanliness.py`
- `python tools/run-repo-sanity.py --skip-node`

Android Gradle/JVM tests were not run locally; they remain CI/pre-provisioned
checks by project rule.
