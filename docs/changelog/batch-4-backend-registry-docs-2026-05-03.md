# Changelog — Strategic Batch 4: Backend Registry foundation (docs + Help) (2026-05-03)

## Summary

| Field | Detail |
|--------|--------|
| **What** | Added **`docs/backend-registry.md`**: native backend rows (Apps Script variants, Worker exit, serverless JSON, tunnel-node), **`HELPER_KIND`** / compat probe alignment, adjacent XHTTP tools table, **unified health result** design target, failure-mode cheat-sheet. Linked from **`docs/index.md`** (Backend Guides), **`docs/trust-center.md`**, **`docs/donor-absorption-matrix.md`**; Desktop Help **`help_walkthrough`** → **`docs/backend-registry.md`**. |
| **Why** | Batch 4 requires one canonical deploy/probe narrative before Rust/UI dedupe work. |
| **Rust/runtime** | None this slice (design-only JSON shape documented). |

## Verification

`cargo fmt --check`; `cargo clippy --all-targets --all-features -- -D warnings`; `python tools/run-repo-sanity.py`.

## Files

`docs/backend-registry.md`, `docs/index.md`, `docs/trust-center.md`, `docs/donor-absorption-matrix.md`, `src/bin/ui.rs`, `elevation_audit_roadmap_source.md`, this changelog.
