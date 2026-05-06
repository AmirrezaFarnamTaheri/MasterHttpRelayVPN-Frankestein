# ADR-0002: Release Artifacts And Authority

## Status

Accepted

## Context

The repository may contain `dist/` or `releases/` material for offline fallback,
backup, or historical reference. At the same time, generated binaries and
archives can bloat the source tree, confuse users, and create split-brain about
which files are official.

This affects CI, release checklist, changelog, fallback download docs,
repository cleanliness, and support instructions.

## Decision

The CI release workflow is the authoritative path for official release
artifacts. Repository-local release artifacts may remain only when clearly
labeled as backup/archive material and must not become the primary release
source.

## Consequences

- `.github/workflows/release.yml` and the release checklist remain the release
  source of truth.
- `releases/README.md` may describe offline or blocked-GitHub fallback use, but
  must not replace official release workflow instructions.
- `tools/check-repo-cleanliness.py` may allow labeled backup/archive folders but
  should still prune build outputs, caches, and accidental generated artifacts.
- Changelog and release notes must describe artifacts through official release
  channels and checksums.
- Any new artifact path needs docs, cleanup policy, and sanity guard updates.

## Follow-Up

Keep `CHANGELOG.md`, `docs/release-checklist.md`, `docs/versioning-policy.md`,
`docs/rollback-policy.md`, and `tools/check-release-governance.py` aligned.
