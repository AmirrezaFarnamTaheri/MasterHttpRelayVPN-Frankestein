# Batch 6 - Android Small Home Copy Localization

Date: 2026-05-05

## What changed

- Moved several small `HomeScreen.kt` visible literals into Android string
  resources:
  - certificate subject label and unknown fallback;
  - `+ Add` action;
  - `Block QUIC` label and helper text;
  - upstream SOCKS5 `host:port` placeholder.
- Added matching Persian resources for every new key.
- Regenerated `docs/android-hardcoded-copy-inventory.md`.
- Updated roadmap counts and localization progress notes.

## Current inventory after this batch

- Total visible hard-coded Android Kotlin literals: 21.
- Should move to string resources: 16.
- Dynamic or technical tokens to review separately: 5.
- Android string resources are paired at 223 English keys and 223 Persian keys.

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
