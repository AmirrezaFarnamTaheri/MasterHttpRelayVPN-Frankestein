# Changelog — Strategic Batches 0–2 perfected (scoped acceptance) (2026-05-03)

This entry marks **completion** of the **scoped** Batch **0**, **1**, and **2** narratives used in `elevation_audit_roadmap_source.md` **Program status**. It does **not** imply downstream batches (Trust Center UI, Route Advisor implementation, Observatory, etc.) are shipped — only prerequisites and absorption policy/tooling.

---

## Summary

| Field | Detail |
|--------|--------|
| **What changed** | **`docs/android-config-preservation.md`**; Android guide + docs hub links; **`tools/report-nova-proxy-config.py`** + **`run-repo-sanity.py`** integration; **`check-repo-cleanliness.py`** prunes **`.cargo`**; **`docs/workspace-cleanup.md`** PowerShell note + **`run-repo-sanity`** verify path; **`docs/donor-absorption-matrix.md`** references report tool; **`tools/README.md`** documents report CLI. |
| **Why** | Close remaining gaps vs Batch **1** unknown-field narration and Batch **2** “rule/report tool” expectation; tighten Batch **0** hygiene parity (cargo cache dirs). |
| **Batch 0 impact** | Cleaner repo walks when `.cargo` exists under tree; workspace doc aligns with release-checklist shells. |
| **Batch 1 impact** | Explicit **`ownedKeys`** / preservation policy doc + discoverability from **`docs/android.md`** / **`docs/index.md`**. |
| **Batch 2 impact** | Actionable **`--path`** migration triage alongside classification matrix. |

---

## Files touched

- `tools/report-nova-proxy-config.py` (new)
- `tools/run-repo-sanity.py`
- `tools/check-repo-cleanliness.py`
- `tools/README.md`
- `docs/android-config-preservation.md` (new)
- `docs/android.md`
- `docs/index.md`
- `docs/workspace-cleanup.md`
- `docs/donor-absorption-matrix.md`
- `elevation_audit_roadmap_source.md`
- `docs/changelog/batches-0-1-2-perfected-2026-05-03.md`

---

## Verification run (maintainer)

- `python -m py_compile tools/report-nova-proxy-config.py`
- `python tools/run-repo-sanity.py`
- `cargo fmt --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all-targets --features ui`
- `cd tunnel-node && cargo clippy --all-targets -- -D warnings && cargo test --all-targets`

---

## Split-brain / risk

- **Report tool** is informational (exit 0) — it does not gate schema correctness alone.
- **Strategic vs maintainer changelog “Batch 2”** naming: ConfigWire gate remains **`batch-2-2026-05-03.md`**; donor program is **`batch-2-strategic-donor-absorption-2026-05-03.md`** + this file.

---

## Progress

Strategic roadmap: **Batches 0–2 done** at scoped definition; rolled **~27%** over eleven batches.
