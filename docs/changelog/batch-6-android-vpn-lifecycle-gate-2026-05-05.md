# Batch 6 - Android VPN Lifecycle Gate

Date: 2026-05-05

## Summary

Added a no-Gradle static guard for Android disconnect lifecycle and native
teardown ordering. This protects the upstream v1.9.8/v1.9.9 disconnect fixes
from regressing in local repo-sanity and CI repo-sanity without requiring local
Gradle execution.

## Changes

- Added `tools/check-android-vpn-lifecycle.py`.
- Wired the new gate into `tools/run-repo-sanity.py`.
- Documented the gate in `tools/README.md`.

## Guarded Contract

- `MainActivity.kt` sends Disconnect through `ACTION_STOP` with
  `startService(stopAction)`.
- `MainActivity.kt` executable code must not call `stopService()` for
  Disconnect; the service owns teardown and calls `stopSelf()` after cleanup.
- `MhrvVpnService.teardown()` must:
  - read and clear `proxyHandle`;
  - call `Native.stopProxy(handle)` before `Tun2proxy.stop()`;
  - close the TUN reference after the proxy/tun2proxy stop path;
  - join `tun2proxyThread` after the upstream socket has been closed;
  - flip UI running state only after native cleanup has run.

## Parity / Split-Brain Notes

- Android lifecycle ownership is now checked in the same repo-sanity path as
  config-sharing, support-redaction, platform-defaults, readiness, and stale
  Android string gates.
- No runtime behavior changed in this batch; this is a regression-prevention
  gate for behavior already ported in the preceding upstream absorption batch.
- The checker strips Kotlin comments before looking for forbidden
  `stopService()` code, so explanatory historical comments can remain.

## Verification

- `python tools/check-android-vpn-lifecycle.py`.
- `python tools/check-doc-links.py`.
- `python tools/check-repo-cleanliness.py`.
- `python tools/run-repo-sanity.py` (full local route including Node syntax,
  Apps Script helper tests, Python drift gates, readiness `-Check`, and
  JSON/XML/Android stale scan).
- No Gradle command was run by this batch. Process inspection still shows the
  pre-existing Gradle daemon noted in the previous batch; this work did not
  start or touch it.

## Cleanup

- Removed generated `tools/__pycache__`.
- Confirmed no remaining `__pycache__` directory under the workspace.
- No Android build output, Gradle cache/wrapper, APK, or JNI artifact was
  created.
