# Platform defaults

Contract version `1`. Canonical JSON: `docs/platform-defaults.json`.
CI runs `python3 tools/check-platform-defaults.py` (code versus JSON) and `python3 tools/generate-platform-defaults-doc.py -Check` (this page versus JSON).

## Purpose

Single contract for intentional platform default differences across Rust serde defaults, Kotlin MhrvConfig, and importer fallbacks.

## Shared expectations

| Setting | Value |
| --- | --- |
| `front_domain` | www.google.com |
| `listen_host_loopback` | 127.0.0.1 |
## Same on Rust + Android (`verify_ssl`, relay path, QUIC/DoH toggles)

| Setting | Value |
| --- | --- |
| `block_quic` | False |
| `serverless_relay_path` | /api/api |
| `tunnel_doh` | True |
| `verify_ssl` | True |
| `youtube_via_relay` | False |
## Rust Desktop / CLI (`serde` defaults in `src/config.rs`)

| Setting | Value |
| --- | --- |
| `google_ip_default` | 216.239.38.120 |
| `listen_port_default` | 8085 |
| `log_level_default` | warn |
| `socks5 when JSON omits socks5_port` | `null` (SOCKS5 listener disabled until `socks5_port` is set) |
| `parallel_relay when JSON omits field` | 0 |
| `coalesce_step_ms when JSON omits field` | 0 |
| `coalesce_max_ms when JSON omits field` | 0 |
| `notes SOCKS5/examples` | Serde default is omitted/null: SOCKS5 listener disabled until socks5_port is set. README and config*.example.json conventionally use 8086 beside HTTP 8085. |
| `notes parallel_relay` | Serde default is u8 zero: off for explicit parallel relay firing unless runtime tuning or imports raise it. Effective relay concurrency may still use round-robin when unset. |
| `notes coalesce` | Zero lets the Rust runtime derive full-mode adaptive coalescing timings. Kotlin defaults to concrete 10/1000 ms to match the current low-latency full-tunnel coalescing profile. |
## Android (`MhrvConfig` + importer fallbacks in `ConfigStore.kt`)

| Setting | Value |
| --- | --- |
| `google_ip_default` | 142.251.36.68 |
| `listen_port_default` | 8080 |
| `socks5_port_default` | 1081 |
| `log_level_default` | info |
| `parallel_relay_default` | 1 |
| `coalesce_step_ms_default` | 10 |
| `coalesce_max_ms_default` | 1000 |
## Rationale (intentional differences)

> **Ports**: Android keeps loopback proxies on 8080/1081 so built-in HELP copy and Proxy-only guides match historically shipped presets; Rust desktop/CLI uses 8085/8086 to reduce collisions with common local services.

> **Google edge IP preset**: Android presets a historically tested Google-anycast edge for captive/mobile paths; Rust desktop favors 216.239.38.120 with scan guidance. Imported configs overwrite both surfaces.

> **parallel_relay**: Android defaults `parallel_relay` to `1` (single parallel fan-out knob in the simplified UI stack). Rust defaults to `0` (off unless user or auto-tuning enables higher fan-out); desktop examples often raise it.

> **Coalesce timers**: Rust treats `coalesce_*_ms` as `0` when omitted (derive runtime-internal defaults). Android seeds `10`/`1000` so mobile saves match the current compiled low-latency profile unless the user explicitly restores older, slower coalescing.
