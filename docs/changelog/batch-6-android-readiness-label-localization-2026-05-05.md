# Batch 6 - Android Readiness Label Localization

Date: 2026-05-05

## What changed

- Changed Android readiness items from storing raw label strings to storing
  `@StringRes` label IDs.
- Moved the selected-mode readiness card title and status chips into resources.
- Moved all readiness row labels into English/Persian string resources:
  deployment IDs, AUTH_KEY, serverless origin, relay path, Google edge IP,
  front SNI, routing mode, LAN exposure/guards, CA trust, and full-tunnel
  readiness labels.
- Regenerated `docs/android-hardcoded-copy-inventory.md`.
- Updated roadmap counts and Android localization status.

## Current inventory after this batch

- Total scanner-visible hard-coded Android Kotlin literals: 3.
- Should move to string resources: 0.
- Dynamic or technical tokens to review separately: 3.
- Android string resources are paired at 244 English keys and 244 Persian keys.

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
