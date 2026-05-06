# Batch 6 Verification Profile Pack - 2026-05-06

## UI / UX

- No product UI behavior changed.
- Contributor and maintainer UX improved by documenting which verification
  checks apply to each change type instead of forcing every change through an
  opaque all-or-nothing checklist.

## Config / Schema

- Added a machine-readable verification profile contract:
  `docs/verification-profiles.json`.
- Added a human-readable projection:
  `docs/verification-profiles.md`.
- Profiles cover docs/governance, config/schema, Android UI, backend helpers,
  Desktop/Rust runtime, full tunnel/tunnel-node, and release readiness.

## Backend Helpers

- Backend helper verification is now a named profile with Apps Script hardening,
  Cloudflare Worker relay, mode vocabulary, fixture, docs, and Apps Script test
  commands grouped together.

## Security / Trust

- Release-ready verification continues to treat CI as authoritative for
  official artifacts and Android JVM tests.
- Verification docs explicitly keep Gradle out of normal local checks unless a
  contributor is intentionally in a provisioned Android environment.

## Breaking / Cleanup

- No compatibility path changed.
- No legacy verification convention was kept as a competing source of truth;
  the new profile docs link back to existing repo-sanity and release-checklist
  authority.

## Parity

- Added `tools/check-verification-profiles.py`.
- Wired the guard into:
  - `tools/run-repo-sanity.py`;
  - `tools/check-ci-local-sanity-parity.py`.
- Linked verification profiles from:
  - `docs/index.md`;
  - `CONTRIBUTING.md`;
  - `tools/README.md`;
  - `docs/release-checklist.md`.

## Race / Split-Brain Review

- The guard prevents split-brain by requiring:
  - a stable JSON schema marker;
  - stable profile IDs and order;
  - high-risk command markers in each profile;
  - docs/contributor/tools/release discoverability;
  - local sanity and CI/local parity wiring.

## Verification

- `python tools/check-verification-profiles.py`
- `python tools/check-doc-links.py`
- `python tools/check-ci-local-sanity-parity.py`

Full repo-sanity and cleanup were run as part of batch closeout.
