# Changelog - Strategic Batch 3 Desktop Trust tab (2026-05-03)

Maintainer-facing record for the larger Desktop Trust Center iteration.

## Summary

| Field | Detail |
|-------|--------|
| What changed | Added a first-class Desktop **Trust** tab, moved CA readiness repair routing to that tab, and documented the expanded Desktop tab set. |
| Why | Help-only trust status was useful but not prominent enough for certificate/signing/support workflows. The Trust tab gives the app a dedicated operational surface without creating a separate trust model. |
| Files changed | `src/bin/ui.rs`, `README.md`, `docs/index.md`, `docs/ui-desktop.md`, `docs/trust-center.md`, `elevation_audit_roadmap_source.md`, this changelog. |
| Desktop impact | New Trust tab with shared Trust Center snapshot, support-bundle manifest table, CA actions, and docs links. CA repair actions now navigate to Trust. |
| Android impact | None in code; Android Trust projection remains pending. |
| Backend impact | None. |
| Docs impact | README, docs index, Desktop UI reference, and Trust Center docs now reflect the dedicated tab. |

## Behavior

- The top tab bar now includes **Trust**.
- The Trust tab shows:
  - shared trust snapshot;
  - CA certificate/key/trust status;
  - Firefox/NSS/certutil probe facts;
  - Android signing policy;
  - support-bundle file table and sensitivity flags;
  - Trust/Safety/Android signing/Doctor doc links.
- **Install CA**, **Remove CA**, and **Check CA** are available in the tab and
  use the existing `Cmd` channel.
- `setup.ca_trust` repair targets now route to the Trust tab.

## Cleanup

- No second CA action implementation was added.
- No background trust probe was added.
- No new dependency or generated artifact was introduced.

## Split-brain / race assessment

- Split-brain reduced: Desktop Trust tab, Desktop Help, CLI, and support bundle
  all consume the same snapshot/manifest sources.
- Race risk stays bounded: CA actions are existing serialized commands; the new
  tab does not run concurrent repair jobs.

## Verification

- `cargo fmt --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all-targets --features ui --quiet` (179 root tests + 5 UI/config tests)
- `cargo run --quiet --bin mhrv-f -- trust-center` (clean human output)
- `cargo run --quiet --bin mhrv-f -- trust-center --json` captured and parsed via `ConvertFrom-Json`
- `python tools/run-repo-sanity.py`
- `python tools/check-doc-links.py`
- `python tools/check-repo-cleanliness.py`
- `cargo clippy --all-targets -- -D warnings` in `tunnel-node`
- `cargo test --all-targets --quiet` in `tunnel-node` (34 tests)
- Removed `tools/__pycache__` after Python checks regenerated it.

## Remaining risk

- Android does not yet expose the Trust Center projection.
- Dedicated screenshot/UI regression coverage is still needed before deeper
  navigation reshaping.
