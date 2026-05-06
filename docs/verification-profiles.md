# Verification Profiles

This document explains which checks to run for common change types. It is a
human-readable projection of [`docs/verification-profiles.json`](verification-profiles.json).
The JSON is the small contract that `tools/check-verification-profiles.py`
guards.

Profiles are additive. If a change touches Android UI and config schema, run
both profiles. If a change is broad or release-bound, run `release_ready`.

## Core Rules

- Run the smallest profile that covers the changed surfaces, then expand when a
  command or review finding points to adjacent drift.
- Keep Desktop, Android, CLI, backend helpers, examples, docs, changelog, and
  roadmap aligned for every cross-surface change.
- Do not run Gradle locally unless you are intentionally in a provisioned
  Android environment. CI owns Android JVM execution; local profiles use static
  guards.
- Remove stale outputs and local caches after verification.
- Treat `tools/run-repo-sanity.py` as the local mirror of CI `repo-sanity`.

## Profile Summary

| Profile | Use When |
|---|---|
| `docs_governance` | Markdown, roadmap, changelog, ADR, contributor, security, ownership, or release-process docs change without runtime behavior. |
| `config_schema` | Config fields, config registry, Desktop ConfigWire, Android ConfigStore, examples, platform defaults, or parity matrix change. |
| `android_ui` | Android Compose UI, Android config import/export, JNI bridge, VPN lifecycle, support snapshot, localization, or mobile diagnostics change. |
| `backend_helpers` | Apps Script helpers, Cloudflare Worker bridge, Vercel/Netlify relay docs, compatibility markers, or backend registry docs change. |
| `desktop_runtime` | Rust runtime, Desktop UI, Doctor/status contracts, readiness, LAN sharing, mode behavior, or support bundle behavior changes. |
| `full_tunnel` | Full mode, tunnel-node, UDP/udpgw, batching/coalescing, tunnel drain behavior, or CodeFull/tunnel-node docs change. |
| `release_ready` | Before tagging or publishing a public release, or after broad cross-platform changes. |

## Docs / Governance Only

Use this for docs-only governance work where no runtime behavior changes:

```powershell
python tools\check-doc-links.py
python tools\check-doc-anchors.py
python tools\check-changelog-headings.py
python tools\generate-changelog-index.py -Check
python tools\check-release-governance.py
python tools\check-repo-governance.py
python tools\check-adr-governance.py
python tools\check-verification-profiles.py
python tools\check-repo-cleanliness.py
```

Regenerate `docs/changelog/index.md` before the `-Check` command when adding a
new changelog file.

## Config / Schema / Examples

Use this for config fields, examples, platform defaults, and parity matrix work:

```powershell
python tools\generate-config-registry.py -Check
python tools\check-config-registry-nested-fields.py
python tools\check-config-registry-map-semantics.py
python tools\check-config-wire-vs-registry.py
python tools\check-android-config-keys.py
python tools\check-android-owned-keys-list.py
python tools\check-mode-example-fixtures.py
python tools\check-platform-defaults.py
python tools\generate-platform-defaults-doc.py -Check
python tools\check-android-platform-defaults-test-static.py
python tools\generate-parity-matrix.py -Check
python tools\check-parity-matrix.py
cargo test bundled_example_configs_load_and_validate
```

## Android UI / Mobile Bridge

Use this for Android UI, JNI, localization, VPN lifecycle, and support snapshot
work:

```powershell
python tools\check-android-string-resource-parity.py
python tools\generate-android-hardcoded-copy-inventory.py -Check
python tools\check-android-config-sharing.py
python tools\check-android-vpn-lifecycle.py
python tools\check-android-support-redaction.py
python tools\check-android-support-snapshot-schema.py
python tools\check-android-doctor-jni-bridge.py
python tools\check-android-doctor-summary-ui.py
python tools\check-platform-defaults.py
python tools\check-android-platform-defaults-test-static.py
```

## Backend Helpers / Relay Scripts

Use this for Apps Script, Cloudflare Worker bridge, serverless relay, and helper
compatibility work:

```powershell
python tools\check-apps-script-hardening.py
python tools\check-cloudflare-worker-relay.py
python tools\check-mode-vocabulary.py
python tools\check-mode-example-fixtures.py
python tools\check-doc-links.py
python tools\check-doc-anchors.py
node assets\apps_script\tests\batch_fallback_test.js
node assets\apps_script\tests\compat_marker_test.js
node assets\apps_script\tests\edge_dns_test.js
```

## Desktop / Rust Runtime

Use this for Desktop UI, Rust runtime, Doctor/status, readiness, LAN sharing,
and mode behavior:

```powershell
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --features ui
python tools\check-status-stats-json-contract.py
python tools\check-doctor-json-contract.py
python tools\check-desktop-doctor-summary.py
python tools\check-desktop-test-relay-mode-guard.py
python tools\check-readiness-ui-contract.py
python tools\check-lan-sharing-ui.py
```

## Full Tunnel / tunnel-node

Use this for full mode, tunnel-node, UDP/udpgw, and batching/coalescing work:

```powershell
python tools\check-coalesce-tuning.py
python tools\check-tunnel-node-drain-concurrency.py
python tools\check-desktop-test-relay-mode-guard.py
cargo fmt --check
cargo test --all-targets --features ui
Push-Location tunnel-node
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
Pop-Location
```

## Release Ready

Use this before tagging or after broad cross-platform work:

```powershell
python tools\run-repo-sanity.py
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --features ui
Push-Location tunnel-node
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
Pop-Location
python tools\check-ci-local-sanity-parity.py
python tools\check-repo-cleanliness.py
```

CI remains authoritative for official artifacts and Android JVM tests.
