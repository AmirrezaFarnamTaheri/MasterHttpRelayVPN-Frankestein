# Maintainer Tools

This directory contains helper projects and maintainer scripts that are part of
the repository contract. Tooling here should be deterministic and safe to run
without creating release artifacts unless its README says otherwise.

The **repo-sanity** CI job runs `python3 tools/run-repo-sanity.py` so the local
command and CI stay identical. [`docs/release-checklist.md`](../docs/release-checklist.md)
lists the same workflow for release prep.

To mirror **repo-sanity** locally (Node syntax, Apps Script tests, Python drift
gates, readiness `-Check`, JSON/XML/string/stale scan):

```powershell
python tools\run-repo-sanity.py
```

Use `python tools/run-repo-sanity.py --skip-node` on machines without Node.js
(Python gates only), or `--skip-readiness` when PowerShell is unavailable.

## Readiness Contract Generator

Rust readiness IDs and repair targets are the source of truth in
`src/readiness.rs`. Android consumes a generated Kotlin mirror at
`android/app/src/main/java/com/farnam/mhrvf/ReadinessIds.kt`.

Regenerate the Android mirror after changing readiness IDs or repair targets:

```powershell
# PowerShell 7 (recommended, matches CI)
pwsh ./tools/generate-readiness-contract.ps1

# Windows PowerShell 5.1 (built-in on most Windows installs)
powershell -NoProfile -ExecutionPolicy Bypass -File tools\generate-readiness-contract.ps1
```

Check that the generated file is current without modifying it:

```powershell
# PowerShell 7 (recommended, matches CI)
pwsh ./tools/generate-readiness-contract.ps1 -Check

# Windows PowerShell 5.1 (built-in on most Windows installs)
powershell -NoProfile -ExecutionPolicy Bypass -File tools\generate-readiness-contract.ps1 -Check
```

CI runs the check form. This generator does not run Gradle and should not create
Android build outputs.

## Config capability registry

`docs/config-registry.json` holds canonical metadata for every **root** serialized
`Config` key. Optional **`nested_fields`** documents inner JSON keys for
composites (`vercel`, `account_groups[]`, `domain_overrides[]`, `fronting_groups[]`).
`tools/generate-config-registry.py` writes `docs/config-registry.md` and
`docs/config-parity-matrix.md`. Rust test **`config_registry_covers_all_config_keys`**
locks registry **top-level** keys to `Config` serde output.

```powershell
python tools\generate-config-registry.py
python tools\generate-config-registry.py -Check
```

Bundled into **`tools/run-repo-sanity.py`** / CI **repo-sanity**.

`tools/check-config-registry-nested-fields.py` asserts each **`nested_fields`**
block matches **`pub`** fields on the corresponding Rust struct in
`src/config.rs` (same mapping as `generate-config-registry.py`).

```powershell
python tools\check-config-registry-nested-fields.py
python tools\check-config-registry-map-semantics.py
```

## Desktop ConfigWire vs registry

`tools/check-config-wire-vs-registry.py` requires **`ConfigWire`** (`src/bin/ui.rs`)
field names to match **top-level** **`docs/config-registry.json`** keys so Desktop
save cannot omit new **`Config`** roots.

```powershell
python tools\check-config-wire-vs-registry.py
```

## Android config drift gates

`tools/check-android-config-keys.py` scans **`ConfigStore.kt`** JSON key literals against
**`docs/config-registry.json`** plus explicit allowlists.

`tools/check-android-owned-keys-list.py` validates the **`ownedKeys`** list used when
computing **`preservedUnknownRootJson`** — every entry must be a registry root or an
allowlisted Android-only / legacy key.

**`tools/android_config_allowlists.py`** holds the shared allowlists (**`ANDROID_ONLY_KEYS`**,
**`LEGACY_KEYS`**, and nested **`NESTED_KEYS`** used only by the ConfigStore literal scan).

```powershell
python tools\check-android-config-keys.py
python tools\check-android-owned-keys-list.py
```

## Android QR/deep-link config sharing drift gate

`tools/check-android-config-sharing.py` keeps Android config sharing aligned with
the product contract:

- exported links use the current `mhrvf://` scheme;
- imports still accept legacy `mhrv-rs://` links;
- payloads stay deflate-compressed, URL-safe Base64 JSON;
- QR/deep-link export preserves imported unknown root keys instead of silently
  dropping Desktop/hand-written advanced config;
- invalid QR/deep-link payloads decode to `null` instead of a partial config.

This is a static no-Gradle gate inside `tools/run-repo-sanity.py` / CI
`repo-sanity`. Android JVM tests in `ConfigStoreTest.kt` remain the executable
contract in CI or a pre-provisioned environment.

```powershell
python tools\check-android-config-sharing.py
```

## Android VPN lifecycle / teardown drift gate

`tools/check-android-vpn-lifecycle.py` keeps the Android disconnect race fixes
from regressing without running Gradle locally:

- `MainActivity.kt` must send `ACTION_STOP` through `startService(stopAction)`.
- `MainActivity.kt` must not call `stopService()` on Disconnect; the service
  owns teardown and calls `stopSelf()` after cleanup.
- `MhrvVpnService.teardown()` must stop the Rust proxy before
  `Tun2proxy.stop()`, TUN close, and `tun2proxyThread.join(...)`, so the native
  worker's SOCKS5 read wakes before runtime memory is released.
- UI state must flip only after native teardown work has run.

This is bundled into `tools/run-repo-sanity.py` / CI `repo-sanity`. Device-level
disconnect stress testing still belongs in CI/pre-provisioned Android testing.

```powershell
python tools\check-android-vpn-lifecycle.py
```

## External / donor proxy JSON report (Batch 2)

