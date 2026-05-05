# Config parity matrix (field × surface)

Generated from `docs/config-registry.json`.

| Field | Desktop | Android | Backend/runtime | Validation | Examples |
|---|---|---|---|---|---|
| `account_groups` | read/write | read/write (simple projection + preserve advanced) | runtime | required for apps_script/full; must include auth_key + >=1 script_id | `config.example.json` |
| `auto_blacklist_cooldown_secs` | read/write | preserve (unknown-root merge) | runtime | must be >0 when set | - |
| `auto_blacklist_strikes` | read/write | preserve (unknown-root merge) | runtime | must be >0 when set | - |
| `auto_blacklist_window_secs` | read/write | preserve (unknown-root merge) | runtime | must be >0 when set | - |
| `block_quic` | read/write | read/write | runtime | only meaningful for full-mode UDP/SOCKS5 datagrams | - |
| `bypass_doh_hosts` | read/write | preserve (unknown-root merge) | runtime | host match list | - |
| `coalesce_max_ms` | read/write | read/write | runtime | 0 uses compiled defaults; used for full-mode batch coalescing | - |
| `coalesce_step_ms` | read/write | read/write | runtime | 0 uses compiled defaults; used for full-mode batch coalescing | - |
| `config_version` | read/write | preserve | runtime | must be <= CURRENT_CONFIG_VERSION | `config.example.json`, `config.direct.example.json` |
| `domain_overrides` | read/write | preserve (unknown-root merge) | runtime | host match syntax; force_route allowed values | - |
| `enable_batching` | preserve (hidden/compat) | preserve | runtime | compat knob; reserved for JSON batch envelopes | - |
| `fetch_ips_from_api` | read/write | preserve (unknown-root merge) | runtime | enables Google IP discovery | - |
| `front_domain` | read/write | read/write | runtime | must be a DNS hostname; used for SNI-rewrite pool expansion | `config.example.json`, `config.direct.example.json` |
| `fronting_groups` | read/write | preserve (unknown-root merge + raw frontingGroupsJson) | runtime | sni must be valid hostname; ip must be IP; domains list must be non-empty | `config.fronting-groups.example.json` |
| `google_ip` | read/write | read/write (platform default differs) | runtime | validated by readiness/doctor probes; optional deep validation | `config.example.json`, `config.direct.example.json` |
| `google_ip_validation` | read/write | preserve (unknown-root merge) | runtime | enables frontend validation when scanning | - |
| `hosts` | read/write | preserve (unknown-root merge) | runtime | host override keys/values validated by routing logic | `config.example.json` |
| `lan_allowlist` | read/write | read/write | runtime | CIDR/IP parsing; required when exposing SOCKS5 on LAN (readiness warning) | - |
| `lan_token` | read/write | read/write | runtime | advisory; required when exposing HTTP on LAN (readiness warning) | - |
| `listen_host` | read/write | default only (phone UI uses loopback) | runtime | LAN exposure guidance + readiness warnings | `config.example.json` |
| `listen_port` | read/write | default only (Android uses 8080) | runtime | must not conflict with socks5_port | `config.example.json` |
| `log_level` | read/write | read/write | runtime | accepted by tracing filter; invalid values fall back or fail parse depending on path | `config.example.json` |
| `max_ips_to_scan` | read/write | preserve (unknown-root merge) | runtime | must be >=1 | - |
| `mode` | read/write | read/write | runtime | must be one of apps_script\|vercel_edge\|direct\|full (legacy google_only loads as direct) | `config.example.json`, `config.direct.example.json`, `config.vercel-edge.example.json` |
| `normalize_x_graphql` | read/write | preserve (unknown-root merge) | runtime | safe optional normalizer | - |
| `outage_reset_cooldown_ms` | read/write | preserve (unknown-root merge) | runtime | must be >0 when set | - |
| `outage_reset_enabled` | read/write | preserve (unknown-root merge) | runtime | null uses compiled default behavior | - |
| `outage_reset_failure_threshold` | read/write | preserve (unknown-root merge) | runtime | must be >0 when set | - |
| `outage_reset_window_ms` | read/write | preserve (unknown-root merge) | runtime | must be >0 when set | - |
| `parallel_relay` | read/write | read/write | runtime | clamped to available healthy script IDs/groups | - |
| `passthrough_hosts` | read/write | read/write | runtime | host match syntax validated by runtime | - |
| `range_chunk_bytes` | read/write | preserve (unknown-root merge) | runtime | must be reasonable size when set | - |
| `range_parallelism` | read/write | preserve (unknown-root merge) | runtime | must be >0 when set | - |
| `relay_rate_limit_burst` | read/write | preserve (unknown-root merge) | runtime | defaults derived from qps when unset | - |
| `relay_rate_limit_qps` | read/write | preserve (unknown-root merge) | runtime | must be >0 when set | - |
| `relay_request_timeout_secs` | read/write | preserve (unknown-root merge) | runtime | must be within sane range when set | - |
| `request_timeout_secs` | read/write | preserve (unknown-root merge) | runtime | clamped to [5,300] when set | - |
| `runtime_auto_tune` | read/write | preserve (unknown-root merge) | runtime | enables profile-driven derived defaults | - |
| `runtime_profile` | read/write | preserve (unknown-root merge) | runtime | eco\|balanced\|max_speed | - |
| `scan_batch_size` | read/write | preserve (unknown-root merge) | runtime | must be >=1 | - |
| `sni_hosts` | read/write | read/write | runtime | each entry must be a hostname | - |
| `socks5_port` | read/write | default only (Android uses 1081) | runtime | must not conflict with listen_port | `config.example.json` |
| `tunnel_doh` | read/write | preserve (unknown-root merge) | runtime | controls DoH bypass default | - |
| `upstream_socks5` | read/write | read/write | runtime | must parse as host:port; used for raw-TCP passthrough chaining | - |
| `vercel` | read/write | read/write (subset) | runtime | base_url must be set; auth_key required; relay_path required | `config.vercel-edge.example.json` |
| `verify_ssl` | read/write | read/write | runtime | controls upstream TLS verification to targets; does not disable local MITM semantics | `config.example.json` |
| `youtube_via_relay` | read/write | read/write | runtime | toggles YouTube HTML/API routing | - |

