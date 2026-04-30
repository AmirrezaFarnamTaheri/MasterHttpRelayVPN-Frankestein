# Elevation Audit Roadmap Source

Last updated: 2026-04-30

This document is the living source of truth for improving, enhancing, and
elevating the UI, UX, in-app help, documentation, guides, and support surfaces
of MasterHttpRelayVPN-Frankestein (`mhrv-f`).

It is intentionally both a roadmap and a bookkeeping document. Use it to plan
work, log decisions, track progress, prevent duplicated documentation, and keep
the desktop UI, Android UI, CLI help, README, and docs aligned.

## How To Use This Document

Update this file whenever meaningful UI/UX/docs work starts, changes direction,
lands, or is deferred.

Recommended workflow:

1. Add or update an item in the relevant workstream table.
2. Set its status to `todo`, `in_progress`, `blocked`, `review`, `done`, or
   `deferred`.
3. Add an owner if known.
4. Add evidence: file paths, screenshots, test results, user feedback, or issue
   links.
5. When implementation lands, fill the acceptance checklist and note any
   follow-up debt.

Status vocabulary:

| Status | Meaning |
|---|---|
| `todo` | Agreed work, not started. |
| `in_progress` | Actively being designed, written, or implemented. |
| `blocked` | Cannot proceed until a dependency, decision, or bug is resolved. |
| `review` | Implemented or drafted, waiting for design/code/docs review. |
| `done` | Accepted and verified. |
| `deferred` | Intentionally postponed, with a reason. |

## Current Product Shape

`mhrv-f` is no longer just a command-line proxy. It is a multi-surface product:

- Desktop app: Rust `eframe` / `egui` UI in `src/bin/ui.rs`.
- Android app: Kotlin Compose UI in `android/app/src/main/java/com/farnam/mhrvf/ui`.
- CLI: setup, serve, diagnostics, scans, support bundle, and config workflows.
- Docs: README plus many focused Markdown guides under `docs/`.
- In-app help: desktop Help tab plus Android guide strings.
- Release and installer surfaces: Windows, macOS, Linux, Android, and release notes.

The project already has strong functional depth. The elevation task is to make
that depth feel coherent, guided, trustworthy, and manageable.

## Audit Snapshot

Observed during inspection:

| Area | Current signal | UX implication |
|---|---|---|
| Desktop UI file | `src/bin/ui.rs` is about 4,827 lines | Hard to evolve safely; UI concepts, state, widgets, copy, commands, and layout are tightly mixed. |
| Android home screen | `HomeScreen.kt` is about 1,746 lines | Compose surface is functional but large; first-run, settings, help, logs, and diagnostics compete in one file. |
| Desktop build | `cargo check --features ui --bin mhrv-f-ui` passes | Refactors can proceed from a compiling baseline. |
| Screenshot assets / references | No `docs/*.png` screenshot was found during this pass; docs and release surfaces still need stale-image and stale-name review | Visual docs can mislead users and contributors if old product identity or old UI images remain referenced. |
| Docs breadth | Many English docs exist | Good coverage, but risk of overlap and unclear canonical entry points. |
| Persian docs | Some are much shorter than English or missing entirely | Language toggle and Persian docs need parity strategy. |
| Android strings | English has 150 string keys, Persian has 138 | Missing localization keys create fallback or incomplete localized UX. |
| Android hard-coded copy | Several visible strings are in Kotlin | Localization and consistency are harder. |
| Desktop docs vs implementation | Docs describe persistent top controls; current code places the full central panel in a scroll area | Expected interaction model and actual implementation may drift. |

## Project-Aware Deep Audit Findings

This update expands the roadmap after inspecting the actual project surfaces,
not just the existing roadmap text.

Inspected surfaces:

| Surface | Files / areas | Current signal | Elevation implication |
|---|---|---|---|
| Rust config contract | `src/config.rs` | `Config` is the real backend schema and validator; canonical Apps Script identity is `account_groups` | Every frontend must serialize the same contract or provide an explicit migration path. |
| Desktop UI | `src/bin/ui.rs` | 4,827 lines; desktop uses `account_groups` and a custom `ConfigWire` serializer | Desktop is closer to canonical config but needs contract tests so future fields do not disappear on save. |
| Android config bridge | `android/app/src/main/java/com/farnam/mhrvf/ConfigStore.kt` | Android still writes top-level `script_ids` and `auth_key` for Apps Script | High-priority parity gap: Rust now validates `account_groups` for `apps_script` and `full`. |
| Android UI | `android/app/src/main/java/com/farnam/mhrvf/ui/HomeScreen.kt` | 1,746 lines; several user-visible strings remain hard-coded | Localization and terminology parity are not enforceable yet. |
| Android JNI bridge | `src/android_jni.rs` and `Native.kt` | JNI exposes `startProxy`, `stopProxy`, `statsJson`, `drainLogs`, `testSni`, `checkUpdate`, `exportCa` | Good bridge exists, but Doctor/support-bundle/status parity is incomplete. |
| Runtime status | `src/status_api.rs`, `src/domain_fronter.rs`, `src/android_jni.rs` | Stats are rendered in multiple shapes; Android emits a subset plus aliases | Needs one status/stats DTO contract shared by CLI, desktop, Android, and support bundles. |
| CLI surface | `src/main.rs` | Commands include `test`, `doctor`, `doctor-fix`, `init-config`, `support-bundle`, `rollback-config`, `scan-ips`, `scan-sni`, `test-sni` | UI/docs should expose or explain all user-relevant commands with the same names and expectations. |
| Documentation | `README.md`, `docs/*.md` | Broad coverage; JSON vs XHTTP distinction is mostly present; Persian docs vary in depth | Needs owner map, parity policy, stale-name scans, and release review gates. |
| Backend helpers | `assets/apps_script/*`, `tools/*-json-relay`, `tools/*-xhttp-relay`, `tunnel-node` | Multiple backend shapes with similar provider names | Needs a single backend matrix so users do not mix native JSON, Apps Script, XHTTP, and full tunnel instructions. |
| Release / install surfaces | `.github/workflows/*`, launchers, installer scripts, Android packaging | Many release channels | Release QA must include UI/docs/config parity checks, not only builds. |

### Backend / Frontend Parity Audit Snapshot

