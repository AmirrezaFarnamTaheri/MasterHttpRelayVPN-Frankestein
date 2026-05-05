# Changelog — Donor absorption matrix deepening (2026-05-03)

## Summary

Expanded **`docs/donor-absorption-matrix.md`** into a multi-layer reference:

- At-a-glance summary (license, hygiene, relation to product).
- Expanded policy (MIT Nova caveat, GPL default).
- Operational tooling matrix.
- **`mhr-cfw-main`**: path inventory table, detailed migration matrix, non-goals (aligned with mirror **`src/`** layout vs audit **`core/`** note).
- **`Nova-Proxy-App-main`**: subsystem inventory (`proxy/`, `sysproxy/`, `sni-server/`, etc.), capability→absorption matrix, settings.json structural notes.
- **`youtube-domain-fronting-patch-main`**: artifact/license matrix, multi-factor risk table, supported-alternatives pointer table.
- Cross-cutting binary policy, split-brain canonical pointers, maintainer checklist.
- Appendices: hygiene skip roots, related docs index.

## Verification

`python tools/check-doc-links.py`; `python tools/check-doc-anchors.py`.

## Files

`docs/donor-absorption-matrix.md`, this changelog.
