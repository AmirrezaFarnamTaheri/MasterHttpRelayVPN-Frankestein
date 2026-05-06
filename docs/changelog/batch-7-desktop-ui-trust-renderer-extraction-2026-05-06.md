# Batch 7 Desktop UI Trust Renderer Extraction - 2026-05-06

## Summary

- Added `src/bin/ui_trust.rs` for read-only Trust Center rendering.
- Moved Trust Center snapshot rendering out of `src/bin/ui.rs`.
- Moved support-bundle preview rendering out of `src/bin/ui.rs`.
- Left Trust tab command buttons in `src/bin/ui.rs` for now because they send
  private app `Cmd` values and must remain tied to the main app command channel.
- Extended `tools/check-desktop-ui-modularization.py` so direct trust snapshot
  and support-manifest rendering cannot drift back into `src/bin/ui.rs`.
- Updated tooling source-map documentation for the Trust renderer boundary.

## Parity

- Desktop behavior is intended to be unchanged: the Trust tab still shows the
  same CA state, Firefox/NSS state, Chrome NSS state, signing policy,
  support-bundle manifest, redaction summary, docs links, and CA action buttons.
- Android, backend helpers, config schema, examples, readiness, Doctor/status,
  runtime proxy lifecycle, tunnel-node, and release workflows are not changed
  by this slice.
- The renderer now receives a config validation result from `ui.rs`, avoiding a
  new dependency on private form state.

## Cleanup

- Removed Trust status label helpers from `src/bin/ui.rs`.
- Removed direct `trust_center::snapshot` and `support_bundle::preview_manifest`
  calls from `src/bin/ui.rs`.
- Kept app-command dispatch in `src/bin/ui.rs` to avoid leaking command-channel
  ownership into a renderer module.

## Verification

- `cargo fmt --check`
- `cargo check --features ui --bin mhrv-f-ui`
- `cargo test --features ui --bin mhrv-f-ui ui_trust`
- `python tools/check-desktop-ui-modularization.py`
- `python tools/check-tooling-source-map.py`
- `python tools/check-doc-links.py`
- `python tools/check-doc-anchors.py`
- `python tools/check-changelog-headings.py`
- `python tools/generate-changelog-index.py -Check`
- `python tools/check-ci-local-sanity-parity.py`
- `python tools/run-repo-sanity.py --skip-node`
- `python tools/check-repo-cleanliness.py`
