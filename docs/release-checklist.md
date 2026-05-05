# Release Checklist

Use this before tagging a public release. CI remains the source of truth for
build artifacts; this checklist catches human-facing drift that CI cannot infer.

## Source And Generated Files

- Run the normal Rust checks: format, tests, clippy, and the UI-feature build
  from the repo root; then run **`tunnel-node`** the same way CI does:

  ```bash
  cd tunnel-node
  cargo clippy --all-targets -- -D warnings
  cargo test --all-targets
  cd ..
  ```
- Run the generated readiness contract check:

  CI runs this step with PowerShell 7 (`pwsh`), but **local Windows verification
  does not require `pwsh`**. Use whichever PowerShell you have:

  ```powershell
  # PowerShell 7 (recommended, matches CI)
  pwsh ./tools/generate-readiness-contract.ps1 -Check

  # Windows PowerShell 5.1 (built-in on most Windows installs)
  powershell -NoProfile -ExecutionPolicy Bypass -File tools\generate-readiness-contract.ps1 -Check
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

- When `docs/parity-matrix.json` or mode/backend docs references change,
  regenerate the parity matrix and run its drift gates (CI runs the same `-Check`
  step):

  ```bash
  python tools/generate-parity-matrix.py
  python tools/generate-parity-matrix.py -Check
  python tools/check-parity-matrix.py
  ```

- When `docs/config-registry.json` or serialized `Config` fields change, regenerate
  the registry docs and run the freshness gate:

  ```bash
  python tools/generate-config-registry.py
  python tools/generate-config-registry.py -Check
  python tools/check-config-registry-nested-fields.py
  python tools/check-config-registry-map-semantics.py
  python tools/check-config-wire-vs-registry.py
  ```

  Rust CI also runs `cargo test config_registry_covers_all_config_keys`.

- When `docs/platform-defaults.json` or importer defaults in Rust/Android change,
  verify generated docs and static Android contract references:

  ```bash
  python tools/check-platform-defaults.py
  python tools/generate-platform-defaults-doc.py -Check
  python tools/check-android-platform-defaults-test-static.py
  ```

- Before pushing broad parity or docs changes, mirror CI **repo-sanity**
  locally:

  ```bash
  python tools/run-repo-sanity.py
  ```

  (`tools/README.md` documents `--skip-node` / `--skip-readiness`.) Alternatively,
  run individual drift gates such as:

  ```bash
  python tools/check-doc-anchors.py
  python tools/check-android-config-keys.py
  python tools/check-android-owned-keys-list.py
  python tools/check-sni-default-pool.py
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

## Release Notification Authority

- **Canonical release:** The GitHub Release created by `.github/workflows/release.yml`
  (tag, artifacts, `SHA256SUMS.txt`, and release body). This is the only
  authoritative “what shipped” announcement for builds and hashes.
- **Optional Telegram post:** When repo variable `TELEGRAM_NOTIFY_ENABLED` is
  `true` and secrets are set, the workflow may post the CI-built universal APK
  and optional changelog text. Treat Telegram as a **convenience mirror**, not
  a second source of truth: if a post disagrees with the GitHub Release, trust
  GitHub.
- Maintainer batch logs (for example `docs/changelog/batch-*.md`) are for audit
  and bookkeeping; they do not replace per-version release notes or the GitHub
  Release body.

## Workspace Cleanup (Before Or After Release Prep)

See [`docs/workspace-cleanup.md`](workspace-cleanup.md) to drop regenerable
`target/`, Gradle caches, and similar artifacts so local verification stays fast.
