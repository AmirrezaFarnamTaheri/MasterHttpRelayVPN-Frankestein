# Batch 6 - Contributor / Security Governance Pack - 2026-05-06

## Summary

Added contributor, security, issue, PR, and ownership governance surfaces so new
work starts with the same parity, verification, redaction, and release
expectations used by the current implementation batches.

## Changed

- Added `CONTRIBUTING.md`.
- Added `SECURITY.md`.
- Added `docs/ownership.md`.
- Added `.github/pull_request_template.md`.
- Added issue templates:
  - `.github/ISSUE_TEMPLATE/bug_report.yml`;
  - `.github/ISSUE_TEMPLATE/android_problem.yml`;
  - `.github/ISSUE_TEMPLATE/backend_helper_problem.yml`;
  - `.github/ISSUE_TEMPLATE/feature_request.yml`.
- Added `tools/check-repo-governance.py`.
- Wired the guard into `tools/run-repo-sanity.py`.
- Added the guard to `tools/check-ci-local-sanity-parity.py`.
- Updated `docs/index.md` and `tools/README.md`.

## UI / UX

N/A. GitHub issue/PR forms affect contributor UX, not app runtime UI.

## Config / Schema

N/A. No app config/schema changed.

## Backend Helpers

N/A. Backend/helper code did not change, but the backend issue template now asks
for helper kind/version/compatibility context without secrets.

## Security / Trust

Added `SECURITY.md` with vulnerability-reporting guidance, supported-version
expectations, secret-handling rules, trust-model caveats, and security-sensitive
change areas. Issue/PR templates explicitly ask reporters/contributors not to
include raw keys, tokens, deployment IDs, signing material, or private URLs.

## Docs

Docs index now links the contributing guide, security policy, and ownership
notes. Tools docs describe the new governance guard.

## Breaking / Cleanup

No legacy compatibility branch was added. This replaces the previous implicit
contributor/security process with explicit docs and static enforcement.

## Parity

The PR template requires Desktop, Android, CLI, backend/helper, docs,
English/Persian copy, config/schema/readiness/status, and redaction review.

## Race / Split-Brain Review

The guard checks that the contributor/security surfaces stay present and linked
through docs and repo sanity. CODEOWNERS remains deferred until maintainers are
available; `docs/ownership.md` is the current lightweight ownership surface.

## Verification

- `python tools/check-repo-governance.py`
- `python tools/check-ci-local-sanity-parity.py`
- `python tools/check-doc-links.py`

## Cleanup

No generated release artifacts or Android/Gradle outputs were created.
