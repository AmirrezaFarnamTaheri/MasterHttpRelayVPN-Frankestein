# ADR-0001: Committed Android Signing Material

## Status

Accepted

## Context

Android users need install-over continuity between releases. The repository
currently keeps release signing material in the source tree by maintainer
decision. Moving the keystore exclusively to private CI secrets would improve
secret hygiene, but it would also create a new install lineage unless the same
key were migrated safely.

This choice affects Android builds, release workflow, support docs, incident
response, and contributor expectations.

## Decision

Keep the committed Android signing material as an explicit project policy for
the current release lineage. Treat CI/release workflow as the source of truth
for official builds, and document the risks, rotation path, recovery steps, and
support implications in `docs/android-signing.md`.

## Consequences

- The committed keystore is intentional, not an accidental leaked secret.
- Release docs must keep warning that anyone with the source can inspect the
  signing material.
- Rotation requires a documented compatibility break or migration campaign.
- CI, release notes, and support docs must identify official artifacts by
  release workflow, checksums, and project channels, not by secrecy of the key.
- Future changes to Android signing must update this ADR, `docs/android-signing.md`,
  `docs/release-checklist.md`, `SECURITY.md`, and changelog notes together.

## Follow-Up

Keep `tools/check-release-governance.py` and repo-sanity checks linked to the
signing policy docs so the policy cannot disappear silently.
