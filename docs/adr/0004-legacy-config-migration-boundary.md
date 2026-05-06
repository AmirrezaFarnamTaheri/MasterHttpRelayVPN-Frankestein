# ADR-0004: Legacy Config Migration Boundary

## Status

Accepted

## Context

Older Android and upstream variants used top-level `script_ids` and `auth_key`.
The canonical Rust config now uses `account_groups`. Removing all legacy import
support would keep internals cleaner, but it would strand existing configs and
QR/deep-link payloads.

The project also has a standing cleanup rule: old internal shapes should not be
kept indefinitely unless they are explicit, documented compatibility surfaces.

## Decision

Canonical output remains `account_groups`. Legacy top-level `script_ids` and
`auth_key` are accepted only as narrow import/migration compatibility. New save
paths should write canonical config and should not expand the legacy surface.

## Consequences

- Rust and Android can accept old configs, but generated examples and new
  exports should use `account_groups`.
- Compatibility logic must be covered by tests or static guards and documented
  as migration-only.
- UI should avoid teaching the legacy keys as active configuration.
- Future cleanup may remove legacy acceptance only through an explicit roadmap,
  changelog, and ADR update.
- Any new legacy alias must be treated as a compatibility decision, not a casual
  parser convenience.

## Follow-Up

Keep `docs/config-registry.json`, `docs/config-registry.md`,
`docs/config-parity-matrix.md`, Android config guards, and Rust config tests in
sync.
