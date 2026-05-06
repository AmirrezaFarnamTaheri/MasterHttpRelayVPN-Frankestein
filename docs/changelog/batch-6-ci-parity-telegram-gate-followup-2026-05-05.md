# Batch 6 - CI Parity Telegram Gate Follow-up

Date: 2026-05-05

## Summary

Closed a small follow-up gap from the previous two hardening batches: the
CI/local sanity parity guard now explicitly requires the Telegram release
notification renderer gate to remain in `tools/run-repo-sanity.py`.

## Changes

- Updated `tools/check-ci-local-sanity-parity.py` so the local-runner required
  gate list includes `tools/check-telegram-release-notify.py`.

## Guarded Contract

- CI must continue to call `python3 tools/run-repo-sanity.py`.
- `tools/run-repo-sanity.py` must continue to include the Telegram release
  renderer check.
- Telegram release rendering remains an optional projection of
  `docs/changelog/v*.md`, not a separate source of truth.

## Parity / Split-Brain Notes

- This connects Batch 40's CI/local parity governance to Batch 41's Telegram
  renderer hardening.
- No runtime, Android, backend, or release workflow behavior changed.

## Verification

- `python tools/check-ci-local-sanity-parity.py`
- `python tools/run-repo-sanity.py --skip-node`
- `python tools/check-doc-links.py`
- `python tools/check-repo-cleanliness.py`
- `python tools/run-repo-sanity.py`

All checks passed on 2026-05-05. The full repo sanity route confirmed the new
required Telegram renderer gate is present in the local sanity runner.

## Cleanup

- Removed generated `.github/scripts/__pycache__` and `tools/__pycache__`
  directories after verification.
- No Gradle command was run and no Android build output was generated.
- Process inspection found no active `gradle`, `java`, or `kotlinc` processes
  during closeout.
