# Batch 48 - Coalesce Example Cleanup

Date: 2026-05-05

## Summary

- Updated `config.example.json` from `coalesce_step_ms: 40` to
  `coalesce_step_ms: 10`.
- Updated `config.full.example.json` from `coalesce_step_ms: 40` to
  `coalesce_step_ms: 10`.
- Extended `tools/check-coalesce-tuning.py` so bundled full-mode examples must
  stay on the current `10` / `1000` low-latency profile.
- Updated `tools/README.md` to document that examples are included in the
  coalesce tuning contract.

## Why

The v1.9.8/v1.9.9 tuning profile moved the default soft coalesce step from
`40 ms` to `10 ms`, but two user-facing examples still carried the old value.
That was a stale legacy default: advanced docs still mention `40` as an
explicit opt-in for users who want older conservative packing, but examples
should present the current maintained profile.

## Parity Notes

- Rust compiled defaults, Android defaults, tunnel-node settle timing, docs, and
  examples now all point at the same current profile.
- The old `40` value remains only in explanatory docs as a deliberate
  user-chosen override.

## Concurrency / Split-Brain Notes

- No runtime code changed.
- The coalesce guard now covers examples too, preventing docs/runtime/examples
  from splitting again.

## Cleanup

- No generated build output was intentionally created.
- Removed Python `__pycache__` directories created by local verification.
- No Gradle command was run.

## Verification

- `python tools/check-coalesce-tuning.py`
- `python tools/check-doc-links.py`
- `python tools/check-ci-local-sanity-parity.py`
- `python tools/run-repo-sanity.py --skip-node`
- `python tools/check-repo-cleanliness.py`
- `python tools/run-repo-sanity.py`

All checks passed on 2026-05-05.
