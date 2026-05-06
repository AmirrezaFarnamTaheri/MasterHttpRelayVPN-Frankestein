# Batch 47 - Full-Mode Coalesce Tuning Drift Gate

Date: 2026-05-05

## Summary

- Added `tools/check-coalesce-tuning.py`.
- Wired the new coalesce guard into `tools/run-repo-sanity.py`.
- Added the new guard to `tools/check-ci-local-sanity-parity.py`.
- Documented the guard in `tools/README.md`.

## Contract Now Guarded

- Rust full-mode client compiled defaults must stay `10 ms` step and
  `1000 ms` max.
- `ProxyServer` must keep translating config `0` into `10` / `1000` compiled
  defaults.
- Android `MhrvConfig` defaults and import fallbacks must stay concrete
  `10` / `1000`.
- Android serialization must omit `coalesce_step_ms` and `coalesce_max_ms` when
  they are still at those concrete defaults.
- tunnel-node straggler settle timing must stay `10 ms` step and `1000 ms` max.
- `docs/platform-defaults.json`, generated platform docs, advanced-options
  docs, and the upstream tuning changelog must explain the same profile.

## Parity Notes

- Rust serialized config keeps using `0` as the absent-field sentinel.
- Android saves concrete values so mobile exports match the current compiled
  low-latency profile.
- tunnel-node return-leg settle timing stays aligned with the client profile.

## Concurrency / Split-Brain Notes

- No runtime code changed.
- The new static gate prevents Rust client constants, proxy fallbacks, Android
  defaults, tunnel-node constants, and docs from becoming separate truths.

## Cleanup

- No generated build output was intentionally created.
- Removed Python `__pycache__` directories created by local verification.
- No Gradle command was run.

## Verification

- `python tools/check-coalesce-tuning.py`
- `python tools/check-ci-local-sanity-parity.py`
- `python tools/check-doc-links.py`
- `python tools/run-repo-sanity.py --skip-node`
- `python tools/check-repo-cleanliness.py`
- `python tools/run-repo-sanity.py`

All checks passed on 2026-05-05.
