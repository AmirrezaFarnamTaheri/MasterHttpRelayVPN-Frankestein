# Batch 3 - Browser profile trust details (2026-05-04)

## Summary

Trust Center now includes redacted per-Firefox-profile browser trust details in
addition to aggregate NSS counts. This makes support snapshots and Desktop
diagnostics more actionable when one Firefox profile trusts the local CA and
another does not.

## User-facing changes

- `mhrv-f trust-center` prints a `Firefox profile details` list when profiles
  are discovered.
- Desktop Trust Center shows a compact, capped profile detail list under the
  Firefox/NSS rows.
- `trust.json` schema version is now `2` and includes
  `browser.firefox_profiles[]` entries with:
  - redacted profile label;
  - NSS DB presence;
  - mhrv-f CA nickname presence when `certutil` can query NSS;
  - app-managed `enterprise_roots` marker state;
  - user-owned `enterprise_roots` setting state.

## Privacy and safety

- Profile details use only the Firefox profile directory name.
- Parent paths, home directories, Windows usernames, and full profile paths are
  not exported through the shared Trust snapshot.
- The probe remains read-only and does not create/edit/remove browser stores.
- CA mutations remain limited to explicit install/remove actions.

## Parity

- CLI text, Desktop Trust tab, support-bundle `trust.json`, Trust Center docs,
  Desktop docs, and Doctor docs now describe the same profile-level signal.
- Android remains N/A for NSS profile details because Android uses
  `AndroidCAStore` and app-level trust policies instead of Firefox desktop NSS.

## Verification

- `cargo fmt --check`
- `cargo test trust_center::tests --quiet` (4 focused Trust Center tests)
- `cargo check --features ui --bin mhrv-f-ui`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all-targets --features ui --quiet` (180 root tests + 5 UI/config tests)
- `cargo run --quiet --bin mhrv-f -- trust-center --json` with JSON parse smoke; local output reported schema `2`, one redacted Firefox profile, and NSS CA presence unavailable because `certutil` is not installed.
- `python tools/run-repo-sanity.py`
- `python tools/check-doc-links.py`
- `cargo clippy --all-targets -- -D warnings` in `tunnel-node`
- `cargo test --all-targets --quiet` in `tunnel-node` (34 tests)
- `python tools/check-repo-cleanliness.py`
- No Gradle download/install/run was performed; Android impact is N/A for Firefox/NSS profile details and documented as such.
