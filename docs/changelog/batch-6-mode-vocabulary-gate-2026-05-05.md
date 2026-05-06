# Batch 52 - Canonical Mode Vocabulary Gate

Date: 2026-05-05

## Summary

- Added `tools/check-mode-vocabulary.py`.
- Wired the guard into `tools/run-repo-sanity.py`.
- Added the guard to `tools/check-ci-local-sanity-parity.py`.
- Documented the guard in `tools/README.md`.
- Updated `docs/relay-modes.md` so the Direct section explicitly names the
  user-facing **Direct fronting** label used by Desktop and Android.

## Guarded Contract

- Wire/config mode values remain `apps_script`, `vercel_edge`, `direct`, and
  `full`.
- User-facing labels stay **Apps Script**, **Serverless JSON**,
  **Direct fronting**, and **Full tunnel**.
- Desktop and Android mode selectors use friendly product labels instead of raw
  config names.
- `README.md`, `docs/index.md`, and `docs/relay-modes.md` keep the canonical
  mode comparison and `vercel_edge` compatibility explanation.

## Parity Notes

- This closes the roadmap mode-vocabulary drift without changing runtime
  behavior.
- The first targeted run caught a real docs gap: `docs/relay-modes.md` did not
  explicitly contain the **Direct fronting** label even though Desktop and
  Android used it.

## Cleanup

- No legacy compatibility path was added.
- No generated build output was intentionally created.
- Removed Python `__pycache__` directories created by the full local sanity
  pass:
  - `.github/scripts/__pycache__`
  - `tools/__pycache__`
- No Gradle command was run.

## Verification

- `python tools/check-mode-vocabulary.py`
- `python tools/check-ci-local-sanity-parity.py`
- `python tools/check-doc-links.py`
- `python tools/check-repo-cleanliness.py`
- `python tools/run-repo-sanity.py --skip-node`
