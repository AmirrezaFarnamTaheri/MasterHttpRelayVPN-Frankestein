# Changelog — Strategic Batch 2 completion (2026-05-03)

Closes remaining **Batch 2** scope from the donor-absorption narrative:

## Delivered

| Item | Detail |
|------|--------|
| **GPL YouTube / Cronet docs-only path** | New **`docs/youtube-external-patching.md`** (risks, license, why not vendored); linked from **`docs/index.md`**, **`docs/relay-modes.md`**, and **`docs/donor-absorption-matrix.md`**. |
| **Donor tree pointers** | **`mhr-cfw-main/DONOR_REFERENCE.md`**, **`Nova-Proxy-App-main/DONOR_REFERENCE.md`**, **`youtube-domain-fronting-patch-main/DONOR_REFERENCE.md`** → canonical docs (hygiene still skips descending donor dirs). |
| **Nova report tool** | **`tools/report-nova-proxy-config.py`**: nested object/array preview at root; **`--no-nested`**; docstring exit-code clarification. **`tools/README.md`** example updated. |
| **Matrix bookkeeping** | **`docs/donor-absorption-matrix.md`**: donor-relative links + last-reviewed note. |

## Files

`docs/youtube-external-patching.md`, `docs/relay-modes.md`, `docs/index.md`, `docs/donor-absorption-matrix.md`, `mhr-cfw-main/DONOR_REFERENCE.md`, `Nova-Proxy-App-main/DONOR_REFERENCE.md`, `youtube-domain-fronting-patch-main/DONOR_REFERENCE.md`, `tools/report-nova-proxy-config.py`, `tools/README.md`, `elevation_audit_roadmap_source.md`, this changelog.

## Verification

`python -m py_compile tools/report-nova-proxy-config.py` and **`python tools/run-repo-sanity.py`**.

## Explicitly out of scope (later batches)

- Route Advisor / Trust Center / Observatory **implementations** (classified as **`port_concept`** in matrix).
- Batch **9** YouTube strategy **wizard** UI.
