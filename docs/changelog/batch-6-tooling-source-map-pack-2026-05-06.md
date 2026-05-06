# Batch 6 Tooling Source Map Pack - 2026-05-06

## UI / UX

- No product UI behavior changed.
- Maintainer UX improved by adding a source map that answers which generator or
  guard protects high-risk contract docs.

## Config / Schema

- Added `docs/tooling-source-map.json` with schema marker
  `mhrv-f-tooling-source-map/v1`.
- Added `docs/tooling-source-map.md` as the human-readable projection.
- Mapped generated config, parity, platform-default, readiness, Android copy,
  and changelog docs to their sources and guards.

## Backend Helpers

- Backend/helper docs are not changed behaviorally.
- The map now reinforces mode/backend contract ownership through the existing
  parity and helper hardening gates.

## Security / Trust

- Mapped release checklist, ownership notes, Android support snapshot, ADRs,
  status/Doctor contracts, and governance docs to their guards.
- This makes security/release/support documentation less dependent on memory.

## Breaking / Cleanup

- No compatibility behavior changed.
- No new generated build artifacts or local build requirements were introduced.

## Parity

- Added `tools/check-tooling-source-map.py`.
- Wired the guard into:
  - `tools/run-repo-sanity.py`;
  - `tools/check-ci-local-sanity-parity.py`.
- Linked the source map from:
  - `docs/index.md`;
  - `CONTRIBUTING.md`;
  - `tools/README.md`.

## Race / Split-Brain Review

- The guard prevents split-brain by requiring:
  - stable mapped document paths and order;
  - every mapped document, source, generator, and guard to exist;
  - docs/contributor/tools discoverability;
  - repo-sanity and CI/local parity wiring.

## Verification

- `python tools/check-tooling-source-map.py`
- `python tools/check-doc-links.py`
- `python tools/check-ci-local-sanity-parity.py`
- `python tools/check-change-impact-checklist.py`

Full repo-sanity and cleanup were run as part of batch closeout.
