# ADR-0003: Android Config Preservation And Simple Editor

## Status

Accepted

## Context

Desktop is the richer expert editor. Android must be practical on a phone, but
it must not corrupt Desktop-authored or hand-written advanced config. The most
important example is Apps Script account groups: users need a simple mobile
setup path, while advanced multi-group configs should round-trip safely.

This affects Android `ConfigStore.kt`, QR/deep-link sharing, Desktop export,
docs, config registry, parity matrix, and support snapshots.

## Decision

Android uses a simple first-group editor for normal mobile setup while reading,
writing, and preserving canonical config. Advanced unknown root fields and
multi-group config are preserved unless Android explicitly owns and rewrites
that field.

## Consequences

- Android must not silently drop imported advanced Desktop config.
- Android-owned keys must be explicit and guarded.
- Config sharing must preserve unknown root JSON through QR/deep-link export.
- Desktop remains the full expert editor unless a future Android advanced editor
  is deliberately designed.
- Config registry, parity matrix, Android docs, and Android support snapshot
  docs must describe the preservation boundary.
- Static guards must fail if Android key ownership or preservation behavior
  drifts.

## Follow-Up

Continue using `tools/check-android-config-keys.py`,
`tools/check-android-owned-keys-list.py`, and
`tools/check-android-config-sharing.py` as the non-Gradle drift gates.
