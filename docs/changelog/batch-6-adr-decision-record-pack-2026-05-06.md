# Batch 6 ADR Decision Record Pack - 2026-05-06

## UI / UX

- No runtime UI behavior changed.
- Contributor-facing navigation improved by adding a documented ADR hub for
  decisions that affect Desktop, Android, backend helpers, release process,
  security posture, and cleanup rules.

## Config / Schema

- Added ADRs for Android config preservation and the legacy config migration
  boundary.
- Reaffirmed that canonical config output uses `account_groups`, while legacy
  top-level `script_ids` and `auth_key` remain narrow import/migration
  compatibility only.

## Backend Helpers

- No backend helper code changed.
- ADR cleanup policy now explicitly covers donor/backend helper absorption:
  port deliberately, document rejected paths, and avoid stale copied helpers.

## Security / Trust

- Added an ADR for committed Android signing material, including install-over
  continuity, risk, and rotation expectations.
- Added ADRs for release artifact authority and lightweight governance before
  formal CODEOWNERS.

## Breaking / Cleanup

- Added `docs/adr/` with a template and eight seed ADRs.
- No stale legacy docs were kept as a competing decision log; the roadmap
  remains the work log, and ADRs are now the durable rationale layer.

## Parity

- Linked ADRs from `docs/index.md`, `CONTRIBUTING.md`, and `tools/README.md`.
- Added `tools/check-adr-governance.py` and wired it into:
  - `tools/run-repo-sanity.py`;
  - `tools/check-ci-local-sanity-parity.py`.

## Race / Split-Brain Review

- The new guard prevents split-brain by requiring:
  - every seed ADR file to exist;
  - required sections in each ADR;
  - README links for every seed ADR;
  - docs/contributor/tools discoverability;
  - repo-sanity and CI/local parity wiring.

## Verification

- `python tools/check-adr-governance.py`
- `python tools/check-ci-local-sanity-parity.py`
- `python tools/check-doc-links.py`

Full repo-sanity and cleanup were run as part of the batch closeout.
