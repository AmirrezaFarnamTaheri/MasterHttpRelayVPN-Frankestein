# Batch 6 Change Impact Checklist Pack - 2026-05-06

## UI / UX

- No product UI behavior changed.
- Contributor UX improved: PRs and local workflows now have a direct bridge
  from touched surface to expected verification profile, parity review, and
  cleanup duties.

## Config / Schema

- Added `docs/change-impact-checklist.json` with schema marker
  `mhrv-f-change-impact-checklist/v1`.
- Added `docs/change-impact-checklist.md` as the human-readable surface matrix.

## Backend Helpers

- Backend helper changes now map explicitly to the `backend_helpers` profile
  and to helper marker, backend registry, mode docs, examples, and release
  checklist parity.

## Security / Trust

- Release/security/trust changes now map explicitly to docs governance plus
  release-ready verification, with release workflow, signing docs, security
  policy, and rollback docs listed as parity surfaces.

## Breaking / Cleanup

- No compatibility behavior changed.
- The checklist makes cleanup expectations explicit for every surface, including
  stale docs, deprecated field docs, hard-coded Android copy, stale helper
  variants, and duplicate release notification paths.

## Parity

- Added `tools/check-change-impact-checklist.py`.
- Wired the guard into:
  - `tools/run-repo-sanity.py`;
  - `tools/check-ci-local-sanity-parity.py`.
- Linked the checklist from:
  - `docs/index.md`;
  - `CONTRIBUTING.md`;
  - `.github/pull_request_template.md`;
  - `tools/README.md`.

## Race / Split-Brain Review

- The guard prevents split-brain by requiring:
  - stable surface IDs and order;
  - every referenced verification profile to exist;
  - documentation links from docs index, contributing guide, PR template, and
    tools docs;
  - repo-sanity and CI/local parity wiring.

## Verification

- `python tools/check-change-impact-checklist.py`
- `python tools/check-verification-profiles.py`
- `python tools/check-doc-links.py`
- `python tools/check-ci-local-sanity-parity.py`

Full repo-sanity and cleanup were run as part of batch closeout.
