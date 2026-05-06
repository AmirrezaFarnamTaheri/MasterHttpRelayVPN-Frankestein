# Batch 60 - Android Readiness Detail Localization

Date: 2026-05-05

## What Changed

- Moved Android selected-mode readiness-card detail prose out of
  `HomeScreen.kt` and into paired English/Persian string resources.
- Added localized detail copy for Apps Script deployment IDs, AUTH_KEY,
  serverless origin/path/auth, direct Google edge/SNI defaults, VPN vs
  proxy-only routing, LAN sharing guards, CA trust, Android app CA warnings,
  and full tunnel-node readiness hints.
- Changed the readiness builder to receive `Context` and resolve localized
  strings via `ctx.getString(...)`, while preserving the existing readiness
  IDs, `ok` state, blocking/warning classification, and Connect gating.
- Kept actual user-entered values such as serverless base URL, relay path,
  `google_ip`, `front_domain`, and `listen_host` visible as dynamic detail
  values.
- Documented that the current Android hard-coded-copy inventory does not yet
  detect every string-valued builder detail, so future builder prose must
  either move directly to resources or expand the scanner deliberately.

## Parity / Cleanup

- Android English and Persian string resources now have 270 paired keys.
- Scanner-visible hard-coded Android copy remains at 3 dynamic/technical tokens
  and 0 localization candidates.
- No desktop, backend, config schema, Apps Script, tunnel-node, or runtime
  behavior changed.
- No legacy compatibility path was added.

## Verification

- `python tools/generate-android-hardcoded-copy-inventory.py`
- `python tools/check-android-string-resource-parity.py`

Full batch verification is recorded in `elevation_audit_roadmap_source.md`.
