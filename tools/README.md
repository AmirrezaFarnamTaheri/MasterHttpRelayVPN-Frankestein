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

## CI / local sanity parity

`tools/check-ci-local-sanity-parity.py` keeps `.github/workflows/ci.yml` and the
local sanity runner aligned:

- CI must call `python3 tools/run-repo-sanity.py` instead of duplicating a
  second hand-maintained Python/Node drift-check list;
- CI must keep the release-blocking Rust format, clippy, root tests, and
  `tunnel-node` clippy/test steps;
- CI must keep the Android JVM platform-defaults test in the CI/pre-provisioned
  environment while local `run-repo-sanity.py` remains Gradle-free;
- the local runner must still include the high-risk static gates for Android VPN
  teardown, Android Doctor JNI output, Desktop Doctor summaries, Desktop Test
  Relay mode semantics, tunnel-node drain/concurrency, platform defaults,
  config registry, parity matrix, and stale Android/docs scans.

This guard is itself bundled into `tools/run-repo-sanity.py`, so workflow drift
breaks both local sanity and CI repo-sanity quickly.

```powershell
python tools\check-ci-local-sanity-parity.py
```

## Telegram release notification renderer

`tools/check-telegram-release-notify.py` imports
`.github/scripts/telegram_release_notify.py` without contacting Telegram and
checks the release-note renderer:

- leading changelog editor comments are stripped;
- changelog Markdown links, bold spans, and inline code become Telegram HTML;
- raw `<`, `>`, and `&` text is escaped before posting;
- long changelog replies are bounded under Telegram's message size limit;
- blockquote reply structure remains intact.

This keeps the optional Telegram job a safe projection of
`docs/changelog/v*.md`, not a separate release-note source of truth.

```powershell
python tools\check-telegram-release-notify.py
```

## Changelog index generator

`tools/generate-changelog-index.py` builds `docs/changelog/index.md` from every
Markdown changelog file in `docs/changelog/` except the folder README and the
index itself. It reads each file's first `#` heading, sorts entries by the date
embedded in the filename, and gives maintainers a single audit-trail table.
`tools/check-changelog-headings.py` keeps that title source strict by requiring
an H1 in every source changelog file.

```powershell
python tools\check-changelog-headings.py
python tools\generate-changelog-index.py
python tools\generate-changelog-index.py -Check
```

## Release governance guard

`tools/check-release-governance.py` keeps the release/changelog governance
surfaces connected: top-level `CHANGELOG.md`, rolling `docs/RELEASE_NOTES.md`,
the changelog template/index, release checklist, versioning policy, rollback
policy, docs index, and local repo-sanity wiring.

```powershell
python tools\check-release-governance.py
```

## Repo governance guard

`tools/check-repo-governance.py` keeps contributor and security surfaces in
place: `CONTRIBUTING.md`, `SECURITY.md`, ownership notes, PR template, issue
templates, docs-index links, and repo-sanity wiring.

```powershell
python tools\check-repo-governance.py
```

## ADR governance guard

`tools/check-adr-governance.py` keeps lightweight architecture decision records
discoverable and structurally consistent:

- `docs/adr/README.md` and `docs/adr/TEMPLATE.md` must exist;
- required seed ADRs for signing material, release artifacts, Android config
  preservation, legacy migration, platform defaults, status/Doctor contracts,
  lightweight governance, and cleanup policy must stay present;
- every ADR must include `Status`, `Context`, `Decision`, and `Consequences`;
- docs index, contributing guide, tools docs, local repo-sanity, and CI/local
  parity guard must all link or run the ADR guard.

```powershell
python tools\check-adr-governance.py
```

## Verification profiles guard

`tools/check-verification-profiles.py` keeps
[`docs/verification-profiles.md`](../docs/verification-profiles.md) and
[`docs/verification-profiles.json`](../docs/verification-profiles.json)
aligned. The profiles explain which checks to run for common change types:
docs/governance, config/schema, Android UI, backend helpers, Desktop/Rust
runtime, full tunnel/tunnel-node, and release readiness.

The guard requires:

- the JSON schema marker `mhrv-f-verification-profiles/v1`;
- stable profile IDs and order;
- high-risk commands in each profile;
- docs-index, contributing-guide, tools-docs, repo-sanity, and CI/local parity
  wiring.

```powershell
python tools\check-verification-profiles.py
```

## Change-impact checklist guard

