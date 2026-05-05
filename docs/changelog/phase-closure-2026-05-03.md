# Changelog — Phase closure / roadmap reconciliation (2026-05-03)

Maintainer-facing closure record aligned with the **global phase ritual**: verify tooling, reconcile backlog vs CI truth, avoid split-brain counts, update roadmap metrics, record residual risk.

---

## Summary

| Field | Detail |
|--------|--------|
| **What changed** | Full verification rerun on Windows; `elevation_audit_roadmap_source.md` backlog + P0 table reconciled with current gates; Program status refreshed with internal spine %, strategic Batch 0–10 mapping, naming clarification for changelog vs roadmap batch IDs; stale BATCH-1 progress sentence redirected to Program status. |
| **Why** | Prior roadmap rows still cited obsolete Android string counts (150/138), open BL items already covered by CI, and unresolved P0.1–P0.3 wording from pre-parity era — undermining roadmap trust. |
| **Files changed** | `elevation_audit_roadmap_source.md`, `docs/changelog/phase-closure-2026-05-03.md` |
| **Desktop impact** | None (verification only). |
| **Android impact** | None (verification only). |
| **Backend impact** | None. |
| **Docs impact** | Roadmap is authoritative progress narrative again; strategic batches (user Batch 0–10) mapped explicitly. |
| **Config/schema impact** | None. |

---

## Verification performed

| Check | Command / gate | Result |
|--------|----------------|--------|
| Format | `cargo fmt --check` | ok |
| Root Clippy | `cargo clippy --all-targets --all-features -- -D warnings` | ok |
| Root tests | `cargo test --all-targets --features ui --quiet` | ok (167 + integration subsets as emitted) |
| tunnel-node Clippy | `cd tunnel-node && cargo clippy --all-targets -- -D warnings` | ok |
| tunnel-node tests | `cd tunnel-node && cargo test --all-targets --quiet` | ok (34 tests) |
| Repo sanity | `python tools/run-repo-sanity.py` | ok (full suite per script) |

---

## Roadmap / backlog reconciliation

- **Second-pass audit table:** Android string row updated — **199/199** EN/FA keys enforced; historical 150/138 marked obsolete.
- **P0.1 – P0.3:** Marked **mitigated** with pointer to internal batches + static gates (`check-config-wire-vs-registry.py`).
- **P0.10:** Updated to reflect present JVM tests + CI job; instrumentation still thin.
- **BL.2, BL.16, BL.20, BL.24, BL.31:** Status set to **done** with evidence paths.
- **BL.25, BL.26, BL.39:** Set to **in_progress** with explicit “green vs remaining” bullets.
- **Program status:** ~**26%** internal elevation spine; **~15–18%** rolled strategic (Batch 0–10 narrative); Batch 1 ~**70%**; naming note for changelog Batch 2/3 vs roadmap BATCH-2/3.

---

## Cleanup performed

- No generated junk added; no stale artifacts removed from disk (audit-only pass).

---

## Split-brain / race assessment

- **Split-brain:** Documented residual duplication — Android allowlists shared informally between `check-android-config-keys.py` and `check-android-owned-keys-list.py` (must stay manually synced until factored).
- **Race/async:** Not applicable to this documentation-only closure.

---

## Legacy / deprecated

- None removed this step.

---

## Remaining risk (prioritized)

1. **DRY debt:** Factor shared Android-only / legacy key allowlists for config drift scripts.
2. **Coverage gap:** Android instrumentation / UI tests still lighter than Rust/Desktop for large UI refactors.
3. **Docs drift tooling:** Explicit stale-version / product-name scans not yet comprehensive beyond substring/image gates.
4. **Strategic batches 2–10:** Product programs (Trust Center, Route Advisor, donor matrix, etc.) **not started** under that naming — roadmap now states this plainly.

---

## Progress estimate

- **Phase closure work:** 100% (this entry).
- **User strategic roadmap Batches 0–10:** ~**15–18%** aggregate (Batch 0 done; Batch 1 ~70%; others not started as programs).
