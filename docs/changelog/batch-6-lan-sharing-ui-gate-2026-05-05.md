# Batch 45 - LAN Sharing UI Drift Gate

Date: 2026-05-05

## Summary

- Added `tools/check-lan-sharing-ui.py` to protect the friendly desktop LAN
  sharing workflow.
- Wired the new gate into `tools/run-repo-sanity.py`.
- Added the new gate to `tools/check-ci-local-sanity-parity.py`, so CI/local
  parity fails if repo-sanity stops running it.
- Documented the gate in `tools/README.md`.

## Contract Now Guarded

- `src/lan_utils.rs` must keep `detect_lan_ip`, `is_share_on_lan`, and
  `is_loopback_only`, including their unit tests.
- LAN IP detection must keep the UDP route-table lookup and must reject
  unspecified addresses.
- Desktop must keep the **Share with other devices on my Wi-Fi / network**
  checkbox in **Sharing and per-app routing**.
- The checkbox must own the normal `127.0.0.1` to `0.0.0.0` transition.
- A manually configured bind address must show **Custom bind** and must not be
  overwritten by the checkbox/Save path.
- Desktop must keep copyable HTTP and SOCKS5 endpoints.
- Desktop must show detected LAN IP context or the `this-device-LAN-IP`
  fallback.
- Docs must keep the UDP route lookup explanation, the no-packet reassurance,
  `lan_allowlist`, and the SOCKS/token limitation.

## Parity Notes

- Desktop behavior, shared Rust helper code, docs, and CI/local sanity now share
  one static LAN sharing contract.
- Android is not changed in this batch; existing Android LAN sharing remains
  documented as a separate platform behavior.

## Concurrency / Split-Brain Notes

- No runtime code changed.
- The new check reduces split-brain by making the implemented UI and docs fail
  together if one side drifts.

## Cleanup

- No generated build output was intentionally created.
- Removed Python `__pycache__` directories created by local verification.
- No Gradle command was run.

## Verification

- `python tools/check-lan-sharing-ui.py`
- `python tools/check-ci-local-sanity-parity.py`
- `python tools/check-doc-links.py`
- `python tools/run-repo-sanity.py --skip-node`
- `python tools/check-repo-cleanliness.py`
- `python tools/run-repo-sanity.py`

All checks passed on 2026-05-05.
