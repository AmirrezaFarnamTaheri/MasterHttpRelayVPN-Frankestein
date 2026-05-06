# Batch 62 - Android Copy Zero-Localize Gate

Date: 2026-05-05

## What Changed

- Tightened `tools/generate-android-hardcoded-copy-inventory.py -Check` so it
  fails when any detected Android hard-coded visible literal is classified as
  `localize`.
- Kept the generated inventory for intentional dynamic/technical tokens only.
- Regenerated `docs/android-hardcoded-copy-inventory.md` with the explicit
  zero-localize contract.
- Updated `tools/README.md` so future Android copy changes know that
  user-facing prose must move to string resources in the same change.

## Parity / Cleanup

- Android English/Persian resources remain paired at 270 keys.
- Scanner-visible hard-coded Android copy remains at 3 dynamic/technical tokens
  and 0 localize candidates.
- CI/local repo-sanity inherits the stricter rule through the existing
  `-Check` call.
- No Android runtime, backend, desktop, config, Apps Script, or tunnel-node
  behavior changed.

## Verification

- `python tools/generate-android-hardcoded-copy-inventory.py`
- `python tools/generate-android-hardcoded-copy-inventory.py -Check`

Full batch verification is recorded in `elevation_audit_roadmap_source.md`.
