# Changelog - Strategic Batch 3 donor follow-up absorption (2026-05-03)

Maintainer-facing record for the v1.9.6/v1.9.7 follow-up review supplied by
the user.

## Summary

| Field | Detail |
|-------|--------|
| What changed | Added `src/lan_utils.rs`, upgraded the desktop LAN sharing UI to use a friendly checkbox with detected LAN endpoints and custom-bind preservation, added Rust-side `goog.script.init` relay-envelope unwrapping, and linked two Persian setup guide resources from the README. |
| Why | The pasted upstream changes contained two runtime/UX improvements still missing from this repo: legacy Apps Script HtmlService response tolerance and friendlier LAN sharing. Apps Script helper hardening and Telegram release-note plumbing were already present, so they were verified instead of duplicated. |
| Files changed | `src/lan_utils.rs`, `src/lib.rs`, `src/domain_fronter.rs`, `src/bin/ui.rs`, `README.md`, `docs/sharing-and-per-app-routing.md`, `docs/changelog/batch-3-trust-snapshot-2026-05-03.md`, `elevation_audit_roadmap_source.md`, this changelog. |
| Desktop impact | Network sharing now has a single safe checkbox, detected LAN proxy endpoints, a custom-bind badge, and firewall/security hover text. |
| Android impact | No Kotlin change. Android already has LAN token/allowlist editing and config preservation from earlier batches; this batch keeps mobile parity documented rather than adding duplicate controls. |
| Backend impact | Apps Script helpers were checked and already had ContentService `doGet`, IP-leak header stripping, and safe `fetchAll` fallback. Rust can now parse legacy `goog.script.init` wrapped relay JSON defensively. |
| Docs impact | Sharing docs describe the checkbox, route-table LAN IP detection, custom bind preservation, and external Persian guide links. |

## Absorption Decisions

- Ported now:
  - Rust `goog.script.init("...userHtml...")` unwrap for legacy Apps Script
    deployments and redirect edge cases.
  - LAN utility helpers and unit tests.
  - Friendly desktop LAN sharing checkbox and detected endpoint display.
  - README external Persian guide links in compact RTL form.
- Verified already present:
  - Apps Script `ContentService` JSON/decoy behavior.
  - IP-leak header stripping in Apps Script helpers.
  - Safe replay fallback for `UrlFetchApp.fetchAll()` failures.
  - Optional/gated Telegram release notification with changelog input.
- Not ported:
  - Full README rewrite from the donor branch, because this repo has a broader
    multi-mode product surface and existing docs index.
  - Separate Telegram files-channel publishing script, because this repo keeps
    CI release workflow as the source of truth and has a gated Android APK
    Telegram mirror.
  - YouTube thumbnail embed, because the final donor commit itself collapsed it
    into links and this repo avoids stale visual assets in core docs.

## Cleanup

- No donor binaries, Gradle wrappers, or runtime downloads were introduced.
- No duplicate Apps Script helper was added.
- No backward-compatibility shim was added beyond defensive parsing of existing
  deployed Apps Script responses.

## Split-brain / race assessment

- LAN bind state still has one config source: `listen_host`.
- The checkbox only writes `127.0.0.1` or `0.0.0.0` when the current bind is
  not a custom address.
- LAN IP detection is read-only and synchronous; it does not send packets or
  mutate config.
- Relay response parsing remains centralized in `parse_relay_json()`.

## Verification

- `cargo fmt --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all-targets --features ui --quiet` (179 root tests + 5 UI/config tests)
- `python tools/run-repo-sanity.py`
- `python tools/check-doc-links.py`
- `python tools/check-repo-cleanliness.py`
- `cargo clippy --all-targets -- -D warnings` in `tunnel-node`
- `cargo test --all-targets --quiet` in `tunnel-node` (34 tests)
- Removed `tools/__pycache__` after Python checks regenerated it.

## Remaining risk

- Desktop still has a raw **Listen host** field for expert/custom bind cases;
  future UI modernization can demote it further into an advanced/custom-bind
  foldout once screenshot/UI regression coverage exists.
- Android does not yet show the detected desktop LAN endpoint because that
  endpoint is desktop-local information, not Android runtime state.
