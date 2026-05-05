# Changelog — Strategic Batch 3: Trust Center foundation (docs + Help) (2026-05-03)

## Summary

| Field | Detail |
|--------|--------|
| **What** | Added **`docs/trust-center.md`** as the canonical hub for CA lifecycle, OS vs NSS, Android trust limits, APK signing pointers, diagnostics/redaction expectations, and UX contracts (serialized repair, stale results). Linked from **`docs/index.md`**, **`docs/safety-security.md`**, and Desktop **Help & docs** (`help_walkthrough` → `docs/trust-center.md`). |
| **Why** | Strategic Batch 3 starts from a single source of truth before adding panels/probes so Desktop, Android, CLI, and docs do not drift. |
| **Android** | Doc describes current limitations; no Kotlin change this slice. |
| **Desktop** | Help hyperlink only (`src/bin/ui.rs`). |

## Verification

`cargo fmt --check`; `cargo clippy --all-targets --all-features -- -D warnings`; `python tools/run-repo-sanity.py`.

## Remaining Batch 3 (product)

Live NSS trust detection, unified trust snapshot DTO, Trust Center UI region, support-bundle preview UI, explicit stale-async handling in UI trust flows.
