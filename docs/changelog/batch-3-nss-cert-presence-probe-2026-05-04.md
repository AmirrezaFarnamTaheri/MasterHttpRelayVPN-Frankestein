# Batch 3 - Read-only NSS certificate presence probe (2026-05-04)

## Summary

Trust Center now reports whether the mhrv-f CA nickname is actually present in
discovered Firefox and Chrome/Chromium NSS databases when `certutil` is
available. This closes the gap between "we found profiles" and "the browser
store can see the CA".

## User-facing changes

- `mhrv-f trust-center` now prints:
  - Firefox profile count;
  - Firefox profile count with NSS DBs;
  - Firefox NSS CA presence when `certutil` is available;
  - Chrome/Chromium NSS DB presence;
  - Chrome/Chromium NSS CA presence when `certutil` is available.
- Desktop Trust Center shows the same new Firefox/Chrome NSS CA rows.
- `trust.json` gains optional NSS CA presence fields for support bundles and
  future UI projections.

## Safety and cleanup

- The probe is strictly read-only.
- It does not create NSS databases.
- It does not edit Firefox `user.js`.
- It does not install or remove certificates.
- Existing install/remove flows remain the only mutation paths.
- If `certutil` is missing, the new presence fields are `null` / unavailable
  instead of guessing.

## Parity

- CLI, Desktop Trust tab, support-bundle `trust.json`, and docs now agree on
  the deeper NSS signal.
- Android remains N/A for NSS because Android uses `AndroidCAStore` instead of
  browser NSS profiles.

## Verification

- `cargo fmt --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all-targets --features ui --quiet` (179 root tests + 5 UI/config tests)
- `cargo run --quiet --bin mhrv-f -- trust-center --json` with JSON parse smoke; local machine reports NSS CA presence fields as unavailable because `certutil` is not installed.
- `python tools/run-repo-sanity.py`
- `python tools/check-doc-links.py`
- `cargo clippy --all-targets -- -D warnings` in `tunnel-node`
- `cargo test --all-targets --quiet` in `tunnel-node` (34 tests)
- No Gradle download/install/run was performed; Android impact is docs/config parity only for this NSS-specific batch.
