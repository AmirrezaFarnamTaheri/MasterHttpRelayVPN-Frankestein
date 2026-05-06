# Batch 6 - Release Governance Pack - 2026-05-06

## Summary

Added a release/changelog governance pack so release notes, batch audit logs,
versioning decisions, rollback policy, and repo-sanity enforcement no longer
live as loose conventions.

## Changed

- Added top-level `CHANGELOG.md` as the human-facing release-notes hub.
- Added `docs/changelog/TEMPLATE.md` with required sections for future batch
  notes.
- Added `docs/versioning-policy.md`.
- Added `docs/rollback-policy.md`.
- Added `tools/check-release-governance.py`.
- Wired the release-governance guard into `tools/run-repo-sanity.py`.
- Added the guard to `tools/check-ci-local-sanity-parity.py`.
- Updated:
  - `docs/release-checklist.md`;
  - `docs/index.md`;
  - `docs/changelog/README.md`;
  - `docs/RELEASE_NOTES.md`;
  - `tools/README.md`.

## UI / UX

N/A. This is release/process/documentation governance.

## Config / Schema

N/A. No runtime config or JSON schema changed.

## Backend Helpers

N/A. No Apps Script, Cloudflare Worker, Vercel/Netlify, tunnel-node, or exit
node behavior changed.

## Security / Trust

Rollback policy now explicitly covers bad Desktop/CLI releases, Android
releases, config migrations, backend helpers, and tunnel-node releases while
keeping GitHub Release as canonical and Telegram as an optional mirror.

## Docs

Docs now state the hierarchy:

- GitHub Release is canonical for shipped artifacts and checksums.
- `docs/changelog/v<version>.md` is preferred per-tag release-note source when
  present.
- `docs/RELEASE_NOTES.md` is rolling unreleased user-facing staging.
- `docs/changelog/batch-*.md` is the maintainer audit trail.

## Breaking / Cleanup

No compatibility branch was added. The old implicit release-note convention is
replaced by explicit docs plus a guard.

## Parity

Desktop, Android, CLI, backend helpers, docs, and release artifacts now share a
single release-governance checklist rather than separate implicit expectations.

## Race / Split-Brain Review

The guard checks that `CHANGELOG.md`, rolling release notes, batch template,
release checklist, versioning policy, rollback policy, docs index, tools docs,
and repo-sanity wiring remain connected.

## Verification

- `python tools/check-release-governance.py`
- `python tools/check-ci-local-sanity-parity.py`
- `python tools/check-doc-links.py`

## Cleanup

No generated caches or release artifacts were created by the new governance
files.