`tools/check-change-impact-checklist.py` keeps
[`docs/change-impact-checklist.md`](../docs/change-impact-checklist.md) and
[`docs/change-impact-checklist.json`](../docs/change-impact-checklist.json)
aligned. The checklist maps touched surfaces to verification profiles, parity
surfaces, and cleanup duties before a change is closed out.

The guard requires:

- the JSON schema marker `mhrv-f-change-impact-checklist/v1`;
- stable surface IDs and order;
- every referenced verification profile to exist in
  `docs/verification-profiles.json`;
- docs-index, contributing-guide, PR-template, tools-docs, repo-sanity, and
  CI/local parity wiring.

```powershell
python tools\check-change-impact-checklist.py
```

## Tooling source map guard

`tools/check-tooling-source-map.py` keeps
[`docs/tooling-source-map.md`](../docs/tooling-source-map.md) and
[`docs/tooling-source-map.json`](../docs/tooling-source-map.json) aligned. The
map lists high-risk generated or contract docs and the generators/guards that
protect each one.

The guard requires:

- the JSON schema marker `mhrv-f-tooling-source-map/v1`;
- stable mapped document paths and order;
- every mapped document, source, generator, and guard to exist;
- docs-index, contributing-guide, tools-docs, repo-sanity, and CI/local parity
  wiring.

```powershell
python tools\check-tooling-source-map.py
```

## README Persian guide links

`tools/check-readme-persian-guides.py` keeps the README's Persian onboarding
resources visible and compact:

- the language switcher stays near the top;
- the Persian YouTube setup video link remains in the first screen;
- Kian Irani's Persian text guide and credit link remain in the first screen;
- external links keep `target="_blank"` and `rel="noopener noreferrer"`;
- the README does not reintroduce a large YouTube thumbnail embed.

This preserves the useful upstream README onboarding idea without turning the
top of the README into a media block.

```powershell
python tools\check-readme-persian-guides.py
```

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

## Android string resource parity

`tools/check-android-string-resource-parity.py` keeps
`android/app/src/main/res/values/strings.xml` and
`android/app/src/main/res/values-fa/strings.xml` aligned:

- every English string key must exist in Persian;
- Persian must not contain orphan keys missing from English;
- duplicate or blank string values fail fast;
- high-risk mode/action/repair labels must stay present in both locales.

This is a local static guard and does not require Gradle.

```powershell
python tools\check-android-string-resource-parity.py
```

## Android hard-coded copy inventory

`tools/generate-android-hardcoded-copy-inventory.py` generates
`docs/android-hardcoded-copy-inventory.md`, the source-controlled inventory of
visible Android Compose literals that still live in Kotlin:

- `Text("...")` user-visible copy;
- `label = "..."` readiness/detail labels;
- `title = "..."`, `body = "..."`, `detail = "..."`, and
  `placeholder = "..."` builder-style prose;
- `contentDescription = "..."` accessibility labels;
- dynamic placeholders and pure technical tokens are separated from copy that
  should move to string resources.

This is an inventory for intentional dynamic/technical tokens, not a parking
lot for untranslated prose. The `-Check` form is bundled into
`tools/run-repo-sanity.py` / CI `repo-sanity`, and now fails when any detected
literal is classified as `localize`. New hard-coded visible copy must move to
string resources in the same change, or the scanner classification must be
updated only for genuinely technical/dynamic tokens.

Batch 61 expanded the scanner to cover readiness-card `detail` prose and other
common builder-style named arguments after Batch 60 moved those strings into
EN/FA resources. If future Kotlin UI builders use a new string-valued argument
for visible copy, expand this scanner in the same change or move the copy
directly into resources.

```powershell
python tools\generate-android-hardcoded-copy-inventory.py
python tools\generate-android-hardcoded-copy-inventory.py -Check
```

## Status/stats JSON contract

`tools/check-status-stats-json-contract.py` keeps live stats serialization from
splitting across local `/status`, support bundles, and Android JNI:

- `src/status_api.rs` must expose `stats_snapshot_json_value`;
- the shared renderer must include both canonical keys
  (`scripts_total`, `scripts_blacklisted`) and Android legacy aliases
  (`total_scripts`, `blacklisted_scripts`);
- `Native.statsJson(handle)` must call the shared renderer instead of
  hand-maintaining its own `StatsSnapshot` field list.
- `docs/status-stats-json-contract.md` must document the raw Android object,
  the `/status` / support-bundle envelope, every required field, and the
  Android `root.optJSONObject("stats") ?: root` parser rule.

