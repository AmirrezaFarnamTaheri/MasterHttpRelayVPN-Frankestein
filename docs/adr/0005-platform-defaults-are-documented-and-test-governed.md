# ADR-0005: Platform Defaults Are Documented And Test-Governed

## Status

Accepted

## Context

Desktop/Rust and Android intentionally differ on a few defaults, including
local proxy ports and some mobile starter values. Without an explicit contract,
these differences look like drift and invite accidental "fixes" that can break
existing users or platform ergonomics.

## Decision

Keep intentional platform default differences only when they are documented in
`docs/platform-defaults.json`, rendered into `docs/platform-defaults.md`, and
covered by static or executable tests. Shared defaults must remain shared.

## Consequences

- Platform-specific defaults are allowed, but only as explicit contract rows.
- Rust, Android, docs, examples, and tests must agree on whether a default is
  shared or intentionally different.
- Future default changes must update generated docs and guards in the same
  batch.
- Contributors should not "normalize" Android/Desktop differences without
  checking the platform-defaults contract first.

## Follow-Up

Keep `tools/check-platform-defaults.py`,
`tools/generate-platform-defaults-doc.py`, and the Android static/JVM platform
default tests aligned with `docs/platform-defaults.json`.
