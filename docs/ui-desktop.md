# Desktop UI quick reference

This page maps the desktop UI (“mhrv-f-ui”) concepts to config keys and CLI commands.

## Core actions
- **Start / Stop**: runs or stops the local proxy engine (`mhrv-f serve` equivalent).
- **Test relay**: end-to-end probe (`mhrv-f test`).
- **Doctor / Doctor + Fix**: guided diagnostics (`mhrv-f doctor` / `mhrv-f doctor-fix`).

## Key fields (what they mean)
- **Mode** → `mode`
  - `apps_script`: classic relay + local MITM CA (HTTPS works only when CA trusted)
  - `google_only`: bootstrap mode to reach `script.google.com`
  - `full`: Full Tunnel mode (requires tunnel-node; no local CA on client)
- **Google IP** → `google_ip`
- **Front domain (SNI)** → `front_domain`
- **HTTP port** → `listen_port`
- **SOCKS5 port** → `socks5_port`

## Apps Script relay credentials
In modern configs, relay credentials live in account groups:
- **Account groups** → `account_groups[]`
  - `auth_key`
  - `script_ids` (one or many deployment IDs / URLs)

## Diagnostics tools
- **Scan IPs** → `mhrv-f scan-ips`
- **Test SNI pool** → `mhrv-f test-sni`
- **Scan SNI** → `mhrv-f scan-sni`

## Performance / stability knobs (advanced)
- **Range parallelism** → `range_parallelism`
  - What it does: for large GET downloads, the relay probes with a small `Range: bytes=0-...` then fetches the rest in parallel chunks through Apps Script.
  - Why it matters: Apps Script has a high per-request overhead; parallel range fetches make big downloads and video chunking far more usable.
- **Range chunk bytes** → `range_chunk_bytes`
  - What it does: chunk size for the range-parallel fetcher.
- **YouTube via relay** → `youtube_via_relay`
  - What it does: routes YouTube through Apps Script instead of the direct SNI-rewrite tunnel.
  - Trade-off: may bypass SafeSearch-on-SNI restrictions, but consumes Apps Script quota and uses the fixed Apps Script User-Agent.
- **Relay QPS limiter** → `relay_rate_limit_qps` / `relay_rate_limit_burst`
  - What it does: soft governor that smooths bursty relay call spikes (helps avoid quota stampedes on heavy pages).
- **Auto-tune** → `runtime_auto_tune` + `runtime_profile`
  - What it does: picks reasonable defaults for a few hot-path knobs when you don’t want to tune them manually.

