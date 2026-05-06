# Batch 6 - Android Support Snapshot Schema Guard - 2026-05-06

## Summary

Added a dedicated static guard for the Android copied support-snapshot schema.
The existing redaction guard protects the Kotlin owner and tests; this new guard
also checks the schema documentation, docs index, related docs, and repo-sanity
wiring.

## Changed

- Added `tools/check-android-support-snapshot-schema.py`.
- Wired the guard into `tools/run-repo-sanity.py`.
- Added the guard to `tools/check-ci-local-sanity-parity.py` so CI/local parity
  fails if repo sanity drops it.
- Updated `tools/README.md` with the new command.

## Guarded Contract

The guard requires:

- active source/test marker `android-support-snapshot/v2`;
- Doctor summary fields in source, tests, and schema docs;
- redaction exclusions for raw Doctor JSON and Doctor details/fixes;
- `docs/index.md` link to `docs/android-support-snapshot.md`;
- Android EN/FA, Trust Center, and tools docs references;
- local repo-sanity wiring.

## Verification

- `python tools/check-android-support-snapshot-schema.py`
- `python tools/check-android-support-redaction.py`
- `python tools/check-ci-local-sanity-parity.py`
- `python tools/run-repo-sanity.py --skip-node`
