# Batch 7 Desktop UI XHTTP Renderer Extraction - 2026-05-06

## Summary

- Moved the stateful XHTTP VLESS generator renderer from `src/bin/ui.rs` into
  `src/bin/ui_xhttp.rs`.
- Moved the XHTTP cloud deploy pipe and deploy-result polling logic into
  `src/bin/ui_xhttp.rs`.
- Removed direct `xhttp_cloud_deploy` and `XhttpDeployWorkerMsg` ownership from
  the Desktop UI monolith.
- Extended `tools/check-desktop-ui-modularization.py` so XHTTP renderer,
  deploy-pipe, deploy-worker, and cloud-deploy calls cannot drift back into
  `src/bin/ui.rs`.
- Updated tooling-source-map docs and tools documentation for the richer XHTTP
  module boundary.

## Parity

- Desktop behavior is intended to be unchanged: the same Setup/XHTTP panel,
  VLESS link generation, manual deploy notes, API deploy buttons, deploy log,
  token clearing, and relay-host copy behavior are now served by
  `ui_xhttp.rs`.
- Android, backend helper scripts, config schema, examples, readiness,
  Doctor/status, support bundles, runtime proxy lifecycle, tunnel-node, and
  release workflows are not changed by this slice.
- The cloud-deploy token policy remains the same: tokens are held only in RAM
  and are never saved to `config.json`.

## Cleanup

- Removed the XHTTP renderer function from `src/bin/ui.rs`.
- Removed the XHTTP deploy-pipe struct from `src/bin/ui.rs`.
- Removed direct XHTTP cloud deploy worker imports from `src/bin/ui.rs`.
- Left the top-level app update loop in `src/bin/ui.rs`; it now delegates XHTTP
  cloud-deploy polling to `ui_xhttp.rs`.

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
