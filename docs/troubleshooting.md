# Troubleshooting (symptom → cause → next action)

## “Connect/Start is disabled” (Android) or “To start, fix: …” (Desktop)
- **Likely cause**: missing relay credentials (deployment ID/URL, auth_key) in Apps Script mode.
  - **Next**: add at least one enabled account group / deployment ID and auth key.
- **Likely cause**: ports invalid or conflicting.
  - **Next**: pick numeric ports; ensure HTTP and SOCKS5 ports are different.

## “504 Relay timeout”
- **Likely cause**: Apps Script deployment is stale, blocked, or quota-limited.
  - **Next**:
    - redeploy a “New version” in Apps Script and copy the new deployment ID
    - add multiple deployment IDs to spread load
    - add another Google account group for more quota

## “YouTube loads, but video buffers / stutters”
- **Likely cause**: video bytes come from `*.googlevideo.com` and may trigger lots of Apps Script fetches (call-count + quota pressure), especially on higher resolutions.
  - **Next**:
    - add more deployment IDs / more account groups (capacity)
    - reduce fan-out knobs (`parallel_relay`, `range_parallelism`) or enable `relay_rate_limit_qps` to smooth spikes
    - note: for `googlevideo.com` URLs that include `clen=`, the engine automatically uses larger chunks and caps in-flight concurrency to reduce Apps Script call count

## Certificate errors (e.g. `NET::ERR_CERT_AUTHORITY_INVALID`)
- **Likely cause**: the MITM CA is not trusted on this device (or the app does not opt in on Android 7+).
  - **Next**:
    - reinstall/re-trust the CA
    - on Android: some apps will never trust user CAs; use proxy-only or split tunneling
- **Safety note**: never share `ca.key`.

## “It says connected, but sites don’t load”
- **Likely cause**: wrong `google_ip` or wrong `front_domain` / SNI pool.
  - **Next**:
    - run SNI tester / `mhrv-f test-sni`
    - run scan IPs / `mhrv-f scan-ips`
    - try a different known-good `google_ip`
- **Likely cause**: wrong auth key.
  - **Next**: re-copy `AUTH_KEY` from `Code.gs` carefully (no trailing spaces).

## “UDP not working in Full Tunnel mode”
- **Likely cause**: version mismatch.
  - **Next**:
    - if the tunnel-node is older, UDP ops return `UNSUPPORTED_OP` and the client falls back to TCP-only
    - upgrade the tunnel-node and client to compatible releases

## “script.google.com is blocked (can’t deploy the relay)”
- **Next**: use **Google-only (bootstrap)** mode to reach `script.google.com`, deploy, then switch back to Apps Script mode.