| Contract | Backend source of truth | Frontend / docs consumers | Observed gap | Impact | Roadmap response |
|---|---|---|---|---|---|
| Operating modes | `Mode` in `src/config.rs` | Desktop mode selector, Android `Mode`, README, docs, CLI output | Names mostly align, but `vercel_edge` is an internal config name while UI says Serverless JSON | Users and contributors may confuse config values with product labels | Add a mode contract table and require all surfaces to reference it. |
| Apps Script credentials | `account_groups` in `src/config.rs` | Desktop `AccountGroupForm`, Android `appsScriptUrls/authKey`, README examples | Android writes legacy top-level `script_ids` and `auth_key`; Rust validation requires `account_groups` | Android `apps_script` / `full` can fail validation or fail to round-trip desktop configs | High-priority parity workstream G1. |
| Serverless JSON credentials | `vercel` object in `src/config.rs` | Desktop Serverless JSON fields, Android serverless fields, docs | Mostly aligned, but label says Vercel while Netlify is also supported | Confusing provider mental model | Rename user-facing copy to "Serverless JSON"; keep `vercel` as compatibility schema name. |
| Network defaults | Rust defaults and example configs | Desktop defaults, Android defaults, docs | Desktop uses `8085/8086`; Android intentionally uses `8080/1081`; Android Google IP default differs from Rust | Could look like a bug unless documented as platform-specific | Make platform default differences explicit and tested. |
| SNI pool | `DEFAULT_GOOGLE_SNI_POOL` in `src/domain_fronter.rs` | Desktop SNI editor, Android `DEFAULT_SNI_POOL`, docs | Android manually mirrors Rust list | Lists can drift silently | Add generated/checking parity script. |
| Stats/status shape | `StatsSnapshot`, `status_api`, JNI `statsJson` | Desktop Monitor, Android usage card, support bundle, local `/status` | Similar data emitted in different JSON shapes | Dashboards and support bundles can disagree | Define one status schema and reuse it. |
| Readiness rules | `Config::validate`, `unsafe_warnings`, Doctor | Desktop disabled Start, Android disabled Connect, docs | Desktop and Android compute readiness locally and differently | A config can look valid in one UI and fail in native runtime | Add shared readiness matrix and test fixtures. |
| Diagnostics | `doctor.rs`, `test_cmd.rs`, `scan_*` | CLI, desktop logs, Android SNI tester, docs | Doctor is structured internally but UI mostly logs results; Android has SNI/update but no full Doctor summary | Troubleshooting remains log-driven | Add structured diagnostic summaries and UI mapping. |
| Safety warnings | `unsafe_warnings`, CA installer, LAN auth logic | Desktop Network/Help, Android CA dialog, docs | Warnings exist but are not governed as one vocabulary | Risk copy may drift or become less actionable | Add safety copy contract and review gate. |
| Localization | Android resources, Persian docs | Android UI, README Persian, docs/*.fa.md | English Android has 150 keys; Persian has 138; some visible strings are hard-coded | Persian experience is visibly incomplete | Add string parity tooling and hard-coded copy inventory. |
| Example configs | `config*.example.json` | README, docs, desktop import, Android share/import | Examples are useful but not schema-tested across UIs | Examples can rot after schema changes | Add example round-trip tests. |
| Backend helper taxonomy | `assets/`, `tools/`, `tunnel-node/` | Setup tab, Help, README, docs | JSON vs XHTTP is documented, but similar provider names remain easy to mix | Users deploy the wrong thing for the chosen mode | Add backend matrix, banners, and UI affordances. |

### Immediate High-Risk Parity Gaps

These should be treated as first-wave work, because they can create user-visible
breakage even before visual polish begins.

| ID | Gap | Evidence | Proposed resolution |
|---|---|---|---|
| P0.1 | Android Apps Script config uses legacy `script_ids` / `auth_key` instead of `account_groups` | `ConfigStore.toJson()` writes top-level legacy keys; `Config::validate()` requires `account_groups` for `apps_script` / `full` | Either serialize one default `account_groups` entry from Android, or add a Rust compatibility migration from legacy keys into `account_groups`, then test both paths. |
| P0.2 | Android import/export ignores canonical `account_groups` | `ConfigStore.loadFromJson()` reads top-level `script_ids`, not `account_groups` | Teach Android to read canonical groups and map the first simple group into the current mobile fields; preserve complex multi-group JSON where the mobile UI cannot fully edit it. |
| P0.3 | Desktop save still omits some `Config` fields | Static field comparison: `Config` has 47 fields; `ConfigWire` has 45; missing `domain_overrides` and `enable_batching` | Add `domain_overrides` to `ConfigWire`; decide whether `enable_batching` is deprecated, hidden, or intentionally omitted. Add a field-parity test. |
| P0.4 | Android config model lacks many backend fields | Android reads/writes a subset: no canonical `account_groups`, `domain_overrides`, `relay_rate_limit_*`, runtime profile, timeout, outage reset, max body, `youtube_via_relay`, etc. | Classify every backend field as edit, preserve, default, ignore, or Android-only; preserve fields that Android cannot edit. |
| P0.5 | SNI defaults are duplicated across Rust and Android | Android comment says it mirrors Rust `DEFAULT_GOOGLE_SNI_POOL` | Add a parity check that fails when the lists differ. |
| P0.6 | Android visible copy still has hard-coded English | `ModeOverviewCard`, mode labels, `Block QUIC`, language button labels | Move all user-facing copy to resources, then enforce values/values-fa parity. |
| P0.7 | Persian Android strings are missing keys | English 150 keys; Persian 138 keys; missing serverless and guide/group keys | Fill missing keys and add a script to keep parity. |
| P0.8 | Status JSON is split between status API and Android JNI shape | `status_api::render_status_json` and `Native.statsJson` build JSON separately | Create a single Rust renderer for status/stats and call it from status API, JNI, support bundle, and desktop. |
| P0.9 | Platform defaults are not explicitly governed | Desktop docs use 8085/8086; Android docs use 8080/1081; Rust default Google IP differs from Android | Add a platform defaults table and decide which differences are intentional. |
| P0.10 | No Android unit or instrumentation tests were found under `android/app/src/test` or `android/app/src/androidTest` | Static inventory found no test files | Add focused tests for `ConfigStore`, readiness, localization key usage, and import/export before large UI refactors. |
| P0.11 | Android release signing is intentionally committed but not governed in the roadmap | `android/app/release.jks` exists and `build.gradle.kts` contains release signing credentials/comments tied to legacy signature continuity | Record an explicit security/release decision: keep and document, rotate and move to CI secrets, or split public/dev signing from official release signing. |
| P0.12 | Generated release output contains screenshot/docs that are not present in source docs | `dist/mhrv-f-windows-installer-v1.2.13/docs/ui-screenshot.png` exists while source `docs/*.png` does not | Treat `dist/` as backup/archive only; release packaging must copy from current source or generated release screenshots, never stale local output. |
| P0.13 | Full Persian parity is much broader than the Android string gap | Many English docs have no `.fa.md` counterpart, and some existing Persian docs are much shorter: `relay-modes.md`, `fronting-groups.md`, JSON relay docs, XHTTP docs, `ui-desktop.md`, `troubleshooting.md` | Add a Persian parity matrix for every user-facing and maintainer-facing doc; implement full parity or tracked exceptions. |
| P0.14 | Temporary dependency patch needs a cleanup owner | `Cargo.toml` has a temporary `tun2proxy` git patch for Android `udpgw_server` JNI support | Track upstream status, add removal criteria, and include this in the compatibility/deprecation registry. |
| P0.15 | Build/release outputs are large and easy to confuse with source | `target/`, `tunnel-node/target`, `dist/`, and `releases/` together measured roughly 9.8 GB in this workspace | Add artifact inventory, cleanup scripts/checks, and source-vs-generated policy before release work. |
| P0.16 | Release notes/changelog source of truth is still unclear | `docs/RELEASE_NOTES.md`, release-drafter config, GitHub workflows, and Telegram release notification all exist | Define changelog/release-note ownership so users, GitHub releases, docs, and notifications do not drift. |

### Contract Ownership Rule

Whenever a backend feature, config field, mode, status metric, diagnostic ID, or
user-visible safety warning changes, the change is not complete until this
checklist is addressed:

1. Rust config schema and validation.
2. Example config files.
3. Desktop form load/save and readiness.
4. Android `MhrvConfig` load/save/import/export.
5. CLI help or command output if user-facing.
6. In-app Help/guide copy.
7. English docs and Persian docs policy.
8. Tests or scripts that catch future drift.
9. Release checklist entry when the behavior affects setup, safety, or support.

## Product Elevation Goals

The elevated experience should feel like a clear connection cockpit, not a dense
settings editor.

Primary goals:

1. Help a first-time user reach a working tunnel with fewer wrong turns.
2. Help a returning user see health, risk, quota pressure, and the next action
   in seconds.
3. Separate everyday actions from expert tuning.
4. Preserve power-user depth without forcing everyone to read it.
5. Make Desktop, Android, CLI, docs, and in-app help tell the same story.
6. Treat trust, security, certificate behavior, and quota cost as first-class UX.
7. Make the codebase easier to improve without accidental regressions.

Non-goals:

- Do not remove advanced capabilities.
- Do not hide security trade-offs behind cheerful copy.
- Do not turn the desktop app into a marketing page.
- Do not invent unsupported automatic routing features.
- Do not make docs shorter by deleting necessary caveats; instead, layer them.

## Second-Pass End-To-End Audit Addendum

This second pass looked beyond the original UI/config/docs hotspots and checked
artifact folders, scripts, workflows, Android resources, helper tools, launchers,
installer assets, and generated output.

Additional inspected surfaces:

| Surface | Current signal | Risk | Roadmap response |
|---|---|---|---|
| Generated artifacts | `target/`, `tunnel-node/target`, `dist/`, and `releases/` total roughly 9.8 GB in this workspace | Generated output can hide stale docs/screenshots and make the source tree feel unprofessional | Artifact policy, cleanup scripts, and release-source-of-truth checks. |
| Packaged screenshot | `dist/mhrv-f-windows-installer-v1.2.13/docs/ui-screenshot.png` exists but no source `docs/*.png` exists | Release packages can ship stale visuals even when source docs look clean | Release packaging should fail or warn when packaged docs contain stale screenshot references. |
| Persian documentation | Source docs include many English-only guides and several shorter Persian equivalents | Maintainer chose full Persian parity, so this is larger than an Android string cleanup | Add doc-level parity matrix and prioritize mode/setup/troubleshooting/helper docs. |
| Android string parity | English Android strings: 150; Persian: 138; 12 keys missing in Persian | Localized UI can fall back or become incomplete | Keep P0 string parity item; add CI gate before UI copy expansion. |
| Temporary dependency patch | `Cargo.toml` patches `tun2proxy` from a git branch until upstream support lands | Temporary compatibility code can become permanent invisible debt | Track as a compatibility surface with owner, upstream condition, and removal criteria. |
| Release communication | `docs/RELEASE_NOTES.md`, release-drafter config, release workflow, and Telegram notification script coexist | Users can receive inconsistent release notes | Define one release-note source and generated/published projections. |
| Helper tooling | JSON relays, XHTTP relays, Apps Script helpers, and tunnel-node each have separate docs and deploy shapes | Similar provider names can still confuse mode selection | Backend matrix and mode-specific rich setup surfaces. |
| Launchers/installers | Windows installer scripts, macOS app builder, launchers, OpenWRT init script, and release workflow all exist | Install experience is part of UX and must be reviewed with the app, not after it | Add installer/launcher UX and docs checks to release QA. |

Second-pass conclusion:

The main missed risk is not a single code file. It is drift between source,
generated packages, docs translations, helper deploy paths, and release
communication. The roadmap should therefore treat UI/UX elevation and repo
cleanliness as one system: every visible improvement must include docs,
artifacts, release packaging, localization, tests, and cleanup.

## Executive Elevation Plan

Use this as the practical next sequence. The full workstreams below remain the
detailed source of truth, but this plan keeps the work ordered.

### Stage 1: Stabilize Product Contracts

Purpose:

Make sure Desktop, Android, CLI, examples, docs, and helper backends all agree
before large UI changes sit on top of drifting behavior.

Do first:

- Fix Android canonical `account_groups` import/export.
- Use a simple first-group Android editor for normal setup while preserving
  complex multi-group Desktop configs; add an advanced editor only after the
  preservation path is stable.
- Fix Desktop `ConfigWire` parity for `domain_overrides` and decide
  `enable_batching`.
- Add `Config`/Desktop/Android field parity checks.
- Add Android `ConfigStore` tests.
- Add SNI, string, and example-config parity checks.
- Define shared readiness IDs and status JSON shape.

Success signal:

- Every supported mode can be configured, saved, loaded, validated, and tested
  from the relevant UI without schema surprises.

### Stage 2: Make First-Run Setup Feel Guided

Purpose:

Turn setup from a dense settings exercise into a guided path by mode.

Do next:

- Desktop: fixed command center, complete Setup tab, one primary action,
  mode-specific required fields, inline validation.
- Android: summary-first home screen, sticky Connect/Disconnect, guided setup
  sections, localized copy, canonical config sharing.
- Docs: canonical setup entry point, backend matrix, JSON-vs-XHTTP distinction,
  platform defaults table.

Success signal:

- A new user can choose Apps Script, Serverless JSON, direct, or full tunnel and
  see the exact next step without visiting Advanced or searching multiple docs.

### Stage 3: Elevate Troubleshooting And Support

Purpose:

Make failures understandable without requiring users to read raw logs first.

Do next:

- Shared status/support snapshot.
- Structured Doctor IDs and severity.
- Desktop Monitor as a health dashboard.
- Android diagnostics summary.
- Redacted support bundles.
- Mode-aware empty/error states.

Success signal:

- A failed setup produces a plain-language reason, one repair action, and a
  redacted support artifact.

### Stage 4: Professionalize Docs And Repo Maintenance

Purpose:

Make the repository feel like a maintained product, not a pile of powerful
parts.

Do next:

- Docs owner map, last-reviewed metadata, stale-name/link checks.
- `CONTRIBUTING.md`, `CHANGELOG.md`, release checklist, PR template, issue
  templates, ADRs for major product/security decisions.
- CI gates for parity and docs quality.
- Module boundaries for large UI files after behavior is protected by tests.
- Clear policy for generated artifacts, release artifacts, local build outputs,
  and committed signing material.

Success signal:

- New contributors can understand how to build, test, change config schema,
  update docs, and prepare a release without reverse-engineering the project.

### Stage 5: Polish, Design System, And Long-Term Refinement

Purpose:

Use the stable contract and repo foundation to polish without breaking things.

Do next:

- Semantic colors for state, trust, warnings, quota, and errors.
- Layout density rules for Desktop and Android.
- Screenshot QA for Desktop and Android states.
- Component extraction and previews.
- Accessibility, RTL, small-screen, and short-viewport reviews.

Success signal:

- The product feels coherent, calm, and trustworthy across Desktop, Android,
  CLI, docs, and release surfaces.

## Next Implementation Packages

These packages are the recommended next implementation slices. They are sized
to reduce risk, create visible progress, and avoid mixing product design,
schema migration, and repo cleanup in one oversized change.

### Package 1: Mobile Config Contract Repair

Outcome:

Android can safely create, import, export, and preserve configs that Rust and
Desktop understand.

Scope:

- Android writes canonical `account_groups` for Apps Script/full mode.
- Android reads canonical `account_groups`.
- Normal mobile setup edits one primary group: auth key plus script IDs.
- Imported multi-group configs are preserved even when not fully editable.
- UI shows a compact "advanced groups preserved" state when applicable.
- Legacy `script_ids` / `auth_key` imports continue to work.
- Add `ConfigStore` tests for canonical export, legacy import, multi-group
  preservation, and redaction.

Out of scope for first pass:

- Full mobile clone of the Desktop multi-account group editor.
- Drag/reorder/group-level tuning UI.
- Advanced per-group controls unless preservation tests are already green.

### Package 2: Desktop Command Center And Setup Flow

Outcome:

Desktop starts feeling like a connection cockpit instead of a long form.

Scope:

- Fixed command center with state, mode, readiness, and one primary action.
- Complete Setup tab for Apps Script, Serverless JSON, direct, and full.
- Inline readiness reasons tied to shared validation IDs.
- Network/CA/LAN warnings are contextual.
- Advanced tab stops being required for basic setup.
- Screenshot baseline before and after.

### Package 3: Docs And Guides Reset

Outcome:

Users and contributors know where to start, which backend to deploy, and which
docs are canonical.

Scope:

- `docs/README.md` canonical map.
- Backend matrix: Apps Script, Serverless JSON, XHTTP, direct, full tunnel.
- JSON-vs-XHTTP banners in relevant docs.
- Platform defaults table.
- Full Persian parity plan and first parity pass for core docs.
- Last-reviewed metadata for major docs.
- Stale-name, stale-version, stale-screenshot-reference, and link checks.

### Package 4: Repo Professionalization Baseline

Outcome:

The repo starts looking and operating like a maintained engineering project.

Scope:

- `CONTRIBUTING.md`.
- `docs/maintainer-guide.md`.
- `CHANGELOG.md`.
- `SECURITY.md`.
- PR template and issue templates.
- `docs/adr/` with initial ADRs for signing material, release artifact policy,
  Android account-group UX, platform defaults, and status schema.
- Artifact policy: CI/release workflow is source of truth; `dist/` /
  `releases/` are labeled backup/archive material only.

### Package 5: Diagnostics, Status, And Support

Outcome:

Failures produce a clear reason, one next action, and a redacted support
artifact.

Scope:

- Shared status snapshot schema.
- Doctor severity IDs and UI mapping.
- Desktop Monitor health dashboard.
- Android diagnostics summary.
- Redacted support bundle parity.
- Mode-specific stats empty states.

### Package 6: Visual Polish And Maintainability

Outcome:

The UI becomes calmer and easier to keep improving.

Scope:

- Semantic state colors.
- Layout density and typography rules.
- Android RTL and small-screen QA.
- Desktop short-viewport QA.
- Extract pure helper logic before moving large UI components.
- Split Desktop and Android UI files only after contract/screenshot tests exist.

### Package 7: Cleanup And Garbage Collection Discipline

Outcome:

Every improvement leaves the repo cleaner than it found it.

Scope:

- Remove stale or deprecated code paths after migrations are complete.
- Remove unused functions, parameters, resources, docs snippets, screenshots,
  examples, generated artifacts, and old compatibility names unless they are
  intentionally supported.
- Mark intentional legacy compatibility with comments, tests, and docs.
- Run dead-code, stale-name, unused-resource, docs-link, and example-config
  checks before marking work done.
- Update changelog/release notes when cleanup changes user-visible behavior.
- Add a "garbage collection completed" evidence note to the roadmap item.

Out of scope:

- Removing compatibility surfaces that still protect real users, such as
  legacy deep-link schemes or old config imports, unless a migration window and
  release note exist.

## Decisions Needed From Maintainer

These decisions are worth making before implementation proceeds too far. When
decided, this table records the implementation direction so future work does
not re-litigate the same product calls.

| ID | Decision | Current direction | Why it matters |
|---|---|---|---|
| D-1 | Android account groups UI: full multi-account editor or simple editor that preserves complex groups? | Decided: keep Android on par with Desktop at the config contract level, but use the mobile-native simple-first UI. Normal setup edits one primary group; imported complex multi-group configs are preserved safely; a full advanced mobile editor is optional later. | Fixes parity without making the first mobile setup feel heavy. |
| D-2 | Legacy Android config migration boundary | Support legacy `script_ids` / `auth_key` in Android import and add Rust compatibility only if real legacy files must load outside Android | Keeps Rust schema clean while preserving user imports. |
| D-3 | `enable_batching` status | Treat as advanced/experimental; preserve in config and document, but do not make it prominent | It exists in backend/serverless docs and should not be silently lost. |
| D-4 | Android vs Desktop defaults | Decided by maintainer delegation: keep current platform-specific ports for now and document them; do not churn defaults until smoke tests exist. Treat Google IP drift as test-governed: align only after validation proves no platform-specific reason remains. | Prevents "is this a bug?" confusion while avoiding risky default churn. |
| D-5 | Status renderer ownership | Make Rust shared status snapshot canonical, with Android allowed to display a mobile projection | Prevents `/status`, Desktop, Android, and support bundles from drifting. |
| D-6 | Android release signing material | Decided: keep committed signing material for install-over compatibility, and document the policy, risk, rotation path, and official-release expectations explicitly. | This is a security and trust decision, not only a build convenience. |
| D-7 | Persian documentation depth | Decided: full Persian parity, not curated parity. Every user-facing guide and core maintainer/release/security doc should have a Persian counterpart or a tracked exception. | Makes Persian a first-class product surface. |
| D-8 | Desktop config sharing | Decided: plan Desktop QR/deep-link support in addition to file import/export, with redaction and size limits. | Cross-device setup is part of the product experience. |
| D-9 | Repo governance level | Decided: add lightweight professional governance: PR template, issue templates, ADRs, changelog, release checklist, contributing guide. | Enough structure to maintain quality without turning the repo bureaucratic. |
| D-10 | Large UI refactor timing | Refactor after contract tests and screenshot baselines exist | Avoids moving thousands of lines while behavior is still under-specified. |
| D-11 | Release artifacts in repo | Decided: CI/release workflow is the source of truth; keep `dist/` / `releases/` only as backup/archive material with clear labeling and artifact policy. | Balances backup convenience with source-tree cleanliness and release trust. |
| D-12 | Desktop tab labels | Decided: shift toward user-centered labels such as `Connect`, while keeping technical labels in secondary/help text. | Improves first-run comprehension without hiding technical concepts. |
| D-13 | Screenshot policy | Decided: avoid screenshots in core docs unless they are regenerated per release; stale screenshots should be removed or clearly historical. | Prevents outdated visuals from becoming product misinformation. |
| D-14 | Doctor UI | Decided: Doctor should become structured enough to drive Desktop and Android summary cards. | Troubleshooting should start with clear state and next action, not raw logs. |
| D-15 | Dirty config action | Decided: when fields are dirty, show `Save and start` / `Save and connect` as the primary action while keeping explicit save available. | Reduces failed starts caused by unsaved edits. |
| D-16 | Legacy Android config migration | Decided: support legacy top-level `script_ids` / `auth_key` in Android import and add a narrow Rust legacy compatibility path. | Maximizes backward compatibility while keeping canonical `account_groups` as the future shape. |
| D-17 | `enable_batching` status | Decided: keep as an active advanced/experimental field, preserve it in config, document it, and keep it out of first-run UI. | Existing backend/docs references should not be silently broken. |
| D-18 | Status renderer ownership | Decided: create a shared canonical Rust status snapshot with UI-specific projections for Desktop, Android, `/status`, and support bundles. | Prevents status and support surfaces from drifting. |
| D-19 | Mode capability richness | Decided: each mode should expose all relevant parameters, capabilities, diagnostics, and limits for that mode, while unrelated mode controls stay hidden. | Every mode should feel complete without making the whole app noisy. |
| D-20 | Android package visibility | Decided: investigate narrowing `QUERY_ALL_PACKAGES`; if not feasible, keep it and document the privacy rationale clearly. | Balances app-splitting capability with user trust. |
| D-21 | Backend helper version markers | Decided: add lightweight helper version/compatibility markers where feasible. | Makes backend skew diagnosable. |
| D-22 | CI gate strictness | Decided: use core blocking release gates plus advisory extended checks. | Keeps releases safe without making broad multi-platform CI brittle. |
| D-23 | Stale/deprecated cleanup policy | Decided: do not keep stale or deprecated code, functions, parameters, docs, screenshots, examples, or generated artifacts unless they are explicitly documented compatibility surfaces with tests. Every completed item needs a cleanup/garbage-collection pass. | Keeps the repo neat, professional, and maintainable. |

## North Star UX

The ideal user path:

1. User opens app.
2. App shows current state in plain language:
   - Mode
   - Ready/not ready
   - CA trust
   - Relay config status
   - Network/SNI reachability
   - Running/stopped
3. App shows exactly one primary next action.
4. User completes mode-specific setup through a guided surface.
5. App runs diagnostics and explains failures in task language.
6. Once connected, app shifts from setup to monitoring:
   - Traffic status
   - Quota pressure
   - Failures
   - Active deployments
   - Copyable proxy endpoints
7. Advanced tuning remains available but does not compete with the main flow.

## Mode-Specific Richness Rule

Each mode should feel complete on its own. The UI should not expose every
product capability at once, but once a user chooses a mode, that mode should
surface all of its relevant setup fields, diagnostics, limits, safety warnings,
backend health, and documentation links.

Mode experience targets:

| Mode | Rich mode surface should include |
|---|---|
| Apps Script | Account group setup, auth key, script IDs, quota/concurrency explanation, CA trust, test relay, blacklist/quota diagnostics, Code.gs links, support bundle fields. |
| Serverless JSON | Base URL, relay path, auth key, health check, non-JSON/protection-page diagnosis, max body/batching advanced fields, Vercel/Netlify/Cloudflare helper links. |
| Direct fronting | Google IP, front domain, SNI pool, fronting groups, domain overrides, passthrough hosts, scanner/test tools, route explanations, block-QUIC guidance. |
| Full tunnel | Apps Script full-mode group, CodeFull, tunnel-node URL/auth/health/version, UDP/udpgw limits, cloud deploy docs, backend state, full-mode support bundle fields. |
| Android VPN/TUN | VPN state, foreground service state, split tunneling, package visibility rationale, CA/user-trust limits, per-app routing, import/export/deep-link state. |

Cross-mode rule:

- Hide irrelevant controls until their mode is selected.
- Preserve advanced fields from other modes during import/export.
- Keep mode-specific Advanced sections available for users who need the full
  capability set.
- Docs and help should route users to the selected mode's exact backend and
  troubleshooting path.

## Design Principles

### Progressive Disclosure

Show what is needed now. Hide detail until the user asks or the state requires
it.

Examples:

- First-run: show mode, required credentials, CA, diagnostics.
- Running: show health, endpoints, traffic, quota, stop action.
- Failed: show diagnosis, reason, and repair action.
- Expert: expose SNI pool, rate limits, domain overrides, profiles, and logs.

### One Primary Action

At any moment, the primary action should be obvious:

- `Start`
- `Stop`
- `Save and start`
- `Install CA`
- `Run Doctor`
- `Fix config`
- `Copy proxy endpoint`

Avoid presenting equal visual weight for Save, Start, Doctor, Test, Walkthrough,
Install CA, Check CA, and update check at the same time.

### State Before Settings

The user should not have to inspect fields to understand the current state.
The UI should summarize readiness.

Readiness examples:

- "Apps Script mode needs at least one enabled account group."
- "CA is not trusted on this machine."
- "SNI probe has no successful host yet."
- "Running locally on HTTP 127.0.0.1:8085 and SOCKS5 127.0.0.1:8086."
- "LAN sharing is exposed; allowlist is missing."

### Safety Is UX

This product touches local certificates, proxy exposure, LAN sharing, quota, and
traffic routing. Safety warnings should be actionable and contextual.

Avoid generic warnings like "Be careful." Prefer:

- "LAN sharing is on and no allowed IPs are configured. SOCKS5 clients will be
  rejected until you add an allowlist."
- "Turning off TLS verification can make the outer hop vulnerable to
  interception. Use only while diagnosing handshake failures."
- "This mode decrypts HTTPS locally. Only install the CA on devices you
  control."

### Same Concepts, Same Words

Use the same terms across UI, docs, logs, and CLI:

- Apps Script
- Serverless JSON
- Direct fronting
- Full tunnel
- Account group
- Deployment ID
- Auth key
- Front domain
- Google IP
- SNI pool
- MITM CA
- Doctor
- Test relay
- Proxy-only
- VPN/TUN

## Target Information Architecture

### Desktop App Target

Recommended top-level structure:

| Region | Purpose | Notes |
|---|---|---|
| Fixed command center | Current mode, readiness, running state, primary action, key secondary actions | Should not scroll away. |
| Setup tab | Mode-specific setup and first-run wizard | Must be sufficient for a basic working config. |
| Network tab | Google IP, front domain, ports, LAN sharing, endpoints, SNI tools | Network concepts stay together. |
| Monitor tab | Traffic, quota, failures, per-site stats, logs, update status | Running-state dashboard. |
| Advanced tab | Expert tuning, profiles, account pool weighting, domain overrides | No nested "Advanced" gate inside the Advanced tab. |
| Help tab | Short contextual guide plus links to canonical docs | Not a duplicate README. |

Potential tab labels:

Option A, conservative:

- Setup
- Network
- Advanced
- Monitor
- Help

Option B, more user-centered:

- Connect
- Setup
- Network
- Monitor
- Advanced
- Help

Recommendation: keep existing labels initially to reduce churn, but reshape
their contents. Consider renaming `Setup` to `Connect` only after the fixed
command center exists.

### Android App Target

Recommended top-level structure:

| Region | Purpose | Notes |
|---|---|---|
| Top app bar | Product name, language, version/update | Keep compact. |
| Status summary | Running state, mode, readiness, CA, route | Always near top. |
| Sticky bottom action | Connect/Disconnect or next blocking action | Mobile needs the main action available after scrolling. |
| Guided setup sections | Mode, relay credentials, network, CA, diagnostics | Use progressive disclosure. |
| Advanced sections | SNI, app splitting, logs, tuning | Collapsed unless relevant. |
| Help drawer/sheet | Task-based help and docs links | Avoid dumping long prose into the main form. |

## Shared Contract Map

This section defines which surface owns each concept. It exists to keep backend,
desktop frontend, Android frontend, CLI, docs, and release artifacts consistent.

### Mode Contract

| User-facing label | Config value | Backend path | Desktop expectation | Android expectation | Docs expectation |
|---|---|---|---|---|---|
| Apps Script | `apps_script` | Local MITM + Apps Script JSON fetch relay | Requires at least one enabled account group | Requires Apps Script IDs/auth key, serialized as canonical `account_groups` or migrated safely | Explain Code.gs, CA trust, quotas, account groups, Test relay. |
| Serverless JSON | `vercel_edge` | Local MITM + Vercel/Netlify/compatible JSON fetch relay | Requires `vercel.base_url`, `vercel.relay_path`, `vercel.auth_key` | Requires Base URL, relay path, AUTH_KEY | Explain that config key remains `vercel`/`vercel_edge` for compatibility even when using Netlify. |
| Direct fronting | `direct` | SNI rewrite / configured fronting groups; no relay credentials | Requires Google IP/front domain; account groups optional/ignored | Same, with mobile-specific caveat around VPN/proxy mode | Explain it is not a full proxy for every destination. |
| Full tunnel | `full` | Apps Script full deployment + tunnel-node | Requires account groups and full-mode tuning awareness; local CA not needed | Requires full-mode credentials and a clear "not proven by Test relay" warning | Explain CodeFull.gs, tunnel-node, no local MITM CA, higher latency. |
| Legacy Google-only alias | `google_only` | Parsed as Direct | Should normalize to `direct` on load/save | Should import as Direct | Mention only as legacy alias when necessary. |

Mode contract tasks:

| ID | Task | Status | Owner | Evidence |
|---|---|---|---|---|
| CM.1 | Add this mode table or equivalent to `docs/relay-modes.md` and `docs/index.md` | todo | | |
| CM.2 | Ensure desktop labels never expose `vercel_edge` as the primary label except where explaining config JSON | todo | | |
| CM.3 | Ensure Android labels match desktop labels for the four product modes | todo | | |
| CM.4 | Add a test fixture for each mode that validates in Rust and can be loaded by desktop/Android | todo | | |

### Config Field Ownership Matrix

| Field / group | Rust source | Desktop | Android | Docs/examples | Parity requirement |
|---|---|---|---|---|---|
| `mode` | Required in `Config` | Edit/select | Edit/select | All mode docs | Same accepted values and same user-facing labels. |
| `account_groups` | Canonical Apps Script/full identity | Full editor exists in Advanced; minimal editor needed in Setup | Must serialize/read canonical form or migrate legacy fields | README and examples use canonical form | No frontend should create invalid Apps Script/full config. |
| `vercel.*` | Serverless JSON settings | Edit in Setup | Edit in mobile setup | JSON relay docs | Provider-neutral user copy; schema name remains compatibility detail. |
| `google_ip`, `front_domain` | Google edge/SNI path | Edit/scan/test | Edit/auto-detect/test | Setup and troubleshooting docs | Defaults and validation differences must be documented. |
| `sni_hosts` | Optional explicit SNI pool | SNI editor | SNI editor/tester | Advanced/troubleshooting docs | Rust and Android default pool must be checked for drift. |
| `listen_host`, `listen_port`, `socks5_port` | Local listeners | Edit/copy endpoint | Mobile defaults and proxy-only endpoints | Sharing/per-app docs | Platform defaults must be explicit. |
| `lan_token`, `lan_allowlist` | LAN guardrails | Edit/warn | Not currently surfaced | Sharing docs | Android absence must be intentional and documented. |
| `fronting_groups` | Direct-mode multi-edge catalog | Preserved but not fully edited | Preserved raw JSON | Fronting group docs | UIs must not erase hand-edited groups. |
| `domain_overrides`, `passthrough_hosts` | Routing overrides | Preserved / partially explained | Passthrough surfaced; domain overrides not surfaced | Advanced docs | Decide what each frontend edits vs preserves. |
| Runtime tuning fields | Effective runtime profile helpers | Advanced editor | Partial advanced editor | Advanced docs | UI must show profile-derived values without lying about explicit overrides. |
| `block_quic`, `tunnel_doh`, `bypass_doh_hosts` | Protocol handling | Advanced editor | Advanced section | Advanced/Android docs | Labels and defaults must match backend semantics. |
| Android-only keys | Ignored by Rust serde | N/A | `connection_mode`, `split_mode`, `split_apps`, `ui_lang` | Android docs | Clearly marked as wrapper-layer settings. |

### Status / Diagnostics Contract

Target one shared status shape with these sections:

| Section | Fields | Used by |
|---|---|---|
| Runtime | running, mode, started_at/uptime, version, platform | Desktop command center, Android status summary, `/status`, support bundle |
| Endpoints | HTTP endpoint, SOCKS5 endpoint, LAN exposure, token/allowlist state | Desktop Network tab, Android Proxy-only summary, sharing docs |
| Trust | CA generated, CA trusted, mode needs CA, Firefox/NSS caveat if known | Desktop command center, Android CA card, safety docs |
| Readiness | ready/not ready, blocking reason, next action | Desktop primary action, Android sticky action, wizard |
| Relay stats | relay calls, failures, bytes, cache, per-site, active scripts, degraded state | Monitor, Android usage card, support bundle |
| Diagnostics | Doctor item IDs, severity, detail, fix text | CLI, desktop Doctor card, Android diagnostics sheet |
| Update | current version, latest version, route used, asset availability | Desktop update check, Android update check, release docs |

Status contract tasks:

| ID | Task | Status | Owner | Evidence |
|---|---|---|---|---|
| CS.1 | Define `StatusSnapshot` / `ReadinessSnapshot` structs in Rust, separate from UI code | todo | | |
| CS.2 | Render status JSON through one shared function used by status API, JNI, support bundle, and desktop | todo | | |
| CS.3 | Give every readiness failure a stable ID and user-facing repair action | todo | | |
| CS.4 | Map Doctor item IDs to UI cards instead of only log lines | todo | | |
| CS.5 | Add fixtures for stopped, ready, missing credentials, CA missing, relay failing, degraded, LAN-exposed | todo | | |

## Workstream A: Desktop UI/UX Elevation

### A0. Desktop UI Baseline And Inventory

Purpose: capture the current visible product before changes.

Tasks:

| ID | Task | Status | Owner | Evidence |
|---|---|---|---|---|
| A0.1 | Capture current desktop screenshots at default size, short height, and narrow min width | todo | | |
| A0.2 | Capture first-run empty config state | todo | | |
| A0.3 | Capture configured but stopped state | todo | | |
| A0.4 | Capture running state with traffic | todo | | |
| A0.5 | Capture failure states: bad auth key, CA missing, bad SNI, LAN exposed | todo | | |
| A0.6 | List all visible desktop actions and classify as primary, secondary, danger, diagnostic, or link | todo | | |
| A0.7 | List all desktop copy blocks longer than two lines | todo | | |

Acceptance criteria:

- Screenshots are stored in a predictable temporary or docs review folder.
- Every main state has at least one screenshot.
- Action inventory identifies duplicate or misplaced controls.
- Copy inventory identifies candidates for short inline help vs deep docs.

### A1. Fixed Command Center

Problem:

The desktop header and global controls are currently inside the central scroll
area. This makes critical status and actions disappear on smaller windows.

Target:

A fixed command center above the tab body.

Command center content:

- Product mark/name/version.
- Current mode.
- Running/stopped state.
- Readiness status.
- Primary action.
- Secondary actions:
  - Test relay
  - Doctor
  - Save config or Save and start
  - Walkthrough
- Compact status chips:
  - HTTP/SOCKS endpoint
  - LAN exposure
  - CA trust
  - runtime profile

Design requirements:

- The command center stays visible.
- It does not consume too much vertical height.
- One primary button is visually dominant.
- Danger actions such as Stop and Remove CA are clear but not chaotic.
- Status chips are meaningful, not decoration.

Tasks:

| ID | Task | Status | Owner | Evidence |
|---|---|---|---|---|
| A1.1 | Move top command center out of `CentralPanel` scroll area | todo | | |
| A1.2 | Define readiness model from current form state | todo | | |
| A1.3 | Add one-line readiness text | todo | | |
| A1.4 | Collapse duplicate Start/Test/Doctor controls in Monitor | todo | | |
| A1.5 | Add disabled-button explanation for primary action | todo | | |
| A1.6 | Verify short-window behavior | todo | | |

Acceptance criteria:

- Primary action remains visible at minimum supported window height.
- A user can tell why Start is disabled without opening logs.
- There is no duplicate Start/Stop action with equal priority lower in the UI.
- `cargo check --features ui --bin mhrv-f-ui` passes.

### A2. Setup Tab Becomes Actually Complete

Problem:

The Setup tab tells Apps Script users that credentials live under Advanced ->
Multi-account pools. That makes the main setup path feel indirect.

Target:

Setup should support a minimal working configuration for each mode.

Mode-specific setup:

| Mode | Setup tab must include |
|---|---|
| Apps Script | Add/edit at least one account group with auth key and deployment IDs. |
| Serverless JSON | Base URL, relay path, AUTH_KEY, TLS verify, quick health check. |
| Direct | Explanation, Google IP/front domain dependency, no credentials needed. |
| Full tunnel | Required Apps Script full deployment and tunnel-node checklist. |

Recommended interaction:

- Show mode cards or a mode selector with plain-language trade-offs.
- After mode selection, show only fields needed for that mode.
- Keep expert fields in Advanced.
- Use validation badges per setup block.

Tasks:

| ID | Task | Status | Owner | Evidence |
|---|---|---|---|---|
| A2.1 | Design minimal account group editor for Setup | todo | | |
| A2.2 | Keep weight/enabled/advanced pool controls in Advanced | todo | | |
| A2.3 | Add mode readiness checklist | todo | | |
| A2.4 | Add mode-specific "What to deploy" row with open buttons | todo | | |
| A2.5 | Add serverless JSON health expectation: `/api/api` must return JSON | todo | | |
| A2.6 | Add Full tunnel verification guidance without pretending Test relay proves it | todo | | |

Acceptance criteria:

- A first-time Apps Script user can enter required credentials without visiting
  Advanced.
- Setup tab shows only relevant credential fields for the selected mode.
- Each mode has a clear next step.
- Advanced account pool features remain available.

### A3. First-Run Wizard Redesign

Problem:

The wizard exists, but it behaves more like a helpful overlay on top of the
same dense form. It should become a guided path with validation.

Target wizard steps:

1. Choose mode.
2. Deploy backend.
3. Enter credentials.
4. Check CA/trust.
5. Check network/SNI.
6. Run Doctor/Test.
7. Start and verify browsing.

Tasks:

| ID | Task | Status | Owner | Evidence |
|---|---|---|---|---|
| A3.1 | Replace current four-step wizard with state-aware checklist | todo | | |
| A3.2 | Add step completion states | todo | | |
| A3.3 | Add "Skip for now" only where safe | todo | | |
| A3.4 | Add "Open guide" links to canonical docs per step | todo | | |
| A3.5 | Persist dismissed/completed wizard state intentionally | todo | | |

Acceptance criteria:

- Wizard guides the user through a full working setup.
- Completed steps remain visibly completed.
- Failed steps show a specific repair action.
- The wizard does not duplicate large chunks of docs.

### A4. Form System And Layout Quality

Problem:

Desktop form rows use fixed label widths and repeated inline layout. This can
break under narrow widths and makes future polishing slow.

Target:

A small UI component system for `egui`:

- `FormRow`
- `Section`
- `StatusChip`
- `Callout`
- `PrimaryAction`
- `DangerAction`
- `CopyableEndpoint`
- `MetricCard`
- `DataTable`
- `ValidationMessage`

Tasks:

| ID | Task | Status | Owner | Evidence |
|---|---|---|---|---|
| A4.1 | Split reusable UI primitives out of `src/bin/ui.rs` | todo | | |
| A4.2 | Make form rows responsive below medium width | todo | | |
| A4.3 | Standardize buttons by priority and danger level | todo | | |
| A4.4 | Standardize warning, success, info, and danger callouts | todo | | |
| A4.5 | Audit all text inputs for labels, hints, and validation | todo | | |
| A4.6 | Audit all checkboxes and toggles for plain-language labels | todo | | |

Acceptance criteria:

- New screens use shared primitives instead of one-off styling.
- Labels do not clip at minimum width.
- Buttons have consistent visual priority.
- Warnings use consistent color, copy structure, and action.

### A5. Monitor As Health Dashboard

Problem:

Monitor has useful stats, but the user needs interpretation: healthy, degraded,
quota risk, failing relay, bad SNI, CA issue, or LAN exposure.

Target:

Monitor should answer:

- Is it running?
- Is traffic flowing?
- Is quota pressure rising?
- Are failures increasing?
- Which hosts are expensive or failing?
- What should I do next?

Recommended widgets:

- Health summary.
- Traffic volume.
- Relay calls/failures.
- Cache hit rate.
- Quota pressure.
- Active deployments.
- Degrade state and reason.
- Recent notable events.
- Per-site table.
- Recent logs collapsed by default, expandable.

Tasks:

| ID | Task | Status | Owner | Evidence |
|---|---|---|---|---|
| A5.1 | Convert raw stats grid into grouped metric cards | todo | | |
| A5.2 | Add quota pressure interpretation | todo | | |
| A5.3 | Add "recommended action" based on failure/degrade state | todo | | |
| A5.4 | Make logs a diagnostic detail, not the main feedback surface | todo | | |
| A5.5 | Improve per-site table density and scanning | todo | | |
| A5.6 | Add empty states for stopped/no-traffic/no-stats | todo | | |

Acceptance criteria:

- A running user can understand health without reading logs.
- Failure states provide concrete next actions.
- Logs remain available and copyable.

### A6. Network And LAN Sharing UX

Problem:

Network settings include safety-sensitive controls. LAN sharing must be
understood before activation.

Target:

Network tab should clearly distinguish:

- Local-only proxy.
- LAN-shared proxy.
- HTTP token protection.
- SOCKS5 allowlist protection.
- Copyable local endpoints.
- Copyable LAN endpoints.

Tasks:

| ID | Task | Status | Owner | Evidence |
|---|---|---|---|---|
| A6.1 | Add LAN sharing state card | todo | | |
| A6.2 | Add copyable endpoint component | todo | | |
| A6.3 | Warn when LAN sharing lacks token/allowlist | todo | | |
| A6.4 | Make SOCKS5 token limitation visible | todo | | |
| A6.5 | Add "local only" reset action | todo | | |

Acceptance criteria:

- Users cannot accidentally miss that LAN sharing is exposed.
- The UI explains why SOCKS5 needs allowlist protection.
- Copying endpoints is easy.

### A7. Advanced Tab Restructure

Problem:

The Advanced tab currently contains a collapsed Advanced section and many knobs
with long explanations.

Target:

Advanced should be a well-grouped expert workspace.

Suggested groups:

- Runtime profile and auto-tune.
- Speed vs quota.
- Reliability and outage reset.
- Account groups and weighting.
- Routing overrides.
- DNS/QUIC behavior.
- Profiles.
- Logs/debug settings.

Tasks:

| ID | Task | Status | Owner | Evidence |
|---|---|---|---|---|
| A7.1 | Remove nested Advanced collapse inside Advanced tab | todo | | |
| A7.2 | Group knobs by user intent | todo | | |
| A7.3 | Add default/recommended/current values where useful | todo | | |
| A7.4 | Add "reset advanced to safe defaults" if config APIs support it | todo | | |
| A7.5 | Move long tuning explanations to docs links | todo | | |

Acceptance criteria:

- Advanced users can find related knobs quickly.
- First-time users are not pushed into tuning.
- Each dangerous knob explains risk briefly.

## Workstream B: Android UI/UX Elevation

### B0. Android Baseline

Tasks:

| ID | Task | Status | Owner | Evidence |
|---|---|---|---|---|
| B0.1 | Capture Android screenshots in English: fresh, configured, running, error | todo | | |
| B0.2 | Capture Android screenshots in Persian/RTL | todo | | |
| B0.3 | List all hard-coded visible copy in Kotlin | todo | | |
| B0.4 | List string key mismatches between `values` and `values-fa` | todo | | |

Acceptance criteria:

- Baseline screenshots exist for mobile UX review.
- Localization gaps are tracked with exact keys.

### B1. Mobile Connection Summary

Problem:

The mobile screen starts with explanatory content and form controls. It needs a
compact, state-first summary.

Target:

Add a top status summary after the app bar:

- Mode.
- Connection mode: VPN/TUN or Proxy-only.
- Running state.
- Required setup missing.
- CA status if known.
- Proxy endpoint when connected.

Tasks:

| ID | Task | Status | Owner | Evidence |
|---|---|---|---|---|
| B1.1 | Design `ConnectionSummaryCard` | todo | | |
| B1.2 | Add readiness text | todo | | |
| B1.3 | Show proxy endpoints only when useful | todo | | |
| B1.4 | Add visual state for running/stopped/blocked | todo | | |

Acceptance criteria:

- User sees state before settings.
- Connect disabled state is explained.

### B2. Sticky Mobile Primary Action

Problem:

The Connect button appears near the top but the screen is long. On mobile, the
main action should remain reachable.

Target:

Use a sticky bottom action area:

- Connect.
- Disconnect.
- Install CA if that is the blocking next step.
- Run SNI test or open diagnostics if that is the blocking next step.

Tasks:

| ID | Task | Status | Owner | Evidence |
|---|---|---|---|---|
| B2.1 | Add bottom action bar to `Scaffold` | todo | | |
| B2.2 | Make action label state-aware | todo | | |
| B2.3 | Avoid overlap with scroll content | todo | | |
| B2.4 | Verify on small screens | todo | | |

Acceptance criteria:

- Connect/Disconnect stays reachable after scrolling.
- Disabled state has visible explanation nearby.

### B3. Android Guided Setup

Target:

Turn the single long setup page into a guided, collapsible flow:

1. Mode.
2. Credentials for selected mode.
3. Network.
4. Certificate.
5. SNI tester.
6. App routing.
7. Logs and help.

Tasks:

| ID | Task | Status | Owner | Evidence |
|---|---|---|---|---|
| B3.1 | Add completion indicators to collapsible sections | todo | | |
| B3.2 | Expand only the next incomplete required section by default | todo | | |
| B3.3 | Keep advanced sections collapsed unless relevant | todo | | |
| B3.4 | Move long help into help sheet/docs links | todo | | |

Acceptance criteria:

- Fresh install shows a short, obvious path.
- Returning users do not have to scroll through first-run prose.

### B4. Android Localization And RTL

Problem:

Persian resources are missing keys and Kotlin contains hard-coded visible copy.

Tasks:

| ID | Task | Status | Owner | Evidence |
|---|---|---|---|---|
| B4.1 | Move mode overview copy to string resources | todo | | |
| B4.2 | Move labels like `Block QUIC`, coalesce labels, and dialog labels to resources | todo | | |
| B4.3 | Add missing Persian keys | todo | | |
| B4.4 | Add script/check for values vs values-fa parity | todo | | |
| B4.5 | Review RTL layout for app bar, rows, and long technical tokens | todo | | |

Acceptance criteria:

- No user-facing Android copy is hard-coded unless intentionally nonlocalized.
- Persian key parity is enforced or easy to check.
- RTL screenshots show no clipped or awkward layouts.

## Workstream C: Documentation, Guides, And Help Elevation

### C0. Documentation Architecture Decision

Problem:

Docs are broad but dispersed. Some overlap is useful, but uncontrolled overlap
causes stale instructions and contradictory setup paths.

Target:

Define canonical docs by user question:

| User question | Canonical doc | Supporting docs |
|---|---|---|
| Where do I start? | `docs/index.md` | README fast path |
| Which mode should I use? | `docs/relay-modes.md` | In-app mode cards |
| How do I set up desktop? | README setup + `docs/ui-desktop.md` | Troubleshooting, Doctor |
| How do I set up Android? | `docs/android.md` | Sharing/per-app, safety |
| Why is it broken? | `docs/troubleshooting.md` | `docs/doctor.md`, logs |
| What does this advanced knob do? | `docs/advanced-options.md` | In-app short help |
| Is this safe? | `docs/safety-security.md` | CA dialogs, README |
| How do I share/proxy per app? | `docs/sharing-and-per-app-routing.md` | Android guide, Network tab |
| What backend do I deploy? | mode-specific backend guide | relay modes |
| What does this term mean? | `docs/glossary.md` | Inline tooltips |

Tasks:

| ID | Task | Status | Owner | Evidence |
|---|---|---|---|---|
| C0.1 | Create documentation map with canonical owner for each topic | todo | | |
| C0.2 | Mark duplicate/overlapping sections for consolidation | todo | | |
| C0.3 | Decide which docs are user-facing vs maintainer/reference | todo | | |
| C0.4 | Add "canonical source" notes where overlap must remain | todo | | |

Acceptance criteria:

- Every major topic has one canonical location.
- Duplicated explanations either link to canonical docs or are intentionally summarized.

### C1. Documentation Consolidation Candidates

Recommended consolidation:

| Current area | Issue | Recommendation |
|---|---|---|
| README setup and `docs/index.md` | Both orient new users | Keep README as release-facing overview and `docs/index.md` as docs hub. Cross-link clearly. |
| `docs/relay-modes.md` and mode sections in README/Android/UI docs | Repeated mode explanations | Make `relay-modes.md` canonical; other surfaces use compact summaries. |
| `docs/troubleshooting.md`, `docs/doctor.md`, in-app Help troubleshooting | Related but separate | Keep `troubleshooting.md` as symptom tree, `doctor.md` as tool reference, in-app help links to both. |
| `docs/advanced-options.md` and UI hover prose | UI has long tuning explanations | Keep advanced doc canonical; UI should show short risk and link. |
| Vercel/Netlify JSON and XHTTP docs | Similar provider names cause confusion | Add consistent "Native JSON vs external XHTTP" banner to each relevant doc. |
| Persian docs | Some are summaries, not equivalents | Decide parity target: full translation for core docs, summary for niche maintainer docs. |
| Screenshot references | No current `docs/*.png` asset was found; old screenshot references may still exist in docs, release notes, or generated artifacts | Remove stale references, replace with generated fresh screenshots, or mark historical images clearly. |

Tasks:

| ID | Task | Status | Owner | Evidence |
|---|---|---|---|---|
| C1.1 | Add a "Docs Map" section to `docs/index.md` | todo | | |
| C1.2 | Add JSON vs XHTTP warning banner to Vercel/Netlify docs | todo | | |
| C1.3 | Rewrite README setup links to point to canonical guides | todo | | |
| C1.4 | Audit screenshot references and either replace, remove, or mark them historical | todo | | |
| C1.5 | Add "last reviewed" metadata to major docs | todo | | |

Acceptance criteria:

- Users can navigate docs without guessing which guide is latest.
- Native JSON and XHTTP helper confusion is reduced.
- No stale screenshot reference implies an old product identity.

### C2. In-App Help Strategy

Problem:

In-app help is valuable, but too much prose inside the app turns setup into a
reading task.

Target:

Use three layers:

1. Inline hint: one sentence near a control.
2. Contextual callout: short explanation plus action.
3. Deep docs link: full guide.

Rules:

- Inline help explains what to do now.
- Docs explain why, edge cases, and examples.
- Logs explain what happened.
- Doctor explains what to fix.

Tasks:

| ID | Task | Status | Owner | Evidence |
|---|---|---|---|---|
| C2.1 | Inventory desktop Help tab copy | todo | | |
| C2.2 | Inventory Android guide strings | todo | | |
| C2.3 | Convert long in-app prose to short summaries plus doc links | todo | | |
| C2.4 | Ensure each mode has a direct doc link | todo | | |
| C2.5 | Ensure CA dialogs link or refer to safety docs | todo | | |

Acceptance criteria:

- App surfaces are readable without becoming manuals.
- Deep explanations remain one click away.

### C3. Persian And Localization Documentation Plan

Recommended policy:

Core docs needing strong Persian parity:

- `docs/index.fa.md`
- `docs/android.fa.md`
- `docs/ui-desktop.fa.md`
- `docs/troubleshooting.fa.md`
- `docs/doctor.fa.md`
- `docs/safety-security.fa.md`
- `docs/advanced-options.fa.md`
- `docs/glossary.fa.md`

Docs that can be summary-only unless demand grows:

- provider-specific backend docs
- field notes
- platform alternatives
- reference audits
- installer details

Tasks:

| ID | Task | Status | Owner | Evidence |
|---|---|---|---|---|
| C3.1 | Decide parity level per doc: full, summary, or English-only | todo | | |
| C3.2 | Expand `ui-desktop.fa.md` to match modern desktop UI | todo | | |
| C3.3 | Expand `troubleshooting.fa.md` for core symptoms | todo | | |
| C3.4 | Expand `doctor.fa.md` to match English structure | todo | | |
| C3.5 | Add missing Persian Android strings | todo | | |
| C3.6 | Add glossary entries for all UI terms used in Persian docs | todo | | |

Acceptance criteria:

- Persian users have complete guidance for core workflows.
- Technical terms are consistent across UI and docs.

### C4. Documentation Rewrite Quality Bar

Every polished guide should have:

- Who this is for.
- When to use this path.
- Prerequisites.
- Step-by-step setup.
- Verification.
- Common failures.
- Security/trust notes where relevant.
- Links to canonical next docs.
- Date or version reviewed.

Writing style:

- Prefer short paragraphs.
- Prefer task headings over abstract headings.
- Avoid repeating long warnings everywhere.
- Keep examples concrete.
- Keep mode names exact.
- Be explicit about what a tool is not.

## Workstream D: Visual Design System

### D1. Brand And Theme

Current theme:

- Dark, warm-neutral panels.
- Blue accent.
- Green success.
- Red error.
- Amber warnings.
- Existing SVG mark in `assets/logo/frankestein-mark.svg`.

Target:

Maintain dark professional utility feel, but make hierarchy cleaner.

Recommendations:

- Use the logo mark consistently in desktop, Android, release docs, and maybe
  installer surfaces.
- Reduce brown/orange dominance in base surfaces.
- Use a neutral dark slate base with clear blue/mint/amber/red semantics.
- Keep corners restrained.
- Avoid decorative gradients unless they carry status or brand identity.

Tasks:

| ID | Task | Status | Owner | Evidence |
|---|---|---|---|---|
| D1.1 | Define shared color tokens for desktop and Android | todo | | |
| D1.2 | Align Android theme values with desktop tokens | todo | | |
| D1.3 | Add logo mark to desktop header | todo | | |
| D1.4 | Audit contrast for text, chips, warnings, and disabled states | todo | | |
| D1.5 | Define icon usage rules | todo | | |

Acceptance criteria:

- Desktop and Android feel like the same product.
- Status colors are consistent and accessible.
- Brand mark appears without consuming excessive space.

### D2. Typography And Density

Target:

- Clear hierarchy.
- No clipped labels.
- Monospace only for values, logs, endpoints, IDs, and code.
- Compact but not cramped controls.
- Long technical copy moved out of primary flow.

Tasks:

| ID | Task | Status | Owner | Evidence |
|---|---|---|---|---|
| D2.1 | Define desktop type scale for heading/body/label/mono/button | todo | | |
| D2.2 | Define Android type usage by Material role | todo | | |
| D2.3 | Audit button text wrapping | todo | | |
| D2.4 | Audit long IDs/URLs in narrow layouts | todo | | |

Acceptance criteria:

- Text remains readable at min supported desktop width.
- Android long strings do not break layout in English or Persian.

## Workstream E: Code Architecture For UI Maintainability

### E1. Desktop UI Modularization

Suggested structure:

```text
src/bin/ui.rs                 # app entry/composition glue
src/ui/theme.rs               # colors, spacing, text styles
src/ui/components.rs          # shared egui primitives
src/ui/state.rs               # form/readiness/view state helpers
src/ui/views/setup.rs
src/ui/views/network.rs
src/ui/views/advanced.rs
src/ui/views/monitor.rs
src/ui/views/help.rs
src/ui/views/wizard.rs
```

If keeping modules under `src/bin/` is simpler, use a local `ui/` module
folder next to `ui.rs`. Follow Rust module conventions that build cleanly with
the current binary target.

Tasks:

| ID | Task | Status | Owner | Evidence |
|---|---|---|---|---|
| E1.1 | Extract theme constants and theme application | todo | | |
| E1.2 | Extract reusable components | todo | | |
| E1.3 | Extract each tab into a view function/module | todo | | |
| E1.4 | Extract readiness computation into pure helpers | todo | | |
| E1.5 | Add focused tests for readiness/config validation where practical | todo | | |

Acceptance criteria:

- `ui.rs` becomes composition glue rather than a monolith.
- Behavior does not change during extraction unless planned.
- Build passes after each extraction slice.

### E2. Android UI Modularization

Suggested structure:

```text
ui/HomeScreen.kt              # high-level screen composition
ui/components/
ui/sections/
ui/state/
ui/help/
```

Tasks:

| ID | Task | Status | Owner | Evidence |
|---|---|---|---|---|
| E2.1 | Extract mode overview and status summary components | todo | | |
| E2.2 | Extract setup sections | todo | | |
| E2.3 | Extract logs and usage cards | todo | | |
| E2.4 | Extract localization helpers if needed | todo | | |

Acceptance criteria:

- `HomeScreen.kt` becomes easier to navigate.
- No user-visible regression.
- Compose previews can be added for key states if practical.

## Workstream F: Verification And Quality Gates

### F1. UI Screenshot QA

Desktop viewports:

- Default: 900 x 968.
- Short laptop: 1366 x 768.
- Minimum: 560 x 520.
- Narrow content stress case.

Android states:

- Small phone.
- Large phone.
- English LTR.
- Persian RTL.
- Fresh install.
- Configured stopped.
- Running.
- Error.

Tasks:

| ID | Task | Status | Owner | Evidence |
|---|---|---|---|---|
| F1.1 | Define screenshot capture commands/process | todo | | |
| F1.2 | Store review screenshots consistently | todo | | |
| F1.3 | Add checklist for visual QA | todo | | |
| F1.4 | Verify no stale screenshots ship as current docs | todo | | |

Visual QA checklist:

- Primary action visible.
- No clipped text.
- No overlapping controls.
- Disabled state explained.
- Warnings visible but not noisy.
- Logs do not dominate first-run flow.
- Long URLs and IDs wrap or scroll safely.
- Persian/RTL layout is usable.

### F2. Build And Static Checks

Required checks:

```bash
cargo check --features ui --bin mhrv-f-ui
```

Recommended Android checks:

```bash
cd android
./gradlew assembleDebug
```

Recommended docs checks:

- Markdown link check if tooling exists.
- Android string parity check.
- Search for stale product names such as `mhrv-rs`.
- Search for old version references such as `0.4.1`.

Tasks:

| ID | Task | Status | Owner | Evidence |
|---|---|---|---|---|
| F2.1 | Add a script for Android string parity | todo | | |
| F2.2 | Add a docs stale-name scan | todo | | |
| F2.3 | Add a docs link check if practical | todo | | |
| F2.4 | Record manual QA steps in release checklist | todo | | |

## Workstream G: Backend / Frontend Contract Parity

Goal:

Make the Rust backend contract, Desktop UI, Android UI, CLI, examples, docs, and
release checks behave as one product instead of loosely synchronized surfaces.

### G0. Contract Inventory And Ownership

Problem:

The project has a rich backend schema and multiple frontends. Without a formal
field/mode/status owner map, a change can compile while silently disappearing
from one UI or being serialized in a legacy shape.

Tasks:

| ID | Task | Status | Owner | Evidence |
|---|---|---|---|---|
| G0.1 | Create a canonical contract inventory covering `Mode`, `Config`, account groups, Vercel/serverless settings, fronting groups, domain overrides, LAN settings, outage reset, and status/stats | todo | | |
| G0.2 | For each contract item, name the Rust source of truth, Desktop consumer, Android consumer, CLI/docs consumer, and test owner | todo | | |
| G0.3 | Classify each `Config` field as `edit`, `preserve`, `default`, `derived`, `deprecated`, or `platform-only` for Desktop | todo | | |
| G0.4 | Classify each `Config` field as `edit`, `preserve`, `default`, `derived`, `deprecated`, or `platform-only` for Android | todo | | |
| G0.5 | Add a fixture set for representative configs: Apps Script single group, Apps Script multi-group, Serverless JSON, direct fronting, full tunnel, LAN sharing, domain overrides, advanced tuning, and legacy Android format | todo | | |
| G0.6 | Add a static parity check that compares Rust `Config` fields with Desktop `ConfigWire` fields and forces an explicit allowlist for intentional omissions | todo | | |
| G0.7 | Add a static parity check that compares Rust `Config` fields with Android `MhrvConfig` load/save handling and forces an explicit allowlist for intentional omissions | todo | | |
| G0.8 | Add a field-change checklist to PR/release docs: schema, validation, Desktop, Android, CLI, docs, examples, tests, migration | todo | | |
| G0.9 | Decide and document unknown-key policy for Desktop and Android config editing | todo | | |
| G0.10 | Decide and document Android-only keys such as `connection_mode`, `split_mode`, `split_apps`, `ui_lang`, and how they coexist with shared JSON | todo | | |

Acceptance criteria:

- Every backend-facing field has an owner and a frontend behavior.
- No new backend field can be added without a parity decision.
- Desktop and Android intentional omissions are visible, reviewed, and tested.
- Fixture configs are easy to run through all supported surfaces.

### G1. Config Serialization And Round-Trip Parity

Problem:

Config is the highest-risk parity point. Android currently writes legacy
Apps Script keys while Rust validation expects canonical `account_groups`.
Desktop is closer to canonical but its `ConfigWire` does not include every
backend field.

Tasks:

| ID | Task | Status | Owner | Evidence |
|---|---|---|---|---|
| G1.1 | Update Android `toJson()` to serialize Apps Script credentials as canonical `account_groups` for `apps_script` and `full` modes | review | | Implemented in `ConfigStore.kt`; local Gradle execution intentionally skipped by maintainer policy. |
| G1.2 | Update Android `loadFromJson()` to read canonical `account_groups` and map the first simple enabled group into current mobile fields | review | | Implemented in `ConfigStore.loadFromJson()` and shared load path. |
| G1.3 | Preserve complex Android-imported `account_groups` JSON when the mobile UI cannot fully edit multiple groups | review | | Added `preservedAccountGroupsJson` and preservation logic; tests added. |
| G1.4 | Add a migration path for legacy top-level Android `script_ids` / `auth_key`, either in Android import or Rust config loading, and document the chosen boundary | review | | Android import implemented in Batch 1; Rust loader migration added in `src/config.rs` with tests for legacy migration and canonical precedence. |
| G1.5 | Add Android tests for legacy Apps Script config import and canonical Apps Script config export | review | | Added `ConfigStoreTest`; not run locally because Gradle download/install is disallowed. |
| G1.6 | Add Android tests for full-mode Apps Script group export, because full mode also depends on `account_groups` | review | | Added full-mode canonical export test in `ConfigStoreTest`. |
| G1.7 | Add Android tests for Serverless JSON fields: `vercel.base_url`, `vercel.relay_path`, `vercel.auth_key`, and optional `vercel.max_body_bytes` | todo | | |
| G1.8 | Add Android tests for direct fronting fields: `google_ip`, `front_domain`, `sni_hosts`, `fronting_groups`, `domain_overrides`, and `passthrough_hosts` preservation | todo | | |
| G1.9 | Add Android tests for shared tuning fields that are not edited on mobile but should not be destroyed on import/export | todo | | |
| G1.10 | Add `domain_overrides` to Desktop `ConfigWire`, `to_config`, load, save, and round-trip tests | review | | `domain_overrides` already loaded/form-preserved; added `ConfigWire` serialization and UI binary test. |
| G1.11 | Decide whether `enable_batching` should be restored to Desktop `ConfigWire`, deprecated, or intentionally hidden with a documented default | review | | Preserved top-level `enable_batching`; added UI preservation for `vercel.enable_batching` and an Advanced compatibility toggle. |
| G1.12 | Add Desktop config round-trip tests for every fixture in G0.5 | review | | Added focused `ConfigWire` regression test plus broad all-current-field serialization guard; root/docs fixture matrix remains a later expansion under example validation. |
| G1.13 | Add CLI `init-config` / example config validation tests for every supported mode | todo | | |
| G1.14 | Add a config redaction helper that removes auth keys before support bundles, logs, screenshots, and docs examples | todo | | |
| G1.15 | Make Desktop and Android config import warnings explicit when unsupported advanced fields are preserved but not editable | todo | | |
| G1.16 | Add QR/share/deep-link config tests for Android `mhrvf://` and legacy `mhrv-rs://` links | todo | | |
| G1.17 | Verify config examples under the repo root and `docs/` parse with the same Rust loader used at runtime | todo | | |
| G1.18 | Document config version behavior and migration order for old files, Android exports, and Desktop saves | todo | | |

Acceptance criteria:

- Android never creates an invalid Apps Script/full config for the current Rust
  validator.
- Desktop saves do not drop `domain_overrides`.
- Advanced fields are either editable or preserved, not silently erased.
- Legacy Android config remains importable.
- Config examples are tested, not trusted by inspection alone.

### G2. Readiness And Validation Parity

Problem:

The native validator, Desktop disabled-state logic, Android Connect-state logic,
Doctor, and docs should agree on what makes each mode ready.

Tasks:

| ID | Task | Status | Owner | Evidence |
|---|---|---|---|---|
| G2.1 | Create a stable readiness ID list, for example `missing_account_group`, `missing_serverless_base_url`, `missing_google_ip`, `missing_ca`, `lan_token_missing`, `unsafe_lan_exposure` | todo | | |
| G2.2 | Map `Config::validate()` failures to readiness IDs instead of free-form UI-only messages | todo | | |
| G2.3 | Add Desktop readiness rendering that uses the same IDs as Rust validation | todo | | |
| G2.4 | Add Android readiness rendering that uses the same IDs as Rust validation or a generated mirror of the matrix | todo | | |
| G2.5 | Add mode-specific readiness fixtures for Apps Script, Serverless JSON, direct, full, and legacy `google_only` | todo | | |
| G2.6 | Add tests that Desktop and Android readiness examples match Rust validation outcomes | todo | | |
| G2.7 | Define readiness severity levels: blocker, warning, safety warning, optimization hint | todo | | |
| G2.8 | Ensure Start/Connect disabled states include one primary next action and avoid dumping raw validation text | todo | | |
| G2.9 | Add CA-trust readiness rules per platform: Windows, macOS, Linux/NSS/Firefox, Android user CA, and Android app trust limitations | todo | | |
| G2.10 | Add full-mode readiness rules for CodeFull, tunnel-node URL/auth, UDP support expectations, and cloud relay health | todo | | |
| G2.11 | Add LAN-sharing readiness rules for listen host, LAN token, allowlist, firewall, and exposure copy | todo | | |
| G2.12 | Ensure every readiness message has English/Persian Android copy and matching Desktop/docs wording | todo | | |

Acceptance criteria:

- A config that is invalid in Rust cannot appear ready in Desktop or Android.
- A config that is valid in Rust is not blocked by stale frontend-only rules.
- The same problem uses the same concept name across UI, CLI, and docs.

### G3. Status, Stats, Doctor, And Support Parity

Problem:

Status and troubleshooting data are currently rendered in more than one shape.
That makes Desktop Monitor, Android cards, local status JSON, CLI Doctor, and
support bundles harder to compare.

Tasks:

| ID | Task | Status | Owner | Evidence |
|---|---|---|---|---|
| G3.1 | Define a single `StatusSnapshot` JSON schema with version, runtime, mode, endpoints, readiness, trust, relay stats, diagnostics, and update state | todo | | |
| G3.2 | Refactor `/status` rendering to use the shared schema | todo | | |
| G3.3 | Refactor Android JNI `statsJson()` to return the same schema or a documented mobile projection of it | todo | | |
| G3.4 | Refactor Desktop Monitor data reads to use the same schema fields and labels | todo | | |
| G3.5 | Add support-bundle generation that includes redacted status JSON, config summary, Doctor summary, platform info, and recent logs | todo | | |
| G3.6 | Add Android support-bundle or shareable diagnostics summary, redacted by default | todo | | |
| G3.7 | Add stable Doctor item IDs and severity levels for UI display | todo | | |
| G3.8 | Map Doctor results into Desktop cards: connectivity, CA trust, config validity, backend health, LAN exposure, update status | todo | | |
| G3.9 | Map Doctor/SNI/update results into Android cards or sheets without requiring users to read logs first | todo | | |
| G3.10 | Add per-mode stats empty states so Apps Script, direct, Serverless JSON, and full tunnel do not show irrelevant counters | todo | | |
| G3.11 | Add quota and blacklist threshold labels shared by Desktop, Android, and docs | todo | | |
| G3.12 | Define per-site status table fields: host, route, backend, last success, last error, blacklist state, latency | todo | | |
| G3.13 | Include update-check result shape in the shared status/support contract | todo | | |
| G3.14 | Add JSON-schema or snapshot tests for status output | todo | | |
| G3.15 | Add docs showing how support should interpret the status fields | todo | | |

Acceptance criteria:

- Support data from CLI, Desktop, and Android can be compared without mental
  translation.
- Status output has a versioned contract.
- UI troubleshooting starts from summaries and keeps raw logs available as the
  deeper layer.

### G4. Mode And Backend Taxonomy Parity

Problem:

The repo supports Apps Script, native Serverless JSON relays, direct fronting,
full tunnel, and separate XHTTP helpers. Several names overlap, so users can
deploy the wrong backend for the mode they selected.

Tasks:

| ID | Task | Status | Owner | Evidence |
|---|---|---|---|---|
| G4.1 | Create a backend matrix with columns: app mode, config key, helper files, deploy target, auth key name, request shape, health check, docs link, supported clients | todo | | |
| G4.2 | Add a visible JSON-vs-XHTTP distinction to Setup, Help, README, and relevant docs | todo | | |
| G4.3 | Rename user-facing "Vercel" copy to "Serverless JSON" where the backend also supports Netlify or compatible endpoints | todo | | |
| G4.4 | Keep `vercel_edge` and `vercel` schema names documented as compatibility names rather than product-facing labels | todo | | |
| G4.5 | Verify `assets/apps_script/Code.gs`, `CodeFull.gs`, and `CodeCloudflareWorker.gs` are described with different purposes | todo | | |
| G4.6 | Verify `tools/vercel-json-relay`, `tools/netlify-json-relay`, and `tools/cloudflare-worker-relay` docs align with native Serverless JSON mode | todo | | |
| G4.7 | Verify `tools/vercel-xhttp-relay` is clearly documented as a separate external helper, not the native JSON relay mode | todo | | |
| G4.8 | Add Help-tab affordances that show exactly which backend file to deploy for the selected mode | todo | | |
| G4.9 | Add Android guide entries that show exactly which backend file to deploy for the selected mode | todo | | |
| G4.10 | Add CLI `doctor` checks that identify obvious backend/mode mismatches when possible | todo | | |
| G4.11 | Add docs examples for converting from one backend mode to another without mixing credentials | todo | | |
| G4.12 | Add release checklist item to review backend helper docs whenever helper files change | todo | | |

Acceptance criteria:

- A user can choose a mode and know which backend artifact to deploy.
- Provider names and config schema names are not confused with mode names.
- XHTTP is never accidentally presented as the native Serverless JSON path.

### G5. Defaults, SNI, Ports, And Platform Parity

Problem:

Some defaults differ by platform. That can be fine, but the roadmap needs to
separate intentional product decisions from drift.

Tasks:

| ID | Task | Status | Owner | Evidence |
|---|---|---|---|---|
| G5.1 | Create a platform defaults table for Desktop, CLI, Android, examples, and docs | todo | | |
| G5.2 | Decide whether Rust default `google_ip` and Android default `google_ip` should match or remain platform-specific | todo | | |
| G5.3 | Decide whether Desktop/CLI ports `8085/8086` and Android ports `8080/1081` are intentional platform defaults | todo | | |
| G5.4 | Add docs explaining the platform port difference if it remains intentional | todo | | |
| G5.5 | Add a parity check for Rust `DEFAULT_GOOGLE_SNI_POOL` and Android `DEFAULT_SNI_POOL` | todo | | |
| G5.6 | Add a docs table for default hosts, ports, proxy schemes, CA trust expectations, and Android VPN/TUN behavior | todo | | |
| G5.7 | Define default DNS/DoH behavior per platform and ensure UI labels match runtime behavior | todo | | |
| G5.8 | Define scanner defaults per platform: max IPs, batch size, validation mode, and SNI test behavior | todo | | |
| G5.9 | Define app-splitting defaults and limitations for Android `split_mode` / `split_apps` | todo | | |
| G5.10 | Add release tests that fail on unintended default drift | todo | | |

Acceptance criteria:

- Every default difference has a written reason.
- SNI drift is caught automatically.
- Docs and UI no longer make platform defaults look accidental.

### G6. Config Sharing, Import, Export, And Deep-Link Parity

Problem:

Users can move configs across Desktop, Android, CLI, QR/deep links, and docs
examples. Sharing needs clear compatibility and redaction rules.

Tasks:

| ID | Task | Status | Owner | Evidence |
|---|---|---|---|---|
| G6.1 | Specify `mhrvf://` config link schema, encoding, version, and maximum practical size | todo | | |
| G6.2 | Specify legacy `mhrv-rs://` import behavior and deprecation copy | todo | | |
| G6.3 | Define QR export limits and fallback to file/share sheet when configs exceed safe QR size | todo | | |
| G6.4 | Add redaction options for shared configs: full secret export, redacted support export, and docs-safe example export | todo | | |
| G6.5 | Ensure Android imports preserve advanced fields that mobile cannot edit | todo | | |
| G6.6 | Plan Desktop QR/deep-link import/export in addition to file import/export, including redaction, size limits, and legacy-link handling | todo | | Maintainer decision D-8. |
| G6.7 | Add config-import conflict messages for platform-only Android keys and desktop-only paths | todo | | |
| G6.8 | Add tests for config import/export across Desktop, Android, CLI, and examples | todo | | |
| G6.9 | Add docs that explain what is safe to share and what must be treated as secret | todo | | |
| G6.10 | Add release checklist entry for import/export compatibility after schema changes | todo | | |

Acceptance criteria:

- Cross-surface config sharing is explicit, tested, and redacted correctly.
- Legacy links remain handled deliberately.
- Unsupported fields are preserved or explained.

## Workstream H: Backend Helpers, Full Tunnel, And Ops Parity

Goal:

Make every backend helper deployable, testable, and documented with the same
mode vocabulary used by the apps.

### H1. Apps Script Backend Parity

Tasks:

| ID | Task | Status | Owner | Evidence |
|---|---|---|---|---|
| H1.1 | Audit `assets/apps_script/Code.gs` against Apps Script mode docs and UI copy | todo | | |
| H1.2 | Audit `assets/apps_script/CodeFull.gs` against full tunnel docs and UI copy | todo | | |
| H1.3 | Audit `assets/apps_script/CodeCloudflareWorker.gs` against Cloudflare Worker relay docs | todo | | |
| H1.4 | Verify auth header/key names match UI fields and docs examples | todo | | |
| H1.5 | Verify request/response JSON shape is documented and covered by tests where possible | todo | | |
| H1.6 | Add quota, blacklist, and account-group behavior to Setup/Help/docs in the same words | todo | | |
| H1.7 | Add version marker or compatibility comment to helper scripts so support can identify stale deployments | todo | | |
| H1.8 | Add release checklist entry to review Apps Script helper compatibility on every helper change | todo | | |

### H2. Full Tunnel And Tunnel-Node Ops Parity

Tasks:

| ID | Task | Status | Owner | Evidence |
|---|---|---|---|---|
| H2.1 | Create a full-mode architecture map: app, local proxy/TUN, CodeFull Apps Script, tunnel-node, target network, optional UDP gateway | todo | | |
| H2.2 | Align `tunnel-node` README, Dockerfile, systemd example, GHCR image naming, and release workflow image naming | todo | | |
| H2.3 | Document `TUNNEL_AUTH_KEY`, health endpoints, allowed methods, body limits, timeout behavior, and error codes | todo | | |
| H2.4 | Add full-mode readiness checks for tunnel-node URL/auth/health before users start the local app | todo | | |
| H2.5 | Add Desktop full-mode setup cards that distinguish Apps Script deployment from tunnel-node deployment | todo | | |
| H2.6 | Add Android full-mode guide text and validation that matches Desktop wording | todo | | |
| H2.7 | Document UDP/udpgw support boundaries and unsupported operation codes clearly | todo | | |
| H2.8 | Add integration smoke tests or scripted curl checks for tunnel-node health and basic relay shape | todo | | |
| H2.9 | Add release artifact/version checklist for tunnel-node binary and container image | todo | | |
| H2.10 | Add support-bundle fields for full-mode backend health and tunnel-node version when available | todo | | |

### H3. Serverless JSON Helper Parity

Tasks:

| ID | Task | Status | Owner | Evidence |
|---|---|---|---|---|
| H3.1 | Audit Vercel JSON relay docs and sample env vars against `vercel` config fields | todo | | |
| H3.2 | Audit Netlify JSON relay docs and sample env vars against `vercel` config fields | todo | | |
| H3.3 | Audit Cloudflare Worker JSON relay docs and sample env vars against `vercel` config fields | todo | | |
| H3.4 | Document `AUTH_KEY`, `relay_path`, base URL, max body, and expected health behavior consistently | todo | | |
| H3.5 | Add UI validation that catches common Serverless JSON mistakes: missing scheme, wrong path, trailing protection page, missing auth | todo | | |
| H3.6 | Add CLI/Doctor checks that distinguish an auth failure from an HTML protection page or non-JSON response | todo | | |
| H3.7 | Add helper syntax tests already present in CI to the roadmap evidence table and extend them to contract tests | todo | | |
| H3.8 | Add docs examples for Vercel, Netlify, and Cloudflare with the same field names used in the apps | todo | | |

### H4. XHTTP Helper Parity

Tasks:

| ID | Task | Status | Owner | Evidence |
|---|---|---|---|---|
| H4.1 | Make every XHTTP guide state that XHTTP is a separate helper path, not the native Serverless JSON mode | todo | | |
| H4.2 | Audit XHTTP generator scripts and docs for stale product names, target domain assumptions, and auth names | todo | | |
| H4.3 | Add compatibility notes that explain which clients can use the XHTTP helper and which cannot | todo | | |
| H4.4 | Add UI/docs links only where they help, without making XHTTP appear required for normal setup | todo | | |
| H4.5 | Add CI stale-name and syntax checks for XHTTP helper folders | todo | | |

### H5. Protocol And Version Skew

Tasks:

| ID | Task | Status | Owner | Evidence |
|---|---|---|---|---|
| H5.1 | Define structured backend error codes for helper scripts where feasible | todo | | |
| H5.2 | Surface backend version or compatibility markers in Doctor/support output where feasible | todo | | |
| H5.3 | Add UI warnings for known old helper versions when detectable | todo | | |
| H5.4 | Add docs for updating helpers without losing auth keys or deployment URLs | todo | | |
| H5.5 | Add release notes template section for backend-helper compatibility changes | todo | | |

Acceptance criteria:

- Backend helper docs, UI setup, and runtime validation point to the same deployable artifacts.
- Full tunnel has an end-to-end deploy/verify story, not scattered pieces.
- Helper version skew becomes diagnosable.

## Workstream I: Security, Trust, Privacy, And Exposure

Goal:

Make trust-affecting behavior clear, safe by default where practical, and
consistent across Desktop, Android, CLI, installers, and docs.

### I1. Certificate And Trust Lifecycle

Tasks:

| ID | Task | Status | Owner | Evidence |
|---|---|---|---|---|
| I1.1 | Document CA generation, installation, trust store targets, removal, and rotation per platform | todo | | |
| I1.2 | Align Desktop CA install UI with CLI `install-ca`, `uninstall-ca`, and `install-ca-firefox` behavior | todo | | |
| I1.3 | Document Android user CA limitations and app trust differences in setup copy and docs | todo | | |
| I1.4 | Add warnings for HTTPS interception that are precise without being alarming | todo | | |
| I1.5 | Add support-bundle redaction for CA paths, cert fingerprints, and any private key paths | todo | | |
| I1.6 | Add Doctor checks for missing CA trust and browser-specific trust gaps | todo | | |

### I2. LAN, SOCKS5, And Exposure Safety

Tasks:

| ID | Task | Status | Owner | Evidence |
|---|---|---|---|---|
| I2.1 | Document when `listen_host=0.0.0.0` is safe, risky, or blocked by policy | todo | | |
| I2.2 | Align `lan_token` and `lan_allowlist` UI copy across Desktop, CLI, docs, and examples | todo | | |
| I2.3 | Add readiness warnings when LAN sharing is enabled without a token or allowlist | todo | | |
| I2.4 | Document SOCKS5 limitations, authentication expectations, and local-network exposure separately from HTTP proxy exposure | todo | | |
| I2.5 | Add support-bundle fields for LAN exposure state while redacting secrets | todo | | |

### I3. Android Permissions And Privacy

Tasks:

| ID | Task | Status | Owner | Evidence |
|---|---|---|---|---|
| I3.1 | Document why the app declares `QUERY_ALL_PACKAGES`, where it is used, and whether narrower package visibility is possible | todo | | |
| I3.2 | Document `FOREGROUND_SERVICE_SPECIAL_USE`, VPN/TUN behavior, notification requirements, and user-visible privacy expectations | todo | | |
| I3.3 | Review Android network security config and user CA trust copy for consistency with actual behavior | todo | | |
| I3.4 | Add in-app or docs copy for split tunneling and app selection privacy implications | todo | | |
| I3.5 | Add Android release checklist items for permission review, Play policy notes if applicable, and privacy copy | todo | | |

### I4. Secrets, Logs, And Support Bundles

Tasks:

| ID | Task | Status | Owner | Evidence |
|---|---|---|---|---|
| I4.1 | Create one redaction utility/policy for auth keys, script IDs when needed, tokens, URLs with credentials, LAN tokens, and CA private paths | todo | | |
| I4.2 | Apply redaction to Desktop logs, Android logs, CLI support bundles, status JSON, screenshots, and copied diagnostics | todo | | |
| I4.3 | Add tests with realistic secret-looking values to prove redaction works | todo | | |
| I4.4 | Add user-facing copy that distinguishes "copy for support" from "export full working config" | todo | | |
| I4.5 | Add docs explaining which fields are secrets and which fields are safe identifiers | todo | | |

### I5. Release Signing, Updates, And Artifact Trust

Tasks:

| ID | Task | Status | Owner | Evidence |
|---|---|---|---|---|
| I5.1 | Document Android keystore/signature continuity and why legacy key alias names may remain | todo | | |
| I5.2 | Review release signing docs for Windows/macOS/Linux/Android and align with installer scripts | todo | | |
| I5.3 | Add update-check trust model: download source, checksum/signature expectations, rollback behavior | todo | | |
| I5.4 | Add release artifact hash generation and verification instructions where missing | todo | | |
| I5.5 | Add release checklist item to verify package names, app labels, protocol schemes, and old `mhrv-rs` compatibility surfaces | todo | | |
| I5.6 | Decide whether committed Android signing material stays in-repo, moves to CI secrets, or is split into public/dev and private/release keys | todo | | |

Acceptance criteria:

- Trust-sensitive behavior is explained where users make the decision.
- Support bundles and diagnostics do not leak secrets by default.
- Android permissions and release signing have explicit project rationale.

## Workstream J: Release, CI, And Quality Automation

Goal:

Make parity enforceable. The roadmap should not rely on memory, screenshots, or
manual search once checks can catch drift.

### J1. Existing Gate Inventory

Already observed:

- Root Rust format/clippy/test gates exist in CI.
- `tunnel-node` format/clippy/test gates exist in CI.
- JSON/XML parse sanity checks exist.
- Android string reference sanity checks exist.
- Helper script syntax checks exist.
- Stale encoded marker checks exist.
- Release workflows package multiple surfaces.

Tasks:

| ID | Task | Status | Owner | Evidence |
|---|---|---|---|---|
| J1.1 | Document current CI jobs, what they protect, and what they do not protect | todo | | |
| J1.2 | Add evidence links or command snippets for each existing check in this roadmap | todo | | |
| J1.3 | Separate required gates from best-effort local checks | todo | | |

### J2. New Contract Gates

Tasks:

| ID | Task | Status | Owner | Evidence |
|---|---|---|---|---|
| J2.1 | Add Rust `Config` vs Desktop `ConfigWire` parity test | todo | | |
| J2.2 | Add Rust `Config` vs Android `ConfigStore` handled-field parity script | todo | | |
| J2.3 | Add Android English/Persian string key parity script | todo | | |
| J2.4 | Add Android hard-coded visible string scan with allowlist | todo | | |
| J2.5 | Add SNI pool parity script | todo | | |
| J2.6 | Add config example parse/validate/round-trip tests | todo | | |
| J2.7 | Add status JSON snapshot/schema tests | todo | | |
| J2.8 | Add backend helper contract tests for JSON response shape where practical | todo | | |
| J2.9 | Add docs stale-name/stale-version/stale-screenshot-reference scans | todo | | |
| J2.10 | Add markdown link check for README and core docs | todo | | |

### J3. Android Quality Gates

Tasks:

| ID | Task | Status | Owner | Evidence |
|---|---|---|---|---|
| J3.1 | Add `ConfigStore` JVM unit tests | todo | | |
| J3.2 | Add Android readiness/localization helper unit tests | todo | | |
| J3.3 | Add Compose screenshot or preview review process for fresh, configured, running, error, and RTL states | todo | | |
| J3.4 | Add `./gradlew assembleDebug` or equivalent Android build gate where CI capacity allows | todo | | |
| J3.5 | Add permission/privacy review checklist for Android releases | todo | | |

### J4. Release Readiness Gates

Tasks:

| ID | Task | Status | Owner | Evidence |
|---|---|---|---|---|
| J4.1 | Add a release checklist that covers Desktop, Android, CLI, helper scripts, tunnel-node, docs, and examples | todo | | |
| J4.2 | Add version-bump checklist for Cargo, Android Gradle, README/docs, release notes, helper compatibility markers, and tunnel-node | todo | | |
| J4.3 | Add artifact checklist for binaries, Android APK/AAB, checksums, installer scripts, GHCR image, and docs archive if used | todo | | |
| J4.4 | Add smoke-test matrix per mode before release | todo | | |
| J4.5 | Add changelog template sections for UI changes, config/schema changes, backend-helper changes, docs changes, security/trust changes, and breaking changes | todo | | |

Acceptance criteria:

- CI catches the highest-risk parity drift.
- Release review has an explicit mode/config/docs/helper checklist.
- Android is no longer untested at the config boundary.

## Workstream K: Repository Professionalization And Maintenance

Goal:

Make the repository easier to understand, contribute to, release from, audit,
and maintain over time.

### K1. Contributor And Maintainer Experience

Tasks:

| ID | Task | Status | Owner | Evidence |
|---|---|---|---|---|
| K1.1 | Add or refresh `CONTRIBUTING.md` with local setup, Rust build, Android build, docs checks, test commands, and release expectations | todo | | |
| K1.2 | Add `docs/maintainer-guide.md` covering project architecture, major surfaces, config contract ownership, helper backends, and release flow | todo | | |
| K1.3 | Add PR template with checklist for config schema, Desktop parity, Android parity, docs, tests, security, screenshots, and release notes | todo | | |
| K1.4 | Add issue templates for bug report, setup problem, Android problem, docs problem, backend-helper problem, and feature request | todo | | |
| K1.5 | Add lightweight ADR process under `docs/adr/` for decisions like signing material, config migration, defaults, and status schema | todo | | |
| K1.6 | Add `CODEOWNERS` or ownership notes if maintainers are available for Rust core, Android, docs, helpers, release, and security | todo | | |

Acceptance criteria:

- A new contributor can find how to build, test, change config, update docs, and
  submit a useful PR.
- Maintainer decisions have a place to live after the chat context is gone.

### K2. Repo Layout And Artifact Hygiene

Tasks:

| ID | Task | Status | Owner | Evidence |
|---|---|---|---|---|
| K2.1 | Inventory root-level files and classify them as source, example, docs, release, generated, local-only, or historical | todo | | |
| K2.2 | Document artifact policy: CI/release workflow is source of truth; `dist/` / `releases/` may remain as backup/archive material only when clearly labeled | todo | | Maintainer decision D-11. |
| K2.3 | Update `.gitignore` / artifact policy docs so build outputs and local secrets are handled consistently | todo | | |
| K2.4 | Create a root `docs/README.md` or docs index that groups user docs, maintainer docs, backend helper docs, Android docs, and security docs | todo | | |
| K2.5 | Move or clearly label historical files so current setup paths are not mixed with legacy artifacts | todo | | |
| K2.6 | Add a repo cleanliness check that lists unexpected large/generated files before release | todo | | |

Acceptance criteria:

- The repository root reads as intentional.
- Current source, docs, examples, release artifacts, and historical assets are
  easy to distinguish.

### K3. Codebase Modularity And Naming Hygiene

Tasks:

| ID | Task | Status | Owner | Evidence |
|---|---|---|---|---|
| K3.1 | Define Rust module boundaries for config, status, Doctor, proxy runtime, Desktop UI state, Desktop widgets, support bundle, and backend helpers | todo | | |
| K3.2 | Define Android package boundaries for config, native bridge, VPN/service, UI screens, UI components, localization, diagnostics, import/export, and tests | todo | | |
| K3.3 | Add naming glossary for modes, backend helpers, config keys, user-facing labels, and legacy compatibility names | todo | | |
| K3.4 | Add lint or static scans for stale product names where they are not compatibility references | todo | | |
| K3.5 | Extract pure helper logic before UI widgets where possible, so tests can cover readiness, config mapping, and redaction without rendering UI | todo | | |
| K3.6 | Define deprecation policy for legacy names such as `google_only`, `vercel_edge`, and `mhrv-rs://` | todo | | |

Acceptance criteria:

- Contributors know where new logic belongs.
- User-facing terms, config terms, and legacy compatibility terms are separated.

### K4. Documentation Operations

Tasks:

| ID | Task | Status | Owner | Evidence |
|---|---|---|---|---|
| K4.1 | Add docs ownership table: canonical entry point, setup docs, Android docs, backend helper docs, troubleshooting, security, maintainer docs, Persian docs | todo | | |
| K4.2 | Add `last reviewed` metadata to core docs and release-touching docs | todo | | |
| K4.3 | Add docs style guide: tone, warning format, mode names, command snippets, Persian terminology, screenshots, and version references | todo | | |
| K4.4 | Add docs update checklist to PR template and release checklist | todo | | |
| K4.5 | Add automated scans for broken links, stale names, stale versions, missing Persian counterparts, and untested example configs | todo | | |
| K4.6 | Add screenshot policy: when to include screenshots, where to store them, how to regenerate, and when to remove instead | todo | | |

Acceptance criteria:

- Docs stay accurate because maintenance is built into the workflow.
- Persian/core-doc parity is a tracked policy, not an occasional cleanup.

### K5. Release And Versioning Discipline

Tasks:

| ID | Task | Status | Owner | Evidence |
|---|---|---|---|---|
| K5.1 | Add `CHANGELOG.md` or formal release notes source with sections for UI, Android, CLI, config/schema, helpers, security, docs, and breaking changes | todo | | |
| K5.2 | Add version bump checklist for Cargo, Android, docs, helper scripts, tunnel-node, release workflows, and update metadata | todo | | |
| K5.3 | Define semantic versioning or project-specific version policy, including config-schema and backend-helper compatibility changes | todo | | |
| K5.4 | Add release smoke-test matrix for every supported mode and platform | todo | | |
| K5.5 | Add artifact naming policy for desktop binaries, Android APK splits/universal APK, tunnel-node images, checksums, and support files | todo | | |
| K5.6 | Add rollback policy for bad config migrations, bad Android releases, bad helper scripts, and bad tunnel-node images | todo | | |

Acceptance criteria:

- Releases are repeatable, reviewable, and explain what changed.
- Version and compatibility changes are visible to users and maintainers.

### K6. Security And Compliance Maintenance

Tasks:

| ID | Task | Status | Owner | Evidence |
|---|---|---|---|---|
| K6.1 | Add `SECURITY.md` with vulnerability reporting, supported versions, secret handling, and trust-model caveats | todo | | |
| K6.2 | Add dependency audit policy for Rust crates, Android Gradle dependencies, Node helper packages, and GitHub Actions | todo | | |
| K6.3 | Add secret scanning policy for config examples, docs, logs, support bundles, CI, Android signing, and helper env files | todo | | |
| K6.4 | Add permission review process for Android manifest changes | todo | | |
| K6.5 | Add certificate/trust-model review to release checklist | todo | | |
| K6.6 | Add license/third-party notice review for bundled binaries, Android dependencies, helper templates, and generated artifacts | todo | | |

Acceptance criteria:

- Security-sensitive changes are reviewed deliberately.
- Secret handling, permissions, and dependency risk are part of maintenance.

### K7. Deprecation, Cleanup, And Garbage Collection

Tasks:

| ID | Task | Status | Owner | Evidence |
|---|---|---|---|---|
| K7.1 | Add a deprecation policy that requires an owner, migration path, compatibility window, tests, docs, and removal criteria | todo | | |
| K7.2 | Add stale-code scan tasks for unused functions, unused config fields, stale resources, stale docs snippets, stale screenshots, and old generated artifacts | todo | | |
| K7.3 | Add a compatibility-surface registry for intentional legacy behavior such as old deep-link schemes, old config keys, and release-signing names | todo | | |
| K7.4 | Require every completed roadmap item to include a cleanup pass: remove dead code, remove stale docs, update examples, update tests, and record evidence | todo | | |
| K7.5 | Add CI or local scripts for stale product names, dead docs links, Android unused resources where practical, config-field drift, and example validation | todo | | |
| K7.6 | Add "garbage collection completed" to PR/release checklist templates | todo | | |

Acceptance criteria:

- Deprecated behavior is either intentionally supported and documented, or
  removed.
- Completed work does not leave stale functions, parameters, docs, examples,
  resources, screenshots, or generated artifacts behind.
- Compatibility code is visible, tested, and time-bounded when possible.

## Workstream L: Experience Excellence And Product Coherence

Goal:

Push the product beyond "organized" into an experience that feels clear,
capable, polished, and trustworthy in every mode and every platform.

### L1. Mode-Specific Home Surfaces

Tasks:

| ID | Task | Status | Owner | Evidence |
|---|---|---|---|---|
| L1.1 | Define a selected-mode dashboard contract for Apps Script, Serverless JSON, Direct, Full, and Android VPN/TUN | todo | | |
| L1.2 | For each mode, list required setup, optional tuning, diagnostics, backend health, safety limits, and docs links | todo | | |
| L1.3 | Add mode-specific "ready / degraded / blocked / running" summary text | todo | | |
| L1.4 | Add mode-specific capability panels that hide unrelated controls but expose all relevant parameters for the selected mode | todo | | |
| L1.5 | Add mode-specific empty states for missing backend, missing CA, no traffic yet, quota unknown, and backend health unknown | todo | | |
| L1.6 | Add mode-specific support bundle fields so support sees the selected mode's actual dependencies | todo | | |

Acceptance criteria:

- Each mode feels complete when selected.
- Users are not forced to scan unrelated mode controls.
- Mode-specific diagnostics and docs are one click away.

### L2. Primary Action And Command Model

Tasks:

| ID | Task | Status | Owner | Evidence |
|---|---|---|---|---|
| L2.1 | Define one command state machine shared conceptually by Desktop and Android: save, save-and-start/connect, start/connect, stop/disconnect, test, doctor, install CA, copy endpoint | todo | | |
| L2.2 | Add dirty-config detection and primary `Save and start` / `Save and connect` action | todo | | |
| L2.3 | Define disabled action reasons with stable readiness IDs and one repair action | todo | | |
| L2.4 | Standardize destructive/danger actions: stop, reset advanced, clear logs, revoke/import config, remove legacy artifacts | todo | | |
| L2.5 | Add command-result toasts/cards that say what happened and what to do next | todo | | |

Acceptance criteria:

- Users can always tell the next action.
- No command requires reading logs to know whether it worked.

### L3. Information Architecture And Density

Tasks:

| ID | Task | Status | Owner | Evidence |
|---|---|---|---|---|
| L3.1 | Split each screen into stable zones: state summary, setup, diagnostics, advanced, logs, help | todo | | |
| L3.2 | Define card/table/list density rules for repeated operational use | todo | | |
| L3.3 | Keep logs secondary by default, with search/filter/copy/export when opened | todo | | |
| L3.4 | Add progressive disclosure for expert fields without hiding them from selected-mode users | todo | | |
| L3.5 | Stress-test long URLs, IDs, Persian text, narrow windows, small phones, and keyboard-open states | todo | | |

Acceptance criteria:

- UI is scan-friendly for daily use.
- Advanced capability exists without overwhelming first-run setup.

### L4. Copy, Vocabulary, And Microcopy

Tasks:

| ID | Task | Status | Owner | Evidence |
|---|---|---|---|---|
| L4.1 | Create a product vocabulary table for modes, backends, config keys, user-facing labels, safety warnings, and legacy compatibility names | todo | | |
| L4.2 | Replace provider-specific user-facing labels with mode labels where appropriate, while documenting compatibility schema names | todo | | |
| L4.3 | Write reusable warning patterns for CA trust, LAN exposure, user CA limits, quota pressure, backend protection pages, and release signing | todo | | |
| L4.4 | Add concise helper text for every setup field and move long explanations to canonical docs | todo | | |
| L4.5 | Add Persian equivalents as part of the same copy change, not later cleanup | todo | | |

Acceptance criteria:

- Same concept uses same words across Desktop, Android, CLI, docs, and release notes.
- Field-level help is short, consistent, and localized.

### L5. Accessibility, RTL, And Interaction Quality

Tasks:

| ID | Task | Status | Owner | Evidence |
|---|---|---|---|---|
| L5.1 | Add keyboard navigation review for Desktop controls and dialogs | todo | | |
| L5.2 | Add screen-reader/content-description review for Android buttons, status cards, and app picker | todo | | |
| L5.3 | Review contrast and semantic color use for warning/error/success/running states | todo | | |
| L5.4 | Add RTL layout checks for every Android setup and diagnostics state | todo | | |
| L5.5 | Add touch-target and one-handed-use review for mobile primary actions and dense advanced fields | todo | | |
| L5.6 | Add copy/clipboard affordances for endpoints, config links, support bundles, and backend URLs | todo | | |

Acceptance criteria:

- The app remains usable on small screens, RTL layouts, keyboard-only Desktop,
  and high-contrast needs.

### L6. Installer, Launcher, And Release Experience

Tasks:

| ID | Task | Status | Owner | Evidence |
|---|---|---|---|---|
| L6.1 | Review Windows installer copy, file layout, docs bundled into installer, and stale screenshot exclusion | todo | | |
| L6.2 | Review macOS builder output, app metadata, icon/branding, and docs links | todo | | |
| L6.3 | Review launchers for clear first-run behavior and consistent config paths | todo | | |
| L6.4 | Review OpenWRT init script docs and CLI UX for router users | todo | | |
| L6.5 | Define release package contents and ensure packaged docs are regenerated from current source only | todo | | |
| L6.6 | Add release smoke checklist for installing from packaged artifacts, not only running from source | todo | | |

Acceptance criteria:

- The install/release experience feels as intentional as the app UI.
- Packaged artifacts cannot quietly ship stale docs or screenshots.

## Granular Execution Breakdown

Use this section when turning roadmap rows into implementation tickets. Every
ticket should be small enough to review and should declare its affected
contracts before code changes begin.

Default breakdown for any item:

1. Inventory: identify exact files, fields, labels, tests, docs, and examples
   affected.
2. Contract: record source of truth, frontend consumers, backward compatibility,
   and migration behavior.
3. Design: decide UI wording, layout state, disabled/error state, and docs
   language.
4. Implementation: make the smallest behavior-preserving change first, then the
   visible improvement.
5. Verification: add automated tests where practical, then run manual checks for
   Desktop, Android, CLI, and docs as applicable.
6. Cleanup: remove stale code paths, unused helpers, deprecated parameters,
   stale docs, stale screenshots, obsolete examples, generated leftovers, and
   dead resources introduced or exposed by the change.
7. Compatibility review: any retained legacy behavior must have a reason, test,
   doc note, and removal condition or explicit long-term support decision.
8. Evidence: record commands, screenshots, fixture names, doc links, or support
   bundle samples in the roadmap.
9. Release gate: add or update the release checklist when the item affects
   setup, safety, config schema, helper scripts, or support.

### A. Desktop Detailed Breakdown

| Area | Granular tasks |
|---|---|
| A0 baseline | Capture current screenshots; list actions; list disabled states; list scroll cutoff points; list duplicated settings; list Desktop-only config fields; record exact viewport sizes. |
| A1 command center | Define state model; extract primary action state; place fixed header; add mode/status badges; add readiness summary; add quota/backend health; wire buttons; test dirty-config behavior; verify short viewport. |
| A2 setup tab | For each mode, list required fields; create minimal Apps Script group editor; create Serverless JSON editor; create direct/fronting editor; create full-mode setup cards; add Test actions; add inline errors; link to exact docs. |
| A3 first-run wizard | Define entry conditions; define skip/resume behavior; create steps per mode; add validation per step; add backend deploy links; add CA trust step; add final test; store completion/migration behavior. |
| A4 form system | Inventory widgets; standardize labels/help/errors; normalize spacing; add reusable password/auth field; add list editors; add empty states; add dirty/save affordance; add reset/revert behavior. |
| A5 monitor | Define status schema consumer; create summary cards; create route/backend table; create Doctor summary; create logs as secondary panel; add support-bundle button; add copy-redacted diagnostics. |
| A6 network/LAN | Map listen host/ports/SOCKS5/LAN token/allowlist; add exposure warnings; add local URL copy buttons; add firewall hints; add CA trust state; test loopback vs LAN cases. |
| A7 advanced | Group expert settings by purpose; move common setup fields out; add field descriptions; mark dangerous settings; preserve search/filter if useful; ensure Advanced is not required for first-run Apps Script. |

### B. Android Detailed Breakdown

| Area | Granular tasks |
|---|---|
| B0 baseline | Capture fresh/configured/running/error screenshots; capture LTR/RTL; list hard-coded strings; list missing resource keys; list Android-only config fields; list permissions shown to users. |
| B1 summary | Define mobile status projection; show mode, route, CA trust, backend health, VPN state, and next action; handle empty/error/loading states; localize all copy. |
| B2 sticky action | Define Connect/Disconnect rules; handle dirty config; show one blocker reason; test keyboard overlap; test small phone; test background/foreground service states. |
| B3 guided setup | Split setup by mode; add Apps Script canonical group serialization; add Serverless JSON validation; add direct/fronting fields; add full-mode help; add import/export warnings; add QR/deep-link behavior. |
| B4 localization | Move all visible copy to resources; fill Persian keys; review RTL order; verify numeric/IP fields; add string parity script; add hard-coded string allowlist. |
| B5 config tests | Add JVM tests for load/save; add legacy import tests; add canonical export tests; add preservation tests; add redaction tests; add deep-link tests. |

### C. Docs Detailed Breakdown

| Area | Granular tasks |
|---|---|
| C0 architecture | Create docs map; decide canonical entry points; assign owners; mark generated vs hand-written docs; define last-reviewed metadata. |
| C1 consolidation | Remove or redirect duplicated setup docs; add backend matrix; fix stale names; audit screenshot references; validate code blocks and example configs. |
| C2 in-app help | Create one source of terms; map help links by mode; write short in-app summaries; avoid long pasted docs in UI; add exact troubleshooting links. |
| C3 Persian docs | Define core docs requiring full parity; define niche docs allowed as summaries; fill Android string gaps; review directionality and terminology. |
| C4 quality | Add docs link check; add stale-version scan; add stale product-name scan; add release docs checklist; keep screenshots current or avoid them. |

### D. Visual Design Detailed Breakdown

| Area | Granular tasks |
|---|---|
| D1 theme | Define semantic colors for running, stopped, warning, error, secure, exposed, quota; apply across Desktop/Android; test contrast; document tokens. |
| D2 density | Set heading/body/control sizes; define card/table spacing; avoid nested cards; verify narrow/short screens; stress-test long URLs and IDs. |
| D3 states | Define empty/loading/success/warning/error states; make actions stable in size; make disabled controls explanatory; verify no overlap. |

### E. Maintainability Detailed Breakdown

| Area | Granular tasks |
|---|---|
| E1 Desktop modules | Extract state models; extract command center; extract setup sections; extract monitor; extract reusable widgets; add tests around pure helpers before UI movement. |
| E2 Android modules | Extract config model helpers; extract summary card; extract setup sections; extract logs/status; extract localization helpers; add previews/tests around pure logic. |
| E3 contracts | Move shared schemas toward generated or tested representations; avoid duplicated literals; add allowlists for intentional platform differences. |

### F. Verification Detailed Breakdown

| Area | Granular tasks |
|---|---|
| F1 screenshots | Define capture commands; define viewports; store review artifacts; compare before/after; include RTL; verify no stale screenshots ship as current. |
| F2 builds | Run Rust UI check; run relevant Rust tests; run Android assemble/tests; run docs checks; record exact commands and failures. |
| F3 manual smoke | For each mode, create config, save, start/connect, test backend, view monitor, run Doctor, export support data, stop cleanly. |

### G. Contract Parity Detailed Breakdown

| Area | Granular tasks |
|---|---|
| G0 inventory | List fields; classify behavior; create fixtures; add owner map; add migration decisions; add CI scripts. |
| G1 serialization | Fix Android `account_groups`; fix Desktop `ConfigWire`; preserve unknown advanced fields; test imports/exports; validate examples. |
| G2 readiness | Define IDs; map Rust validation; update UIs; localize messages; test per mode; update docs. |
| G3 status | Define schema; refactor renderers; update Monitor/Android; add support bundles; add schema tests. |
| G4 taxonomy | Build backend matrix; update UI labels; update docs; add Doctor mismatch hints; test helper docs. |
| G5 defaults | Document defaults; add SNI parity; decide port/IP differences; update examples; add drift tests. |
| G6 sharing | Specify link schema; test legacy links; define redaction; preserve advanced fields; document safe sharing. |

### H. Backend Helpers Detailed Breakdown

| Area | Granular tasks |
|---|---|
| H1 Apps Script | Audit Code.gs variants; align auth names; add version markers; document quota; add release checks. |
| H2 full tunnel | Map architecture; align tunnel-node ops docs; add health checks; add setup cards; add version/support fields. |
| H3 Serverless JSON | Audit Vercel/Netlify/Cloudflare helpers; align env vars; add non-JSON/protection-page diagnostics; update examples. |
| H4 XHTTP | Separate from native mode; audit generator docs; add compatibility notes; add stale-name checks. |
| H5 skew | Add error codes/version markers; add UI warnings; add helper update guide; add changelog section. |

### I. Security Detailed Breakdown

| Area | Granular tasks |
|---|---|
| I1 CA trust | Map platform stores; align install/uninstall; add browser notes; add Doctor checks; redact sensitive paths. |
| I2 LAN/SOCKS5 | Explain exposure; validate tokens/allowlists; document SOCKS5 limitations; add support-bundle summary. |
| I3 Android privacy | Review permissions; document VPN/TUN behavior; review network security config; add split-tunnel privacy copy. |
| I4 redaction | Define redaction policy; implement utility; test realistic secrets; apply across logs/status/support/export. |
| I5 release trust | Document signing; add artifact hashes; explain update trust; verify protocol schemes and legacy compatibility. |

### J. Automation Detailed Breakdown

| Area | Granular tasks |
|---|---|
| J1 existing gates | Inventory CI; document coverage; mark required vs advisory; link evidence. |
| J2 new gates | Add field parity; Android string parity; SNI parity; example validation; status schema; docs checks. |
| J3 Android gates | Add JVM tests; add build gate; add screenshot/previews; add permission review. |
| J4 release gates | Add mode smoke matrix; add version checklist; add artifact checklist; add changelog sections. |

### K. Repository Professionalization Detailed Breakdown

| Area | Granular tasks |
|---|---|
| K1 contributor experience | Add contributing guide; maintainer guide; PR template; issue templates; ADR folder; ownership notes. |
| K2 layout hygiene | Classify root files; decide artifact policy; update ignore rules; create docs index; label historical assets; add cleanliness check. |
| K3 code organization | Define Rust and Android module boundaries; add naming glossary; scan stale names; extract pure helpers before UI movement. |
| K4 docs operations | Add docs ownership; last-reviewed metadata; docs style guide; screenshot policy; docs checks; Persian parity policy. |
| K5 release discipline | Add changelog; version policy; release smoke matrix; artifact naming; rollback policy. |
| K6 security maintenance | Add security policy; dependency audit policy; secret scanning policy; Android permission review; trust-model review; third-party notice review. |
| K7 cleanup discipline | Add deprecation policy; stale-code scans; compatibility registry; per-item garbage-collection checklist; cleanup evidence requirement. |
| L1 mode surfaces | Add selected-mode dashboards; mode capability panels; mode-specific diagnostics; mode-specific support fields. |
| L2 command model | Add shared command states; dirty `Save and start`; disabled reasons; command result cards. |
| L3 information architecture | Define stable screen zones; density rules; logs as secondary; long-content stress tests. |
| L4 copy system | Add vocabulary table; warning patterns; field help; Persian copy parity in the same change. |
| L5 accessibility | Add keyboard, screen-reader, contrast, RTL, touch-target, and clipboard-affordance reviews. |
| L6 install/release UX | Review installer/launcher/package UX and prevent packaged stale docs/screenshots. |

## Implementation Phases

### Phase 0: Audit, Baseline, And Governance

Goal:

Create the baseline, stop drift, and make the project contract visible.

Includes:

- Deep inventory of Desktop, Android, CLI, docs, helper scripts, tunnel-node,
  release workflows, installers, and examples.
- Screenshot baseline.
- Documentation map.
- Localization gap list.
- Action inventory.
- Config field ownership map.
- Mode/backend taxonomy map.
- Status/Doctor/support schema decision.
- Security/trust surface inventory.
- Roadmap bookkeeping.

Exit criteria:

- This roadmap is adopted.
- Current state is documented with file-level evidence.
- P0 parity gaps are tracked as first-wave work.
- No one has to rediscover the same UI/docs/config/backend gaps.

### Phase 1: Contract Parity Quick Fixes

Goal:

Fix the highest-risk backend/frontend mismatches before larger UI work builds on
them.

Candidate tasks:

- Android exports canonical `account_groups` for Apps Script and full mode.
- Android imports canonical `account_groups` and migrates/preserves legacy
  `script_ids` / `auth_key`.
- Desktop `ConfigWire` handles `domain_overrides`.
- `enable_batching` omission is decided and documented.
- Add field-parity checks for Rust `Config`, Desktop `ConfigWire`, and Android
  `ConfigStore` allowlists.
- Add Android `ConfigStore` tests for Apps Script, Serverless JSON, direct, and
  full fixtures.
- Add SNI pool parity check.
- Fill missing Persian string keys and move hard-coded visible Android strings
  to resources.
- Replace stale screenshot assumptions with verified screenshot/reference audit.

Exit criteria:

- Android and Desktop can save configs that Rust validates for every supported
  mode.
- Contract drift is caught by tests or explicit allowlists.
- Localization can be checked mechanically.

### Phase 2: Quick UX And Documentation Wins

Goal:

Improve perceived quality without risky architecture work.

Candidate tasks:

- Fix misleading Save/Start/Connect copy.
- Remove duplicate primary actions where easy.
- Add clearer disabled Start/Connect explanations.
- Add docs map link from README or docs index.
- Add JSON vs XHTTP distinction banner to Setup/Help/docs.
- Rename user-facing Serverless JSON copy away from provider-specific labels
  where appropriate.
- Add platform defaults table for ports, Google IP, proxy behavior, and Android
  VPN/TUN behavior.

Exit criteria:

- First-run confusion is reduced.
- Docs are less likely to send users to the wrong backend helper.
- Platform differences look intentional and documented.

### Phase 3: Desktop Command Center And Setup Flow

Goal:

Make desktop feel like a guided control app.

Candidate tasks:

- Fixed command center.
- Readiness model.
- Complete Setup tab.
- State-aware wizard.
- Network safety card.

Exit criteria:

- User can set up Apps Script without visiting Advanced.
- Critical actions do not scroll away.
- Start disabled state is explained.
- Serverless JSON, direct fronting, and full tunnel each have a clear setup path.

### Phase 4: Android Guided Mobile Flow

Goal:

Make Android state-first and mobile-native.

Candidate tasks:

- Status summary card.
- Sticky bottom primary action.
- Guided section completion.
- Help sheet or shorter help sections.
- RTL review.

Exit criteria:

- Fresh Android setup is shorter and clearer.
- Connect/Disconnect remains reachable.
- English and Persian layouts are both usable.
- Android import/export behavior is tested against canonical fixtures.

### Phase 5: Monitor, Diagnostics, Help, And Support Integration

Goal:

Make troubleshooting feel guided instead of log-driven.

Candidate tasks:

- Monitor health dashboard.
- Doctor result summaries.
- Recommended action based on failure state.
- In-app help links to canonical docs.
- Symptom-based docs rewrite.

Exit criteria:

- Users can understand failures without reading raw logs first.
- Logs remain available for support.
- Docs and app use the same repair language.
- Status/support data uses a shared schema across CLI, Desktop, Android, and
  local status API.

### Phase 6: Backend Helper And Full-Tunnel Completion

Goal:

Make Apps Script, Serverless JSON, XHTTP, and full tunnel helper paths
deployable and diagnosable without cross-reading unrelated docs.

Candidate tasks:

- Backend helper matrix.
- Apps Script helper audit.
- Serverless JSON helper audit.
- XHTTP separation audit.
- Full tunnel architecture map.
- Tunnel-node health, Docker, GHCR, systemd, and release docs alignment.
- Full-mode readiness checks and support-bundle fields.

Exit criteria:

- A user can deploy the correct backend for the selected mode.
- Full tunnel has an end-to-end deploy/verify/support path.
- Helper version skew is visible or documented.

### Phase 7: Architecture Refactor And Automation

Goal:

Make future UI work safer and faster.

Candidate tasks:

- Split desktop UI modules.
- Extract Android components.
- Add readiness helper tests.
- Add docs/string parity tooling.
- Add config/status/schema parity gates.
- Add Android build/unit-test gates where practical.
- Add release checklist for artifacts, helpers, docs, security, and mode smoke
  tests.

Exit criteria:

- UI files are smaller and feature ownership is clearer.
- New UI changes do not require editing one giant file.
- Release quality depends less on memory.

### Phase 8: Repository Professionalization

Goal:

Make the repository easier to maintain and easier for serious contributors to
trust.

Candidate tasks:

- `CONTRIBUTING.md`.
- Maintainer guide.
- PR template and issue templates.
- ADR folder and first ADRs for signing policy, config migration, defaults, and
  status schema.
- Changelog and version policy.
- Docs ownership table and style guide.
- Artifact policy for `dist/`, `releases/`, generated files, and signing
  material.
- `SECURITY.md`, dependency audit policy, and secret scanning policy.

Exit criteria:

- Project process is documented enough that future work does not depend on one
  person's memory.
- Repo layout, release process, docs ownership, and security posture look
  intentional.

### Phase 9: Experience Excellence Pass

Goal:

Polish the product into a coherent, mode-rich, accessibility-reviewed
experience after the contracts and repo foundation are stable.

Candidate tasks:

- Selected-mode dashboards.
- Mode-specific capability panels.
- Shared command model and command-result cards.
- Copy/vocabulary system.
- Accessibility and RTL QA.
- Installer/launcher/release package UX.
- Packaged artifact stale-doc/screenshot guard.

Exit criteria:

- Every mode feels rich and complete without showing unrelated controls.
- Desktop, Android, docs, installers, and release packages tell the same story.
- Accessibility, localization, and packaged-output quality are checked before
  release.

## Backlog

Use this table for newly discovered work.

| ID | Area | Item | Priority | Status | Owner | Notes |
|---|---|---|---|---|---|---|
| BL.1 | Docs | Audit screenshot references and remove, replace, or mark stale images historical | high | todo | | No current `docs/*.png` was found; still scan docs/release surfaces for stale references. |
| BL.2 | Android | Add missing Persian string keys | high | todo | | English has 150, Persian has 138. |
| BL.3 | Docs | Create canonical docs map | high | todo | | Prevent dispersed/overlapped docs from drifting. |
| BL.4 | Desktop | Make top command center fixed | high | todo | | Current full UI scrolls. |
| BL.5 | Desktop | Move minimal account group setup into Setup tab | high | todo | | Avoid sending first-run users to Advanced. |
| BL.6 | Android | Move hard-coded visible strings to resources | high | todo | | Needed for localization. |
| BL.7 | Docs | Add JSON vs XHTTP distinction banner | medium | todo | | Repeated source of confusion. |
| BL.8 | Desktop | Remove duplicate Start/Test/Doctor action cluster | medium | todo | | Reduce action noise. |
| BL.9 | Docs | Expand Persian core docs parity | medium | todo | | Focus on core docs first. |
| BL.10 | Desktop | Refactor `src/bin/ui.rs` into modules | medium | todo | | Do after behavior is better understood. |
| BL.11 | Android | Serialize canonical `account_groups` from `ConfigStore.toJson()` | critical | review | | Implemented for Apps Script and full mode; pending CI/approved Android test run. |
| BL.12 | Android | Read and preserve canonical `account_groups` in `ConfigStore.loadFromJson()` | critical | review | | Implemented with simple-first projection plus `preservedAccountGroupsJson`; pending CI/approved Android test run. |
| BL.13 | Desktop | Add `domain_overrides` to `ConfigWire`; decide `enable_batching` handling | critical | review | | Implemented Desktop wire serialization and form preservation for `domain_overrides`, top-level `enable_batching`, and `vercel.enable_batching`; focused UI binary test passes. |
| BL.14 | Tests | Add Rust `Config` vs Desktop `ConfigWire` parity test | critical | review | | Added focused `ConfigWire` regression test and broad all-current-field serialization guard; UI binary tests pass. |
| BL.15 | Tests | Add Android `ConfigStore` load/save/import/export tests | critical | review | | Added JVM tests; not run locally because maintainer disallowed local Gradle download/install. |
| BL.16 | Tests | Add SNI default parity check between Rust and Android | high | todo | | Android list manually mirrors Rust. |
| BL.17 | Backend/UI | Create shared status/stats snapshot renderer | high | todo | | `/status` and JNI currently build JSON separately. |
| BL.18 | Diagnostics | Map Doctor results to structured Desktop/Android summaries | high | todo | | Reduces log-driven troubleshooting. |
| BL.19 | Support | Add redacted support bundle parity for CLI/Desktop/Android | high | todo | | Shared support payload improves debugging and privacy. |
| BL.20 | Docs | Add platform defaults table for ports, IPs, proxy modes, CA trust, and Android VPN/TUN | high | todo | | Some defaults differ intentionally or accidentally. |
| BL.21 | Full tunnel | Add end-to-end tunnel-node deploy/verify/support path | high | todo | | Full mode spans app, Apps Script, and tunnel-node. |
| BL.22 | Android | Document permission/privacy rationale for `QUERY_ALL_PACKAGES`, VPN/TUN, and foreground service | high | todo | | Manifest exposes trust/privacy questions users and reviewers may ask. |
| BL.23 | Release | Add release QA checklist covering config parity, helpers, docs, screenshots, Android permissions, and artifacts | high | todo | | Release workflows need product-level gates. |
| BL.24 | Tests | Validate example configs and docs snippets against the Rust loader | medium | todo | | Examples should fail CI when schema changes break them. |
| BL.25 | Docs/CI | Add stale name, stale version, stale screenshot-reference, and Markdown link checks | medium | todo | | Prevents docs decay. |
| BL.26 | Backend helpers | Add backend matrix for Apps Script, Serverless JSON, XHTTP, direct, and full tunnel | high | todo | | Prevents mode/helper mix-ups. |
| BL.27 | Security | Define redaction utility and apply it to logs, status, support bundles, and shared configs | high | todo | | Auth keys and LAN tokens must not leak by default. |
| BL.28 | Android | Add deep-link/QR import-export schema tests for `mhrvf://` and legacy `mhrv-rs://` | medium | todo | | Config sharing is a contract. |
| BL.29 | Docs | Explain legacy schema names such as `vercel` / `vercel_edge` as compatibility names | medium | todo | | Avoids provider-label confusion. |
| BL.30 | CI | Add status JSON schema or snapshot tests | medium | todo | | Keeps Monitor, Android, status API, and support bundles aligned. |
| BL.31 | Release/Security | Document committed Android release signing material policy | critical | todo | | Decision made: keep committed signing material for install-over compatibility; document risk, rotation path, and official-release expectations. |
| BL.32 | Repo | Add `CONTRIBUTING.md` and maintainer guide | high | todo | | New contributors need build/test/docs/release guidance. |
| BL.33 | Repo | Add PR template, issue templates, and ADR folder | high | todo | | Captures decisions and prevents parity/security misses. |
| BL.34 | Repo | Document artifact policy for `dist/`, `releases/`, generated binaries, Android APKs, and signing files | high | todo | | CI/release workflow is source of truth; local repo artifacts are backup/archive material only. |
| BL.35 | Docs | Add docs ownership table, style guide, last-reviewed metadata, and screenshot policy | high | todo | | Makes docs maintainable after the first cleanup. |
| BL.36 | Release | Add `CHANGELOG.md`, version policy, artifact naming policy, and rollback policy | high | todo | | Makes releases professional and repeatable. |
| BL.37 | Security | Add `SECURITY.md`, dependency audit policy, secret scanning policy, and Android permission review process | high | todo | | Makes security maintenance visible. |
| BL.38 | Codebase | Define Rust and Android module boundary docs before major refactors | medium | todo | | Prevents cleanup from becoming arbitrary file shuffling. |
| BL.39 | Repo | Add deprecation, stale-code cleanup, and per-item garbage-collection policy/checklist | critical | todo | | Every item should remove stale/deprecated leftovers or document tested compatibility. |
| BL.40 | Release/Docs | Add packaged-artifact stale screenshot/docs guard | critical | todo | | `dist/.../docs/ui-screenshot.png` exists while source docs have no PNG. |
| BL.41 | Docs | Add full Persian parity matrix for every Markdown doc | critical | todo | | Many English docs have no `.fa.md` counterpart or much shorter equivalents. |
| BL.42 | Dependencies | Track temporary `tun2proxy` patch with owner, upstream condition, and removal criteria | high | todo | | `Cargo.toml` contains a temporary git patch. |
| BL.43 | Repo | Add artifact inventory and cleanup policy for `target/`, `tunnel-node/target`, `dist/`, and `releases/` | high | todo | | Workspace generated outputs measured roughly 9.8 GB. |
| BL.44 | Release | Define one changelog/release-notes source of truth and projections to GitHub/docs/Telegram | high | todo | | Release notes, release-drafter, workflows, and Telegram notifier can drift. |
| BL.45 | UI/UX | Add selected-mode dashboards and mode-specific capability panels | high | todo | | Each mode should be rich in its own parameters/capabilities. |
| BL.46 | UI/UX | Add shared command model with dirty `Save and start` / `Save and connect` state | high | todo | | Makes the primary action obvious and prevents unsaved-start confusion. |
| BL.47 | UI/UX | Add accessibility, RTL, contrast, touch-target, and keyboard QA matrix | medium | todo | | Elevates polish beyond layout cleanup. |
| BL.48 | Installer | Review installer/launcher/OpenWRT/macOS package UX and bundled docs freshness | medium | todo | | Install experience is part of product UX. |

## Decision Log

Record important decisions here.

| Date | Decision | Reason | Alternatives considered | Owner |
|---|---|---|---|---|
| 2026-04-30 | Keep this roadmap in the project root as `elevation_audit_roadmap_source.md` | User requested a root Markdown source for UI/UX/docs elevation and bookkeeping | `docs/ui-ux-roadmap.md` | |
| 2026-04-30 | Treat backend/frontend contract parity as first-wave elevation work, not follow-up polish | Config, readiness, status, and backend-helper drift can create real breakage before visual polish matters | Keep parity items as ad hoc backlog notes | |
| 2026-04-30 | Android should remain on par with Desktop for canonical `account_groups`, implemented as simple-first mobile editing with advanced preservation | Maintainer wants parity with Desktop while allowing implementation judgment; mobile UX should not become a compressed Desktop form | Full Desktop-style multi-group editor as the first mobile implementation | |
| 2026-04-30 | Keep committed Android signing material and document the policy | Maintainer chose install-over compatibility and explicit documentation | Move official signing entirely to CI/private secrets | |
| 2026-04-30 | Persian docs should target full parity | Maintainer chose first-class Persian documentation | Core-only full parity with curated niche summaries | |
| 2026-04-30 | CI/release workflow is source of truth for release artifacts; repo artifacts may remain as backup/archive | Maintainer wants backups without making local artifacts authoritative | Remove `dist/` / `releases/` from source tree entirely | |
| 2026-04-30 | Add lightweight professional governance | Maintainer approved PR templates, issue templates, ADRs, changelog, release checklist, and contributing guide | Keep process informal | |
| 2026-04-30 | Keep platform defaults stable for now; document current Android/Desktop differences and make alignment test-governed | Maintainer delegated default decision; safer path is no churn before smoke tests | Immediately align ports/IPs without validation | |
| 2026-04-30 | Plan Desktop QR/deep-link config support | Maintainer wants Desktop sharing planned beyond file import/export | Desktop file-only sharing | |

## Progress Log

Add dated entries as work proceeds.

| Date | Area | Change | Status | Evidence |
|---|---|---|---|---|
| 2026-04-30 | Audit | Initial UI/UX/docs audit roadmap created | done | `elevation_audit_roadmap_source.md` |
| 2026-04-30 | Deep audit | Expanded roadmap after project-wide scan of Rust config/UI, Android config/UI/JNI, docs, helpers, tunnel-node, CI, release, security, and parity gaps | done | `Project-Aware Deep Audit Findings`, Workstreams G-J |
| 2026-04-30 | Decisions | Recorded maintainer decisions for Android account-group parity, committed signing material, Persian full parity, release artifact policy, governance, defaults, and Desktop QR/deep-link planning | done | `Decisions Needed From Maintainer`, `Decision Log`, `Open Questions` |
| 2026-04-30 | Decisions | Closed remaining maintainer decisions and added no-stale-leftovers cleanup policy | done | Decision Log D-12 through D-23, K7, Package 7, Open Questions |
| 2026-04-30 | Second-pass audit | Re-scanned artifacts, workflows, launchers, helper tools, Android resources, docs parity, release packages, and stale/deprecated markers; added P0.12-P0.16 and Workstream L | done | Second-Pass End-To-End Audit Addendum, Workstream L, BL.40-BL.48 |
| 2026-04-30 | Batch 1 / Android config contract | Implemented canonical Android `account_groups` write/read/projection/preservation path and added focused JVM tests; local Gradle execution is intentionally pending because maintainer disallowed local Gradle download/install | review | `android/app/src/main/java/com/farnam/mhrvf/ConfigStore.kt`, `android/app/src/test/java/com/farnam/mhrvf/ConfigStoreTest.kt`, `android/app/build.gradle.kts`; static process check showed no Gradle/Java/Kotlin process left running |
| 2026-04-30 | Batch 2 / Rust and Desktop config parity | Added Rust legacy Android config migration; preserved Desktop `domain_overrides`, top-level `enable_batching`, and `vercel.enable_batching`; added focused Rust/UI tests | review | `src/config.rs`, `src/bin/ui.rs`; `cargo test migrates_legacy_android_root_script_ids`; `cargo test canonical_account_groups_take_precedence_over_legacy_android_fields`; `cargo test --features ui --bin mhrv-f-ui config_wire_preserves_domain_overrides_and_batching_flags` |

## Batch Implementation Log

### BATCH-1 - Android canonical config contract

Status: review pending CI or an approved preinstalled Gradle environment.

Progress estimate: code and tests are about 85% complete for this batch; verification is the remaining 15%. Overall elevation program progress is about 5% because the roadmap intentionally covers UI/UX, backend/frontend parity, docs, CI, release, governance, and cleanup.

Scope:

- Fix Android Apps Script/full-mode config output to use canonical `account_groups`.
- Keep legacy top-level Android `script_ids` / `auth_key` import compatibility.
- Implement simple-first mobile projection for imported account groups.
- Preserve advanced Desktop-created or multi-group JSON that Android does not fully edit yet.
- Add focused JVM tests for canonical export, canonical import, full-mode export, legacy import, multi-group preservation, and share/export JSON.
- Do a cleanup pass for stale root-level credential writes from Android export.

Changed files:

- `android/app/src/main/java/com/farnam/mhrvf/ConfigStore.kt`
- `android/app/src/test/java/com/farnam/mhrvf/ConfigStoreTest.kt`
- `android/app/build.gradle.kts`
- `elevation_audit_roadmap_source.md`

Implemented details:

- Added canonical account-group helpers for script ID extraction, URL reconstruction, JSON list parsing, legacy projection, and preservation-aware merge.
- Added `preservedAccountGroupsJson` to Android config state so imported advanced groups are not destroyed by normal mobile editing.
- Updated `toJson()` so Apps Script and full-mode exports write Apps Script credentials under `account_groups` instead of legacy root-level `script_ids` / `auth_key`.
- Updated `loadFromJson()` so Android reads canonical `account_groups` first and falls back to legacy root-level fields.
- Added unit-test coverage for canonical Android config behavior and preservation of unsupported advanced fields.
- Added test dependencies needed by those JVM tests.

Verification:

- Static verification completed: changed files are present, expected roadmap statuses are updated, and no Gradle/Java/Kotlin process remains running.
- Local Gradle verification intentionally not run because maintainer instructed: do not download/install Gradle locally.
- Required next verification: run `:app:testDebugUnitTest --tests com.farnam.mhrvf.ConfigStoreTest` in CI or an approved environment with Gradle already available.

Garbage collection:

- Removed Android's stale canonical-output behavior that wrote Apps Script credentials at the JSON root.
- Kept legacy root-level credential reading only as explicit compatibility behavior.
- No local generated Gradle artifacts were intentionally introduced as source-of-truth material.

Remaining after Batch 1:

- Add the narrow Rust legacy loader migration for old Android top-level `script_ids` / `auth_key` configs. Completed in Batch 2.
- Add Desktop `ConfigWire` parity fixes for `domain_overrides` and `enable_batching`. Completed in Batch 2.
- Add CI-owned Android test execution so local Gradle is not required.
- Add Android import warnings for preserved-but-not-editable advanced account group fields.

### BATCH-2 - Rust and Desktop config parity

Status: review; Rust verification passed locally. Android Gradle verification remains CI/approved-environment only.

Progress estimate: Batch 2 is about 90% complete. The remaining 10% is expanding the focused Desktop wire regression into a generated full-field parity matrix.

Scope:

- Add Rust compatibility for old Android exports that stored Apps Script credentials at the JSON root as `script_ids` / `auth_key`.
- Keep canonical `account_groups` authoritative when both canonical and legacy fields are present.
- Stop Desktop save from dropping `domain_overrides`.
- Stop Desktop form/save/profile load from dropping top-level `enable_batching` and `vercel.enable_batching`.
- Add focused tests around the compatibility and wire-parity fixes.
- Do a cleanup pass to keep legacy behavior as import-only compatibility rather than new output.

Changed files:

- `src/config.rs`
- `src/bin/ui.rs`
- `elevation_audit_roadmap_source.md`

Implemented details:

- Added `migrate_legacy_android_account_groups()` before Rust config deserialization, used by both `Config::from_json_str()` and `Config::load()`.
- Migration creates one enabled `legacy-android-primary` account group only when `account_groups` is absent/null and legacy root fields exist.
- Added tests proving old Android root-level credentials load and canonical `account_groups` wins over stale legacy fields.
- Added `domain_overrides` to Desktop `ConfigWire` serialization.
- Added Desktop form fields for top-level `enable_batching` and `vercel.enable_batching` so loaded/profile configs are preserved through save.
- Added a Serverless JSON advanced checkbox for the JSON batch envelope and an Advanced compatibility checkbox for the legacy top-level batching flag.
- Added a UI binary regression test proving `ConfigWire` preserves `domain_overrides`, top-level `enable_batching`, and `vercel.enable_batching`.

Verification:

- `cargo fmt`
- `cargo test migrates_legacy_android_root_script_ids`
- `cargo test canonical_account_groups_take_precedence_over_legacy_android_fields`
- `cargo test --features ui --bin mhrv-f-ui config_wire_preserves_domain_overrides_and_batching_flags`

Garbage collection:

- Kept legacy root `script_ids` / `auth_key` as Rust import compatibility only; canonical output remains `account_groups`.
- Avoided adding a second Desktop schema path; the existing `ConfigWire`/`FormState` path now carries the missing fields.
- No local Gradle download/install was performed.

Remaining after Batch 2:

- Add a generated/current-field Desktop `Config` vs `ConfigWire` parity test so future fields cannot silently drop.
- Run Android `ConfigStoreTest` only in CI or an approved environment with Gradle already available.
- Add Android warnings for advanced preserved-but-not-editable account groups.
- Start the next UI/UX batch: mode-specific dashboards and primary dirty-state actions.

## Open Questions

| ID | Question | Why it matters | Status | Decision |
|---|---|---|---|---|
| Q1 | Should Desktop tab labels remain conservative or shift to user-centered labels like `Connect`? | Label changes affect docs, screenshots, and user memory. | decided | Shift toward user-centered labels such as `Connect`, while keeping technical labels in secondary/help text. |
| Q2 | Should Persian docs aim for full parity or curated parity? | Full parity costs more; curated parity may serve users better if done deliberately. | decided | Full parity. |
| Q3 | Should stale screenshots be removed entirely or replaced per release? | Screenshots help users but become stale quickly. | decided | Avoid screenshots in core docs unless regenerated per release; remove stale screenshots or mark historical. |
| Q4 | Should Doctor output become structured enough for UI summary cards? | Would improve troubleshooting UX. | decided | Yes, Doctor should provide structured IDs/severity for UI summary cards. |
| Q5 | Should `Save config` and `Start` merge into `Save and start` when fields are dirty? | Reduces mistakes but changes current mental model. | decided | Add `Save and start` / `Save and connect` as the primary action when fields are dirty; keep explicit save available. |
| Q6 | Should Android expose a full multi-account group editor or a simple first-group editor that preserves advanced groups invisibly? | A full editor improves parity but adds mobile complexity; preservation is safer than destructive simplification. | decided | Simple first-group editor for normal mobile setup, full canonical config read/write, and safe preservation of imported complex multi-group configs. Full advanced mobile editing is optional later. |
| Q7 | Should legacy top-level `script_ids` / `auth_key` be migrated in Rust config loading, Android import only, or both? | The boundary affects backward compatibility and how many surfaces must understand legacy Android exports. | decided | Both: Android import support plus narrow Rust legacy compatibility, while canonical output remains `account_groups`. |
| Q8 | Is `enable_batching` still an active config field, a deprecated field, or a hidden default? | Desktop `ConfigWire` omits it; tests need either inclusion or an intentional omission. | decided | Active advanced/experimental field; preserve, document, and keep out of first-run UI. |
| Q9 | Are Android default Google IP and ports intentionally different from Rust/Desktop defaults? | Intentional platform defaults should be documented; accidental drift should be fixed. | decided | Keep current platform defaults stable for now; document differences and make future alignment test-governed. |
| Q10 | Should `status_api` become the canonical status renderer for JNI, Desktop Monitor, and support bundles? | A single renderer reduces drift but may require a projection layer for mobile UI needs. | decided | Use a shared canonical Rust status snapshot with UI-specific projections. |
| Q11 | How much tunnel-node operational state should be visible in Desktop and Android full-mode setup? | More visibility helps full-mode users but can overwhelm non-full-mode users. | decided | Each mode should be rich in its own parameters/capabilities; full mode should expose tunnel-node health/version/auth/limits inside full-mode surfaces. |
| Q12 | Should Android `QUERY_ALL_PACKAGES` remain, be narrowed, or be defended with explicit docs and release rationale? | Permission scope affects privacy expectations, store review, and user trust. | decided | Investigate narrowing first; if not feasible, keep and document the privacy rationale clearly. |
| Q13 | Should Desktop support QR/deep-link config import/export, or should config sharing remain file-based on desktop? | Cross-device setup could improve onboarding but adds security/redaction complexity. | decided | Plan Desktop QR/deep-link support in addition to file import/export. |
| Q14 | Should backend helper scripts expose explicit version markers in responses? | Version markers improve support and skew detection but require helper updates and docs. | decided | Add lightweight helper version/compatibility markers where feasible. |
| Q15 | Which CI checks are release-blocking versus advisory while the project remains broad and multi-platform? | Too many blocking checks can slow releases; too few allow parity drift. | decided | Use core blocking release gates plus advisory extended checks. |
| Q16 | Should the repo keep release artifacts in `dist/` / `releases/`, or should releases be built only by CI and attached outside the source tree? | Artifact policy affects repo size, cleanliness, reproducibility, and trust. | decided | CI/release workflow is source of truth; repo artifacts may remain as labeled backup/archive material. |
| Q17 | Should the project add lightweight ADRs for major decisions? | ADRs preserve rationale for signing, config migration, defaults, status schema, and docs parity decisions. | decided | Add lightweight ADRs. |
| Q18 | Should changelog/release notes be hand-written, generated from PR labels, or a hybrid? | Release quality depends on accurate user-facing summaries. | decided | Add changelog/release process as part of lightweight governance; use hand-written source initially, automation later if useful. |
| Q19 | Should official Android release signing move to private CI secrets, or remain committed for install-over compatibility? | This is the clearest security/governance decision in the current repo. | decided | Keep committed signing material and document policy/risk/rotation path. |
| Q20 | Should the repo adopt CODEOWNERS/area ownership, or stay single-maintainer/lightweight? | Ownership improves review quality but may be too formal for a small project. | decided | Add lightweight governance first; CODEOWNERS/area ownership can be added when maintainers are available. |

## Acceptance Definition For The Whole Elevation Program

The program is successful when:

- A new desktop user can complete a basic Apps Script setup from the Setup flow.
- A new Android user can complete a basic VPN/TUN setup without reading the
  entire docs set first.
- Desktop, Android, and CLI can create or load valid configs for Apps Script,
  Serverless JSON, direct fronting, and full tunnel.
- A backend config field cannot be added without an explicit Desktop, Android,
  docs, examples, migration, and test decision.
- The app always explains why Connect/Start is disabled.
- Critical actions remain reachable.
- Monitor explains health before requiring logs.
- Desktop, Android, CLI, `/status`, and support bundles agree on status,
  readiness, diagnostic severity, and key metric names.
- LAN sharing and CA trust risks are clear and contextual.
- Support bundles, logs, copied diagnostics, and shared examples redact secrets
  by default.
- Docs have a canonical map and reduced overlap.
- Backend helper docs clearly separate Apps Script, Serverless JSON, XHTTP,
  direct fronting, and full tunnel.
- Full tunnel has an end-to-end deploy, configure, verify, troubleshoot, and
  release path covering CodeFull, tunnel-node, and app config.
- Persian core workflows are usable and not visibly incomplete.
- Desktop and Android share vocabulary, colors, and interaction priorities.
- CI or documented release checks catch Config/ConfigWire/Android drift,
  Android string drift, SNI drift, stale docs names, stale screenshot
  references, example config breakage, and status schema drift.
- UI code is modular enough that future polish does not require editing a
  monolithic file for every change.
- The repo has contributor, maintainer, security, changelog, versioning,
  artifact, and release-process documentation.
- Major decisions are captured in ADRs or an equivalent decision log.
- Generated files, release artifacts, local outputs, examples, and source files
  are easy to distinguish.
- Stale and deprecated code, functions, parameters, docs, screenshots,
  examples, generated artifacts, and resources are removed as part of each
  completed item unless they are explicitly supported compatibility surfaces
  with tests and documentation.
- Every completed item includes a final garbage-collection pass and records
  cleanup evidence.
