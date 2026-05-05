# Changelog — Batch 1 follow-up: Android drift allowlists DRY (2026-05-03)

## Summary

| Field | Detail |
|--------|--------|
| **What changed** | Introduced **`tools/android_config_allowlists.py`** with **`ANDROID_ONLY_KEYS`**, **`LEGACY_KEYS`**, and **`NESTED_KEYS`**; **`check-android-config-keys.py`** and **`check-android-owned-keys-list.py`** import from it instead of duplicating sets. |
| **Why** | Roadmap flagged split-brain risk when two drift gates maintained identical allowlists by hand. |
| **Files** | `tools/android_config_allowlists.py` (new), `tools/check-android-config-keys.py`, `tools/check-android-owned-keys-list.py`, `tools/README.md`, `elevation_audit_roadmap_source.md`, this changelog. |
| **Desktop / Android / Backend** | No runtime behavior change; tooling only. |
| **Docs** | `tools/README.md` documents the shared module. |
| **Config/schema** | None. |
| **Tests run** | `python -m py_compile` on touched scripts; **`python tools/run-repo-sanity.py`**. |
| **Cleanup** | Removed duplicated inline sets from both check scripts. |
| **Split-brain** | Reduced — one module owns Android-only / legacy buckets; **`NESTED_KEYS`** remains scoped to the literal-scan script only by design. |
| **Race/async** | N/A. |
| **Remaining risk** | Low; future keys still require deliberate edits to **`android_config_allowlists.py`** and registry when appropriate. |
| **Progress** | Strategic Batch **1** estimate **~72%** (was ~70%); rolled strategic **~15–18%** unchanged at rounding precision. |
