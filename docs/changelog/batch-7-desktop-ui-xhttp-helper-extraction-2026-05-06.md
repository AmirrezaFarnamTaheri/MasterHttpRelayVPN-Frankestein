# Batch 7 Desktop UI XHTTP Helper Extraction - 2026-05-06

## Summary

- Extracted the Desktop XHTTP generator's leaf logic into `src/bin/ui_xhttp.rs`.
- Moved XHTTP candidate lists, generator form state/defaults, host/path
  normalization, VLESS-link generation, and deploy-note generation out of
  `src/bin/ui.rs`.
- Kept the stateful egui XHTTP renderer in `src/bin/ui.rs` for now because it
  still depends on shared styling helpers and deploy-worker plumbing.
- Extended `tools/check-desktop-ui-modularization.py` so the extracted XHTTP
  helpers cannot drift back into the monolith.
- Updated the tooling source map and local sanity wiring documentation for the
  new Desktop UI boundary.

## Parity

- Desktop behavior is intended to be unchanged: the same Setup/XHTTP UI calls
  the same generation actions, but the pure helper logic now lives behind a
  tested module boundary.
- Android, backend helpers, config schema, examples, readiness, Doctor/status,
  support bundles, release workflows, and runtime proxy lifecycle are not
  changed by this slice.
- The boundary is documented in `tools/README.md`,
  `docs/tooling-source-map.json`, and `docs/tooling-source-map.md`.

## Cleanup

- Removed the XHTTP candidate constants, form/default implementation, and pure
  generator helpers from `src/bin/ui.rs`.
- Left no duplicate XHTTP helper implementation in Desktop UI.
- Deferred the stateful XHTTP renderer extraction until the shared egui styling
  helpers and deploy-worker pipe are split into cleaner modules.

## Verification

- `cargo fmt --check`
- `cargo check --features ui --bin mhrv-f-ui`
- `cargo test --features ui --bin mhrv-f-ui ui_xhttp`
- `python tools/check-desktop-ui-modularization.py`
- `python tools/check-tooling-source-map.py`
- `python tools/check-doc-links.py`
- `python tools/check-doc-anchors.py`
- `python tools/check-changelog-headings.py`
- `python tools/generate-changelog-index.py -Check`
- `python tools/check-ci-local-sanity-parity.py`
- `python tools/run-repo-sanity.py --skip-node`
- `python tools/check-repo-cleanliness.py`
