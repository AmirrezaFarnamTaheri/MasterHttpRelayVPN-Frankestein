# Batch 6 - Android Hard-Coded Copy Inventory

Date: 2026-05-05

## What changed

- Added `tools/generate-android-hardcoded-copy-inventory.py`.
- Generated `docs/android-hardcoded-copy-inventory.md`.
- Wired the inventory `-Check` mode into `tools/run-repo-sanity.py`.
- Added the inventory gate to `tools/check-ci-local-sanity-parity.py`.
- Documented the workflow in `tools/README.md`.
- Linked the generated inventory from `docs/index.md`.
- Updated the roadmap so `B0.3` is now complete with an auditable inventory.

## Contract

- Android visible copy that still lives in Kotlin must be listed in the generated
  inventory.
- The inventory separates strings that should move to resources from dynamic
  placeholders and pure technical tokens.
- The gate is static and Gradle-free; it does not build or execute the Android
  app locally.
- This batch inventories the remaining hard-coded copy; it does not complete the
  later migration tasks for moving all visible copy to resources.

## Current inventory

- Total visible hard-coded literals: 30.
- Should move to string resources: 24.
- Dynamic or technical tokens to review separately: 6.

## Verification

- `python tools/generate-android-hardcoded-copy-inventory.py`
- `python tools/generate-android-hardcoded-copy-inventory.py -Check`
- `python tools/check-ci-local-sanity-parity.py`
- `python tools/check-doc-links.py`
- `python tools/check-repo-cleanliness.py`
- `python tools/run-repo-sanity.py --skip-node`

Android Gradle/JVM tests were not run locally; they remain CI/pre-provisioned
checks by project rule.
