# Changelog — Strategic Batch 2: donor absorption matrix (docs) (2026-05-03)

## Summary

| Field | Detail |
|--------|--------|
| **What changed** | Added **`docs/donor-absorption-matrix.md`** classifying donor roots (`mhr-cfw-main`, `Nova-Proxy-App-main`, `youtube-domain-fronting-patch-main`) with statuses **`port_now` / `port_concept` / `docs_only` / `reject` / `quarantine`**; linked from **`docs/index.md`** and **`Nova-Proxy-App-main/rules/README.md`**. |
| **Why** | Strategic Batch 2 requires an explicit absorption policy before borrowing UX or diagnostics ideas from donor trees; avoids half-porting binaries and GPL Cronet patches. |
| **Files** | `docs/donor-absorption-matrix.md`, `docs/index.md`, `Nova-Proxy-App-main/rules/README.md`, `elevation_audit_roadmap_source.md`, this changelog. |
| **Desktop / Android / Backend** | No code changes. |
| **Docs impact** | Canonical donor classification entry point; cross-links to **`cfw-reference-audit.md`** and mode docs. |
| **Config/schema** | None. |
| **Tests run** | **`python tools/run-repo-sanity.py`** (includes Markdown link + anchor checks). |
| **Cleanup** | None. |
| **Split-brain** | Matrix references maintained Worker/Apps Script paths as source of truth vs donor copies (**`reject`** as competing implementations). |
| **Race/async** | N/A. |
| **Remaining risk** | Contributors may still skim donor folders for copy-paste; hygiene skips those trees — rely on reviews + this matrix. |
| **Progress** | Strategic Batch **2 ~35%**; rolled strategic program **~17–20%**. |

## Note on batch numbering

Maintainer changelog **`batch-2-2026-05-03.md`** documents the **ConfigWire ↔ registry** CI gate. This file is **strategic Batch 2** (donor program). See roadmap **Naming clarity** bullet.
