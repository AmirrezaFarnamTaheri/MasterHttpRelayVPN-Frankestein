# Release Checklist

Use this before tagging a public release. CI remains the source of truth for
build artifacts; this checklist catches human-facing drift that CI cannot infer.

## Source And Generated Files

- Run the normal Rust checks: format, tests, clippy, and the UI-feature build.
- Run the generated readiness contract check:

  ```bash
  pwsh ./tools/generate-readiness-contract.ps1 -Check
  ```

- Confirm generated Android readiness IDs and `docs/readiness-matrix.md` are
  committed when readiness rules, repair targets, or repair anchors changed.
- Confirm Android English/Persian string keys are still paired.
- Confirm root `config*.example.json` files still parse and validate through
  the Rust config contract:

  ```bash
  cargo test bundled_example_configs_load_and_validate
  ```

- Do not commit local build outputs from `target/`, Android `build/`, `dist/`,
  or ad-hoc release folders.
- Run the repository cleanliness check:

  ```bash
  python tools/check-repo-cleanliness.py
  ```

  Local `dist/` and `releases/` directories may exist as backup/archive
  material, but CI-generated releases remain authoritative.
- Run the local Markdown link check:

  ```bash
  python tools/check-doc-links.py
  ```

## Apps Script Helpers

- If any file in `assets/apps_script/` changed, review all helper variants:
  - `Code.gs`
  - `CodeFull.gs`
  - `CodeCloudflareWorker.gs`
- Confirm each helper has the current compatibility markers:
  - `HELPER_KIND`
  - `HELPER_VERSION`
  - `HELPER_PROTOCOL`
  - `HELPER_FEATURES`
- Run the helper tests:

  ```bash
  node assets/apps_script/tests/batch_fallback_test.js
  node assets/apps_script/tests/compat_marker_test.js
  node assets/apps_script/tests/edge_dns_test.js
  ```

- Syntax-check `.gs` helpers by copying or piping them through `node --check`
  with a `.js` extension or stdin; Node does not accept `.gs` directly.
- After deployment, open:

  ```text
  https://script.google.com/macros/s/DEPLOYMENT_ID/exec?compat=1
  ```

  Confirm the returned `kind`, `version`, `protocol`, and `features` match the
  helper documented for the selected mode.

## Backend And Docs Parity

- Re-read `docs/relay-modes.md` for mode names and backend responsibilities.
- Check `README.md`, `docs/index.md`, Android docs, and Desktop docs for stale
  mode names, ports, helper names, screenshots, or release artifact names.
- For Cloudflare Worker relay changes, update
  `docs/cloudflare-worker-json-relay.md` and `docs/cfw-reference-audit.md`.
- For full tunnel changes, update `docs/relay-modes.md`,
  `docs/doctor.md`, and `tunnel-node/README.md`.

## Release Artifacts

- Let `.github/workflows/release.yml` publish official artifacts.
- Treat local `dist/` and `releases/` contents as backups only.
- Verify release notes mention user-visible UI, helper, Android, and backend
  behavior changes.
- Verify `SHA256SUMS.txt` is present in the GitHub Release before announcing.