The check is part of `tools/run-repo-sanity.py`, so CI inherits it through the
single repo-sanity job.

Contract reference: [`docs/status-stats-json-contract.md`](../docs/status-stats-json-contract.md).

```powershell
python tools\check-status-stats-json-contract.py
```

## Doctor JSON contract

`tools/check-doctor-json-contract.py` keeps structured Doctor diagnostics from
splitting between Rust Doctor, support-bundle `doctor.json`, and future UI
diagnostic cards:

- `src/doctor.rs` must expose `doctor_report_json_value` and
  `doctor_item_json_value`;
- the shared renderer must include `ok`, `items`, `id`, `level`, `title`,
  `detail`, and `fix`;
- levels must stay the stable strings `ok`, `warn`, and `fail`;
- support bundles must call `doctor::doctor_report_json_value(&report)` instead
  of hand-building `doctor.json`;
- `docs/doctor-json-contract.md` must document the shape and consumer rules.

The check is part of `tools/run-repo-sanity.py`, so CI inherits it through the
single repo-sanity job.

Contract reference: [`docs/doctor-json-contract.md`](../docs/doctor-json-contract.md).

```powershell
python tools\check-doctor-json-contract.py
```

## Android Doctor JNI bridge

`tools/check-android-doctor-jni-bridge.py` keeps Android diagnostics on the same
Doctor JSON contract before the mobile UI grows a full Doctor card:

- `src/android_jni.rs` must expose `Native.doctorJson(configJson)`;
- the JNI bridge must parse through Rust `Config::from_json_str`;
- the bridge must run Rust Doctor and serialize with
  `doctor::doctor_report_json_value(&report)`;
- invalid config and runtime-init failures must still return the same
  contract-shaped `ok/items/id/level/title/detail/fix` JSON envelope;
- `Native.kt` must declare the JNI method and document the shared contract.

The Android UI card is guarded separately by
`tools/check-android-doctor-summary-ui.py`.

```powershell
python tools\check-android-doctor-jni-bridge.py
```

## Android Doctor summary UI

`tools/check-android-doctor-summary-ui.py` keeps the mobile Doctor card aligned
with the shared bridge and localization rules:

- `HomeScreen.kt` must parse the `ok/items/id/level/title/detail/fix` Doctor
  contract from `Native.doctorJson(configJson)`;
- Doctor runs must snapshot `cfg.toJson()` before entering the JNI call;
- stale results must be ignored if the config changed while Doctor was running;
- the card must render in the main home flow;
- all visible Doctor-card copy must exist in both English and Persian string
  resources.

```powershell
python tools\check-android-doctor-summary-ui.py
```

## Desktop Doctor summary guard

`tools/check-desktop-doctor-summary.py` keeps the Desktop Monitor from falling
back to log-only Doctor output after the structured Doctor contract landed:

- `UiState` must keep the latest typed `DoctorReport` and update timestamp;
- the Monitor tab must render `render_doctor_summary_card(...)`;
- plain Doctor runs must store the report they just produced;
- Doctor+Fix runs must store the post-fix report, not the stale pre-fix state;
- the Doctor+Fix log loop must borrow the report items so the final report is
  still available for the UI card.

Desktop consumes typed Rust state directly, while Android consumes the
contract-shaped JNI bridge. Both are guarded so later UI refactors do not
recreate log-only diagnostics.

