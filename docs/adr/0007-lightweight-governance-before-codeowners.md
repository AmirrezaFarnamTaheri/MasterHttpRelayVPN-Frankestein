# ADR-0007: Lightweight Governance Before CODEOWNERS

## Status

Accepted

## Context

The project spans Rust runtime, Desktop UI, Android, backend helpers, release
workflow, docs, and security policy. Formal CODEOWNERS can improve review
quality, but only when maintainers are available for each area. Adding formal
ownership too early can become noisy or misleading.

## Decision

Use lightweight governance first: `CONTRIBUTING.md`, `SECURITY.md`,
`docs/ownership.md`, PR/issue templates, release governance docs, and static
guards. Defer formal CODEOWNERS until there are maintainers who can honor those
review boundaries.

## Consequences

- Contributors get clear expectations without blocked PR routing.
- Area ownership is documented as guidance rather than an enforced GitHub rule.
- Security and release-sensitive changes still have explicit review checklists.
- A future CODEOWNERS file should update this ADR, `docs/ownership.md`,
  contributing docs, and repo governance guards.

## Follow-Up

Keep `tools/check-repo-governance.py` as the guard for contributor/security
surfaces until a formal CODEOWNERS policy exists.
