# Batch 6 - Desktop Test Relay Mode Gate

Date: 2026-05-05

## Summary

Added a repo-sanity static gate for the Desktop **Test Relay** mode guard. This
keeps `full` and `direct` modes from regressing back to a misleading red
`Test failed` result when the relay-path test is not the correct verification
tool.

## Changes

- Added `tools/check-desktop-test-relay-mode-guard.py`.
- Wired the new gate into `tools/run-repo-sanity.py`.
- Documented the gate in `tools/README.md`.

## Guarded Contract

- The `Cmd::Test` UI path must inspect `cfg.mode_kind()`.
- `Mode::Full` must show a skip/explainer message instead of running
  `test_cmd::run`.
- `Mode::Direct` must show a skip/explainer message instead of running
  `test_cmd::run`.
- Full/direct skips must set `last_test_ok = None`, not `Some(false)`.
- The skip branch must log `[ui] test skipped: ...` and `continue` before the
  normal relay test is spawned.
- `docs/relay-modes.md` must keep the matching user-facing verification
  guidance.

## Parity / Split-Brain Notes

- Desktop UI behavior and docs are checked together, so Test Relay does not
  become a second routing oracle separate from runtime mode semantics.
- Android/backend behavior is unchanged; this is Desktop UX regression
  prevention for the upstream absorption batch.

## Verification

- `python tools/check-desktop-test-relay-mode-guard.py`
- `python tools/run-repo-sanity.py`
- `python tools/check-doc-links.py`
- `python tools/check-repo-cleanliness.py`

All checks passed on 2026-05-05. The full repo sanity route also re-ran the
Apps Script syntax/tests, backend JavaScript syntax checks, config/readiness
registry gates, Android static parity gates, generated-doc checks, and stale
JSON/XML/Android scan.

## Cleanup

- Removed the generated `tools/__pycache__` directory after verification.
- No Gradle command was run and no Android build output was generated.
- No legacy Test Relay path was kept for `full` / `direct`; those modes now
  remain skip/explainer-only until a true end-to-end verifier exists.