**`tools/report-nova-proxy-config.py`** compares top-level keys of a Nova-style or
other donor JSON export against **`docs/config-registry.json`** — migration triage
only; it does **not** import rules into mhrv-f.

```powershell
python tools\report-nova-proxy-config.py --demo
python tools\report-nova-proxy-config.py --path D:\downloads\nova-export.json
python tools\report-nova-proxy-config.py --demo --no-nested
```

CI/local **`run-repo-sanity.py`** runs **`--demo`** (skipped silently if the Nova tree is absent).

## Example Config Contract Test

The Rust config tests load every root `config*.example.json` file through
`Config::from_json_str`, including migration and validation. Run this focused
test after changing config schema, examples, mode names, or readiness blockers:

```powershell
cargo test bundled_example_configs_load_and_validate
```

## Platform defaults contract (Rust × Android)

`docs/platform-defaults.json` records **intentional** differences (for example
Android `8080/1081` vs Desktop `8085/8086`, parallel relay / coalesce starter
numbers, and Rust vs Android preset `google_ip`) plus **`parity_shared_defaults`**
must stay identical (`verify_ssl`, QUIC/DoH/YouTube-relay booleans,
`relay_path`). `tools/check-platform-defaults.py` parses `src/config.rs` and
`ConfigStore.kt` and fails if the code diverges from the JSON contract.
`tools/generate-platform-defaults-doc.py` renders `docs/platform-defaults.md`.

Rust also asserts minimal-config serde defaults against the JSON contract via
`cargo test minimal_direct_json_matches_platform_defaults_contract`.

Android JVM coverage lives in
`android/app/src/test/java/com/farnam/mhrvf/PlatformDefaultsContractTest.kt`. **MAINTAINERS OUTSOURCE GRADLE TO CI**: GitHub **`android-unit-tests`** runs those JVM tests on Ubuntu (SDK installed there; **`cargo-ndk`** skipped).

Without Gradle locally, repo sanity still enforces alignment via:

```powershell
python tools\check-platform-defaults.py
python tools\check-android-platform-defaults-test-static.py
python tools\generate-platform-defaults-doc.py
python tools\generate-platform-defaults-doc.py -Check
```

`check-android-platform-defaults-test-static.py` asserts every **`shared` / `parity_shared_defaults` / `android`** contract field appears twice in the Kotlin test source (both test bodies).

## Android support redaction drift gate

`tools/check-android-support-redaction.py` keeps Android copied support
diagnostics owned by `SupportRedaction.kt`. It fails if `HomeScreen.kt`
reintroduces local support-snapshot or deployment-ID masking helpers, and it
requires the static JVM-test source to keep assertions for omitted auth keys,
serverless auth keys, LAN tokens, upstream SOCKS5 credentials, raw unknown JSON,
and full Apps Script deployment IDs.

This gate is bundled into `tools/run-repo-sanity.py` / CI `repo-sanity`. It is
static and does not run Gradle; the executable Android JVM test remains the
deeper CI/pre-provisioned contract.

```powershell
python tools\check-android-support-redaction.py
```

## SNI default pool parity (Rust × Android)

`DEFAULT_GOOGLE_SNI_POOL` in `src/domain_fronter.rs` must match the ordered
`DEFAULT_SNI_POOL` in `android/app/.../ConfigStore.kt` (see roadmap P0.5 /
G5.5). CI compares both lists mechanically.

```powershell
python tools\check-sni-default-pool.py
```

## Repository Cleanliness Check

`tools/check-repo-cleanliness.py` keeps local build output, oversized source
files, binary artifacts, local secrets, and stale-prone image references out of
the maintained source tree.

Run it from the repository root:

```powershell
python tools\check-repo-cleanliness.py
```

CI runs the same script. Local `dist/` and `releases/` folders are reported as
allowed backup/archive material, not as release sources. The official release
artifacts still come from `.github/workflows/release.yml`.

## Markdown Local Link Check

`tools/check-doc-links.py` checks local Markdown links in README, docs,
maintainer tools, Apps Script helper docs, tunnel-node docs, and release
fallback docs. It skips external URLs and pure in-page anchors, then verifies
that relative file/directory targets exist.

Run it from the repository root:

```powershell
python tools\check-doc-links.py
```

CI runs the same check.

## Markdown Anchor Check

`tools/check-doc-anchors.py` validates `file.md#anchor` fragments against actual
headings in the target Markdown file (GitHub-style slug rules). This prevents
docs from silently accumulating broken section links.

Run it from the repository root:

```powershell
python tools\check-doc-anchors.py
```

CI runs the same check.

## Parity Matrix Generator

`tools/generate-parity-matrix.py` generates `docs/parity-matrix.md` from the
canonical JSON source `docs/parity-matrix.json`. CI runs `-Check` to ensure the
matrix stays current.

```powershell
python tools\generate-parity-matrix.py
python tools\generate-parity-matrix.py -Check
```

`tools/check-parity-matrix.py` is a drift gate on `docs/parity-matrix.json`: **`backend_taxonomy`**
must match the `backends` keys in order; mode keys must match Rust `Mode::as_str`; and every
`docs` / `examples` path must exist under the repo root.

```powershell
python tools\check-parity-matrix.py
```

## Workspace Cleanup

For removing ignored build outputs (`target/`, `tunnel-node/target/`, Android
`.gradle/`, Python `__pycache__/`, and similar), see
[`docs/workspace-cleanup.md`](../docs/workspace-cleanup.md).

The cleanliness script **does not walk** donor reference trees at the repo
root (`mhr-cfw-main/`, `Nova-Proxy-App-main/`, `youtube-domain-fronting-patch-main/`)
so CI stays fast; keep binaries and local secrets out of those folders anyway.