```powershell
python tools\check-desktop-doctor-summary.py
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

## Apps Script relay hardening

`tools/check-apps-script-hardening.py` keeps the Apps Script helper hardening
aligned across `Code.gs`, `CodeFull.gs`, and `CodeCloudflareWorker.gs`:

- exactly one `doGet` per helper;
- `doGet` and `_json` use `ContentService`, not `HtmlService` wrappers;
- identity / IP-leak headers are stripped in every helper;
- `fetchAll` fallback replays only safe methods;
- batch responses preserve original indexes;
- Rust keeps the `goog.script.init` / `userHtml` unwrap helpers and tests.

This gate is bundled into `tools/run-repo-sanity.py` / CI `repo-sanity`.

```powershell
python tools\check-apps-script-hardening.py
```

## Cloudflare Worker relay bridge

`tools/check-cloudflare-worker-relay.py` keeps the optional Apps Script +
Cloudflare Worker bridge aligned:

- the Worker must require `WORKER_AUTH_KEY`;
- the Worker must keep loop/self-fetch guards and strip forwarded/client-IP and
  Cloudflare identity headers;
- `CodeCloudflareWorker.gs` must keep its compatibility marker, separate
  client/Worker secrets, batch support, and safe replay fallback;
- docs, Desktop backend tools, and release checklist must keep presenting this
  as optional `apps_script` egress, not as a separate mode or full tunnel
  replacement.

```powershell
python tools\check-cloudflare-worker-relay.py
```

## Desktop LAN sharing UI guard

`tools/check-lan-sharing-ui.py` keeps the friendly LAN-share workflow from
regressing back to a raw `listen_host` edit:

- `src/lan_utils.rs` must keep the cross-platform UDP route-table helper plus
  wildcard/loopback classifiers and unit tests;
- the Desktop **Sharing and per-app routing** section must keep the
  **Share with other devices on my Wi-Fi / network** checkbox;
- the checkbox must own the normal `127.0.0.1` ⇄ `0.0.0.0` transition;
- custom bind addresses must show a **Custom bind** badge and must not be
  overwritten by Save;
- HTTP/SOCKS endpoints must stay copyable and use the detected LAN IP or the
  `this-device-LAN-IP` fallback;
- docs must explain the UDP route lookup, no-packet behavior, `lan_allowlist`,
  and the SOCKS/token limitation.

This static gate is bundled into `tools/run-repo-sanity.py` / CI
`repo-sanity`; the Rust `lan_utils` unit tests remain the executable helper
contract.

```powershell
python tools\check-lan-sharing-ui.py
```

## Fronting groups starter example

`tools/check-fronting-groups-example.py` keeps
`config.fronting-groups.example.json` useful and aligned with docs:

- the example must remain a `direct` mode, loopback-only starter;
- Vercel, Fastly, and Netlify/CloudFront groups must stay present;
- the Fastly starter keeps the curated Reddit, Pinterest, CNN, BuzzFeed,
  GitHub asset, PyPI, and Fastly domain families;
- docs must keep explaining that these are examples to verify and trim, not
  guaranteed routes;
- the parity matrix must keep `config.fronting-groups.example.json` attached to
  `direct` mode.

This protects donor/upstream example value without promoting it into default
runtime behavior.

```powershell
python tools\check-fronting-groups-example.py
```

## Full-mode coalesce tuning

`tools/check-coalesce-tuning.py` keeps the v1.9.8/v1.9.9 low-latency
full-tunnel coalescing profile aligned:

- Rust client compiled defaults stay `10 ms` step and `1000 ms` max;
- `ProxyServer` still translates config `0` into those compiled defaults;
- Android config defaults/import fallbacks stay concrete `10` / `1000`;
- bundled full-mode examples stay on `10` / `1000` rather than the old
  conservative `40` / `1000` starter;
- tunnel-node straggler settle stays `10 ms` step and `1000 ms` max;
- platform defaults docs, advanced-options docs, and the tuning changelog keep
  explaining the zero-sentinel vs concrete-mobile-default split.

This is a static guard; runtime behavior remains covered by Rust and
tunnel-node tests.

```powershell
python tools\check-coalesce-tuning.py
```

## Desktop Test Relay mode guard

`tools/check-desktop-test-relay-mode-guard.py` keeps the Desktop **Test Relay**
button honest across modes. The button is a relay-path probe; in `full` mode the
real data plane is `CodeFull.gs` plus `tunnel-node`, and in `direct` mode there
is no relay backend. The guard fails if the UI stops short-circuiting those
modes with an explanatory skip message before `test_cmd::run(...)`.

The same gate also checks that `docs/relay-modes.md` documents how users should
verify `full` and `direct` mode without treating Test Relay as a routing oracle.

```powershell
python tools\check-desktop-test-relay-mode-guard.py
```

## Desktop UI modularization guard

`tools/check-desktop-ui-modularization.py` keeps the first low-risk Desktop UI
extraction from collapsing back into `src/bin/ui.rs`:

- formatting helpers live in `src/bin/ui_format.rs`;
- file/resource-opening helpers live in `src/bin/ui_fs.rs`;
- visual tokens, theme setup, section/help primitives, primary buttons, and
  compact form rows live in `src/bin/ui_style.rs`;
- Trust tab layout, Trust Center snapshot rendering, and support-bundle preview
  rendering live in `src/bin/ui_trust.rs`;
- Doctor summary rendering and Doctor level-label helpers live in
  `src/bin/ui_doctor.rs`;
- Help-tab walkthrough prose, backend-tool catalog data, and row rendering live
  in `src/bin/ui_help.rs`;
- mode summary, mode dashboard readiness, repair routing, and dashboard chip
  helpers live in `src/bin/ui_mode.rs`;
- first-run Setup wizard rendering lives in `src/bin/ui_setup.rs`;
- XHTTP form defaults, candidate lists, VLESS-link generation, deploy-note
  generation, cloud deploy polling, and the XHTTP renderer live in
  `src/bin/ui_xhttp.rs`;
- `src/bin/ui.rs` imports those helpers instead of redefining them;
- helper modules keep small unit tests;
- the tooling source map and repo-sanity wiring stay aware of the boundary.

```powershell
python tools\check-desktop-ui-modularization.py
```

## Canonical relay-mode vocabulary

`tools/check-mode-vocabulary.py` keeps the product-mode names aligned across
docs, Desktop, and Android:

- wire/config values remain `apps_script`, `vercel_edge`, `direct`, and `full`;
- user-facing labels stay **Apps Script**, **Serverless JSON**,
  **Direct fronting**, and **Full tunnel**;
- Desktop and Android mode selectors must use the friendly labels rather than
  raw config names;
- `docs/relay-modes.md`, `docs/index.md`, and `README.md` must keep the same
  mode comparison and compatibility explanation.

```powershell
python tools\check-mode-vocabulary.py
```

## Mode example fixtures

`tools/check-mode-example-fixtures.py` keeps the bundled config examples aligned
with Rust validation tests and the parity matrix:

- each product mode must have at least one `config*.example.json` fixture;
- each fixture must declare the expected `mode`;
- `src/config.rs` must keep the `bundled_example_configs_load_and_validate`
  test and include each fixture with the expected Rust `Mode`;
- `docs/parity-matrix.json` must list examples under the matching mode;
- Android import/export must keep the same wire-mode mappings and advanced
  unknown-field preservation path.

```powershell
python tools\check-mode-example-fixtures.py
```

## Readiness UI contract

`tools/check-readiness-ui-contract.py` keeps stable readiness IDs and repair
targets connected to user-facing surfaces:

- Rust remains the source of truth in `src/readiness.rs`;
- generated Android IDs and `docs/readiness-matrix.md` must contain every Rust
  readiness ID;
- Desktop mode/dashboard repair actions must consume `readiness::ReadinessId`,
  `repair_for_id`, and `repair_anchor_for_target`;
- Android Home must consume generated `ReadinessIds`,
  `ReadinessRepairTargets`, and `ReadinessRepairAnchors` for its repair cards;
- `tools/run-repo-sanity.py` must still run both the generated readiness
  contract check and this UI drift gate.

This protects the stable ID plus repair action contract without requiring
Gradle locally.

```powershell
python tools\check-readiness-ui-contract.py
```

## tunnel-node drain / concurrency guard

`tools/check-tunnel-node-drain-concurrency.py` keeps the v1.9.9 tunnel-node
correctness fixes from regressing without needing to run a tunnel server:

- watcher tasks must be wrapped in `AbortOnDrop`, so `tokio::select!`
  cancellation cannot detach stale notify waiters;
- batch TCP/UDP drain lists must carry cloned session `Arc`s, so the global
  sessions maps are not held across per-session awaits;
- mixed TCP+UDP long-polls must use an empty-aware `select!`, not a conjunctive
  `join!`;
- EOF cleanup must follow the `drain_now` / `drain_udp_now` return value, so
  over-cap tail bytes are not dropped when a raw EOF atomic is already set;
- the tunnel-node regression tests for tail preservation and mixed TCP/UDP
  latency must stay present.

This static gate is bundled into `tools/run-repo-sanity.py` / CI
`repo-sanity`. The executable tunnel-node tests remain the deeper runtime
contract.

```powershell
python tools\check-tunnel-node-drain-concurrency.py
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
and full Apps Script deployment IDs. It also requires Android Doctor output to
flow into the copied snapshot only as a redacted summary: availability, ok/fail
counts, and warning/failing item IDs. Full Doctor titles, details, fixes, URLs,
and endpoint text stay out of the copied mobile support text.
The current copied-text schema is documented in
`docs/android-support-snapshot.md`; schema marker, docs, tests, and guard
markers must change together.

This gate is bundled into `tools/run-repo-sanity.py` / CI `repo-sanity`. It is
static and does not run Gradle; the executable Android JVM test remains the
deeper CI/pre-provisioned contract.

```powershell
python tools\check-android-support-redaction.py
python tools\check-android-support-snapshot-schema.py
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
