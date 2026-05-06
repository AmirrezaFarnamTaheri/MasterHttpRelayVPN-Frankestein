# Batch 61 - Android Copy Scanner Builder Coverage

Date: 2026-05-05

## What Changed

- Expanded `tools/generate-android-hardcoded-copy-inventory.py` to detect more
  builder-style visible-copy literals:
  - `title = "..."`
  - `body = "..."`
  - `detail = "..."`
  - `placeholder = "..."`
- Kept existing detection for `Text("...")`, `label = "..."`, and
  `contentDescription = "..."`.
- Regenerated the Android hard-coded-copy inventory. The widened scanner still
  reports only the three intentional dynamic/technical tokens already tracked.
- Updated `tools/README.md` so future Android UI copy changes know which shapes
  are guarded and when the scanner must be expanded again.

## Parity / Cleanup

- This converts the Batch 60 readiness-detail manual audit into an automated
  guard for future changes.
- Android English/Persian string resources remain paired at 270 keys.
- No Android runtime, backend, desktop, config, Apps Script, or tunnel-node
  behavior changed.

## Verification

- `python tools/generate-android-hardcoded-copy-inventory.py`
- `python tools/generate-android-hardcoded-copy-inventory.py -Check`

Full batch verification is recorded in `elevation_audit_roadmap_source.md`.
