# Architecture Decision Records

This folder captures lightweight architecture decision records for choices that
affect multiple project surfaces: Rust runtime, Desktop UI, Android UI, backend
helpers, CI, docs, examples, release process, and support tooling.

The roadmap remains the running work log. ADRs are the durable rationale for
decisions that future contributors should not have to rediscover from long
batch notes.

## When To Add An ADR

Add or update an ADR when a change:

- chooses one source of truth over another;
- accepts a security, release, compatibility, or support tradeoff;
- intentionally keeps platform behavior different;
- changes migration boundaries for old config shapes;
- changes what is release-blocking versus advisory;
- rejects an attractive donor/upstream feature for licensing, security, or
  maintainability reasons.

Small implementation details do not need ADRs. They still need changelog and
roadmap entries when they are part of an implementation batch.

## Format

Use [`TEMPLATE.md`](TEMPLATE.md). Every ADR must include:

- `## Status`
- `## Context`
- `## Decision`
- `## Consequences`

Status values are intentionally plain text: `Accepted`, `Superseded`, or
`Proposed`. If an ADR is superseded, link the replacement in the status section.

## Current ADRs

| ADR | Decision |
|---|---|
| [ADR-0001](0001-committed-android-signing-material.md) | Keep committed Android signing material with explicit risk and rotation docs. |
| [ADR-0002](0002-release-artifacts-and-authority.md) | CI release workflow is authoritative; repository artifacts are labeled backup/archive material. |
| [ADR-0003](0003-android-config-preservation-and-simple-editor.md) | Android remains simple-first while preserving canonical and advanced config. |
| [ADR-0004](0004-legacy-config-migration-boundary.md) | Canonical config output uses `account_groups`; legacy root imports are narrow and documented. |
| [ADR-0005](0005-platform-defaults-are-documented-and-test-governed.md) | Intentional Desktop/Android default differences live in a generated, tested contract. |
| [ADR-0006](0006-canonical-status-and-doctor-contracts.md) | Status and Doctor data use shared Rust renderers with platform-specific projections. |
| [ADR-0007](0007-lightweight-governance-before-codeowners.md) | Use lightweight contributor/security/ownership docs before formal CODEOWNERS. |
| [ADR-0008](0008-no-stale-leftovers-cleanup-policy.md) | Every completed batch removes stale/deprecated leftovers unless explicitly supported. |

## Guard

`tools/check-adr-governance.py` checks the required ADR files, required
sections, README links, docs-index links, contributor guidance, tools docs, and
repo-sanity wiring. CI inherits that guard through `tools/run-repo-sanity.py`.
