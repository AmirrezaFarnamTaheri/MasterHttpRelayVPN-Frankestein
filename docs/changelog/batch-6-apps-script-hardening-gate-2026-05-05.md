# Batch 6 - Apps Script Hardening Gate

Date: 2026-05-05

## Summary

Added a static guard for the Apps Script relay hardening absorbed from upstream:
ContentService `doGet`, IP-leak header stripping, safe-method `fetchAll`
fallback, original-index batch results, and Rust `goog.script.init` unwrapping.

## Changes

- Added `tools/check-apps-script-hardening.py`.
- Wired it into `tools/run-repo-sanity.py`.
- Added it to the CI/local parity guard.
- Documented it in `tools/README.md`.
- Expanded `CodeCloudflareWorker.gs` header stripping to match the fuller
  identity-header family already protected in the main Apps Script helpers and
  Rust client.

## Guarded Contract

- `Code.gs`, `CodeFull.gs`, and `CodeCloudflareWorker.gs` must each have exactly
  one `doGet`.
- Helpers must use `ContentService` for `doGet` and `_json`.
- Helpers must not use `HtmlService.createHtmlOutput`.
- Helpers must strip the forwarded/proxy/client-IP identity header family.
- Batch fallback must replay only `GET`, `HEAD`, and `OPTIONS`.
- Batch results must preserve original request indexes.
- Rust must keep the `goog.script.init` / `userHtml` unwrap helpers and
  regression tests.

## Parity / Split-Brain Notes

- All Apps Script helper variants are checked together.
- Rust client defense-in-depth is checked with the helper contract, so old
  HtmlService-wrapped deployments remain supported while maintained helpers stay
  ContentService-based.

## Verification

- `python tools/check-apps-script-hardening.py`
- `node assets/apps_script/tests/batch_fallback_test.js`
- `node assets/apps_script/tests/compat_marker_test.js`
- `python tools/check-ci-local-sanity-parity.py`
- `python tools/run-repo-sanity.py --skip-node`
- `python tools/run-repo-sanity.py`
- `python tools/check-doc-links.py`
- `python tools/check-repo-cleanliness.py`

All checks passed on 2026-05-05. The first targeted run caught a real parity
gap: the Rust client stripped `x-forwarded-ssl`, but the main Apps Script
helpers did not yet. The helper header lists were aligned and the targeted,
Node, docs, cleanliness, and full repo-sanity checks passed afterward.

## Cleanup

- Removed generated `.github/scripts/__pycache__` and `tools/__pycache__`
  directories after verification.
- No Gradle command was run and no Android build output was generated.
- Process inspection found no active `gradle`, `java`, or `kotlinc` processes
  during closeout.
