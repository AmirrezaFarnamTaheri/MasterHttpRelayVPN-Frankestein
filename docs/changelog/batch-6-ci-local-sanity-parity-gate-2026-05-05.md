# Batch 6 - CI / Local Sanity Parity Gate

Date: 2026-05-05

## Summary

Added a static guard that keeps GitHub Actions CI and local `repo-sanity`
verification from drifting apart. The project now has several high-risk local
drift gates; this batch ensures CI continues to inherit them from
`tools/run-repo-sanity.py` instead of growing a second, stale checklist.

## Changes

- Added `tools/check-ci-local-sanity-parity.py`.
- Wired the new gate into `tools/run-repo-sanity.py`.
- Documented the gate in `tools/README.md`.

## Guarded Contract

- `.github/workflows/ci.yml` must keep the `repo-sanity` job.
- CI `repo-sanity` must call `python3 tools/run-repo-sanity.py`.
- CI must keep Node setup for backend JavaScript and Apps Script syntax/tests.
- CI must keep root Rust `fmt`, all-feature `clippy`, and UI-feature tests.
- CI must keep `tunnel-node` clippy and tests.
- CI must keep Android JVM `PlatformDefaultsContractTest` in the
  pre-provisioned CI environment.
- Local `tools/run-repo-sanity.py` must continue to include the Android VPN
  lifecycle, Desktop Test Relay mode, tunnel-node drain/concurrency,
  platform-defaults, config-registry, parity-matrix, and stale Android/docs
  gates.

## Parity / Split-Brain Notes

- CI and local repo sanity now have a guard enforcing one source of truth.
- Android Gradle remains CI/pre-provisioned only; local repo-sanity remains
  static and Gradle-free.
- Release-blocking Rust/tunnel-node compilation remains in CI's `rust` job
  rather than being folded into the Python sanity runner.

## Verification

- `python tools/check-ci-local-sanity-parity.py`
- `python tools/run-repo-sanity.py --skip-node`
- `python tools/check-doc-links.py`
- `python tools/run-repo-sanity.py`
- `python tools/check-repo-cleanliness.py`

All checks passed on 2026-05-05. The full repo sanity route confirmed CI/local
parity after running backend JavaScript syntax checks, Apps Script syntax/tests,
all Python drift gates, generated config/readiness/parity checks, and stale
JSON/XML/Android scans.

## Cleanup

- Removed the generated `tools/__pycache__` directory after verification.
- No Gradle command was run and no Android build output was generated.
- Process inspection found no active `gradle`, `java`, or `kotlinc` processes
  during closeout.
