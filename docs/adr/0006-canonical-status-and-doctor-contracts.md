# ADR-0006: Canonical Status And Doctor Contracts

## Status

Accepted

## Context

Live status, support bundles, Desktop Monitor, Android Doctor summaries, and
local HTTP status output need the same facts. Before the shared contract work,
there was a risk that each surface would hand-build a slightly different JSON
shape or diagnosis summary.

This affects `src/status_api.rs`, `src/doctor.rs`, Android JNI, Desktop UI,
support redaction, docs, and future Observatory/Route Advisor work.

## Decision

Rust owns canonical status/stats and Doctor JSON renderers. Platform surfaces
may project that data into UI-specific cards, but they must not become separate
sources of truth for the underlying schema.

## Consequences

- Local `/status`, support bundles, and Android JNI use shared Rust renderers.
- Android may parse raw or enveloped stats for compatibility, but the contract
  remains documented.
- Doctor summary cards must use stable item IDs, levels, counts, and
  redaction-safe projections.
- Future Observatory and Route Advisor views should build on these contracts,
  not parse logs as their primary data source.
- Schema changes require docs, guards, tests, changelog, and support snapshot
  updates together.

## Follow-Up

Keep `docs/status-stats-json-contract.md`,
`docs/doctor-json-contract.md`, `docs/android-support-snapshot.md`, and their
static guards in repo-sanity.
