# Batch 7 Desktop UI Help Backend Catalog Extraction - 2026-05-07

## Summary

- Added `src/bin/ui_help.rs` as the Desktop Help module for backend-tool catalog
  data and row rendering.
- Moved the **Backend tools and deployment recipes** rows from repeated inline
  calls in `src/bin/ui.rs` into one `ToolHelpEntry` catalog rendered through a
  single loop.
- Kept the visible tool list and copy unchanged: Apps Script Code.gs,
  Cloudflare Worker exit, Vercel/Netlify JSON, Vercel/Netlify XHTTP helpers,
  Field notes, and tunnel-node remain present.
- Added focused tests that lock the backend-tool catalog order and require each
  row to keep a local path.

## Parity

- Desktop Help still presents the same backend taxonomy and local resource
  links; only ownership moved from the monolith to `ui_help.rs`.
- Android, backend helpers, config schema, Apps Script helpers, tunnel-node,
  and runtime behavior are unchanged.
- The tooling source map now records `src/bin/ui_help.rs` as the guarded source
  for this Desktop UI boundary.

## Cleanup

- Removed the inline `tool_help_row` helper from `src/bin/ui.rs`.
- Removed the now-unused `open_local_resource` import from `src/bin/ui.rs`; the
  Help module owns that action.
- Extended `tools/check-desktop-ui-modularization.py` so future edits cannot
  reintroduce the backend-tool row renderer into the monolith or lose the new
  catalog tests.
- Updated `tools/check-cloudflare-worker-relay.py` so the existing Cloudflare
  Worker bridge guard follows the extracted `src/bin/ui_help.rs` catalog
  instead of requiring the Worker helper row to remain inline in `src/bin/ui.rs`.

## Verification

- `cargo fmt --check`
- `cargo check --features ui --bin mhrv-f-ui`
- `cargo test --features ui --bin mhrv-f-ui ui_help`
- `python tools/check-desktop-ui-modularization.py`
- `python tools/check-tooling-source-map.py`
- `python tools/check-doc-links.py`
- `python tools/check-doc-anchors.py`
- `python tools/check-changelog-headings.py`
- `python tools/generate-changelog-index.py -Check`
- `python tools/check-ci-local-sanity-parity.py`
- `python tools/run-repo-sanity.py --skip-node`
- `python tools/check-repo-cleanliness.py`
