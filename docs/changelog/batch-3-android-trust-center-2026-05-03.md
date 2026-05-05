# Batch 3 - Android Trust Center projection (2026-05-03)

## Summary

Android now has a visible Trust Center surface instead of relying only on the
readiness card plus the standalone certificate button.

## User-facing changes

- Added a main-screen **Trust Center** card to the Android Compose UI.
- The card shows whether the current mode needs a local user CA:
  - Apps Script, Serverless JSON, and Direct show the CA install/trust state.
  - Full tunnel explains that no local CA is needed.
- Moved the MITM certificate install/repair action into the Trust Center card.
- Added Android copy that explains user-CA scope, including the Android 7+
  per-app opt-out behavior.
- Added signing continuity copy that points users toward the committed-keystore
  and CI-release policy.
- Added support-data sharing copy so copied logs and exported configs are
  treated as sensitive material.

## Parity and consistency

- Desktop has a dedicated Trust tab.
- CLI has `mhrv-f trust-center` and `mhrv-f trust-center --json`.
- Support bundles include `trust.json`.
- Android now has the mobile Trust Center projection, while deliberately keeping
  support-bundle preview/export as future work.
- English and Persian Android strings were updated together.
- README, Android docs, Persian Android docs, docs index, and Trust Center docs
  were updated in the same step.

## Cleanup

- Removed the separate Android main-screen certificate button to avoid two trust
  action surfaces.
- Reused the existing serialized CA install dialog and `CaInstall` checks.
- Added no new background trust job, no new CA mutation path, and no duplicate
  trust model.
- Kept Gradle untouched; Android verification remains static/local plus CI.

## Verification

Passed:

- `cargo fmt --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all-targets --features ui --quiet` (179 root tests + 5 UI/config tests)
- `python tools/run-repo-sanity.py`
- `python tools/check-doc-links.py`
- `python tools/check-repo-cleanliness.py`
- `cargo clippy --all-targets -- -D warnings` in `tunnel-node`
- `cargo test --all-targets --quiet` in `tunnel-node` (34 tests)

Not run:

- Gradle / Android build tasks, by maintainer instruction. Android coverage for
  this batch is static XML/string/config parity plus CI responsibility.
