use rustls::pki_types::ServerName;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("failed to read config file {0}: {1}")]
    Read(String, #[source] std::io::Error),
    #[error("failed to parse config json: {0}")]
    Parse(#[from] serde_json::Error),
    #[error("invalid config: {0}")]
    Invalid(String),
}

/// Operating mode. `AppsScript` is the full client — MITMs TLS locally and
/// relays HTTP/HTTPS through a user-deployed Apps Script endpoint.
/// `Direct` is a no-relay path: only the SNI-rewrite tunnel is active,
/// targeting Google's edge by default plus any user-configured
/// `fronting_groups`. It is useful as a bootstrap for reaching
/// `script.google.com` before an Apps Script relay exists, and as a
/// standalone mode for users who only need fronted CDN/Google targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    AppsScript,
    VercelEdge,
    Direct,
    Full,
}

impl Mode {
    pub fn as_str(self) -> &'static str {
        match self {
            Mode::AppsScript => "apps_script",
            Mode::VercelEdge => "vercel_edge",
            Mode::Direct => "direct",
            Mode::Full => "full",
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
#[derive(Serialize)]
pub enum ScriptId {
    One(String),
    Many(Vec<String>),
}

impl ScriptId {
    pub fn into_vec(self) -> Vec<String> {
        let raw = match self {
            ScriptId::One(s) => vec![s],
            ScriptId::Many(v) => v,
        };
        let mut out = Vec::new();
        for id in raw {
            let id = normalize_script_id(&id);
            if !id.is_empty() && !out.iter().any(|seen| seen == &id) {
                out.push(id);
            }
        }
        out
    }
}

fn normalize_script_id(input: &str) -> String {
    let mut s = input.trim();
    if s.is_empty() {
        return String::new();
    }

    // Accept both bare deployment IDs and full Apps Script deployment URLs.
    // Older Python/Android configs stored whichever the user pasted; runtime
    // request paths must always contain only the ID.
    if let Some((_, tail)) = s.split_once("/macros/s/") {
        s = tail;
    }
    if let Some((head, _)) = s.split_once('/') {
        s = head;
    }
    if let Some((head, _)) = s.split_once('?') {
        s = head;
    }
    s.trim().to_string()
}

fn default_config_version() -> u32 {
    1
}

fn default_weight() -> u8 {
    1
}

fn default_enabled() -> bool {
    true
}

/// Multi-account Apps Script configuration.
///
/// Each account group represents one Google account (or quota pool) with its
/// own shared secret and one or more deployment IDs. Runtime routing/failover
/// is implemented in the relay layer; the config schema lives here.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AccountGroup {
    /// Optional label shown in UI/logs.
    #[serde(default)]
    pub label: Option<String>,
    /// Shared secret (must match AUTH_KEY in the deployed Apps Script).
    #[serde(default)]
    pub auth_key: String,
    /// One or more deployment IDs for this account group.
    pub script_ids: ScriptId,
    /// Relative weight/priority for selection when multiple groups are healthy.
    #[serde(default = "default_weight")]
    pub weight: u8,
    /// Allow disabling a group without removing it.
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

fn default_vercel_relay_path() -> String {
    "/api/api".into()
}

fn default_vercel_verify_tls() -> bool {
    true
}

fn default_vercel_max_body_bytes() -> usize {
    4 * 1024 * 1024
}

/// Native serverless Edge JSON relay configuration.
///
/// This is intentionally separate from `account_groups`: Apps Script accounts
/// rotate deployment IDs and quotas, while Vercel is a single authenticated
/// fetch endpoint with different limits and trust assumptions.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VercelConfig {
    /// Deployment origin, for example `https://my-relay.vercel.app` or
    /// `https://my-relay.netlify.app`.
    #[serde(default)]
    pub base_url: String,
    /// Edge function path. Defaults to the bundled tool's `/api/api`.
    #[serde(default = "default_vercel_relay_path")]
    pub relay_path: String,
    /// Shared secret. Must match AUTH_KEY in the serverless project.
    #[serde(default)]
    pub auth_key: String,
    /// Whether the client verifies the relay TLS certificate.
    #[serde(default = "default_vercel_verify_tls")]
    pub verify_tls: bool,
    /// Upper bound for a single decoded request body sent to the relay.
    #[serde(default = "default_vercel_max_body_bytes")]
    pub max_body_bytes: usize,
    /// Reserved for the JSON batch envelope `{k,q:[...]}`.
    #[serde(default)]
    pub enable_batching: bool,
}

impl Default for VercelConfig {
    fn default() -> Self {
        Self {
            base_url: String::new(),
            relay_path: default_vercel_relay_path(),
            auth_key: String::new(),
            verify_tls: default_vercel_verify_tls(),
            max_body_bytes: default_vercel_max_body_bytes(),
            enable_batching: false,
        }
    }
}

pub const CURRENT_CONFIG_VERSION: u32 = 1;

/// Per-domain overrides for routing and performance knobs.
///
/// This is intentionally small and safe: it's meant to handle the common
/// real-world cases (force direct for fragile sites, force relay for blocked
/// sites, disable range-parallel chunking for anti-bot flows).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DomainOverride {
    /// Hostname match rule. Supports:
    /// - exact: "example.com"
    /// - suffix: ".example.com" (matches example.com and any subdomain)
    pub host: String,
    /// Force the high-level route for this host.
    /// Allowed values: "direct", "sni_rewrite", "relay", "full_tunnel".
    #[serde(default)]
    pub force_route: Option<String>,
    /// If true, disable range-parallel chunking for this host (treat as single relay()).
    #[serde(default)]
    pub never_chunk: bool,
}

/// One multi-edge fronting group. Multi-tenant CDNs can host many sites on one
/// edge pool and dispatch by the inner HTTP Host header after TLS terminates.
/// A group lets the local MITM path re-encrypt to `ip` while using `sni` as
/// the upstream TLS name, then send the original requested host inside TLS.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FrontingGroup {
    /// Human-readable label used in logs and docs.
    pub name: String,
    /// Edge IP address to dial.
    pub ip: String,
    /// Upstream TLS SNI. Must be a real DNS name served by that edge.
    pub sni: String,
    /// Domains routed through this edge. Entries match exact host and
    /// dot-anchored subdomains, case-insensitively.
    pub domains: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    /// Schema version for the config file format.
    #[serde(default = "default_config_version")]
    pub config_version: u32,
    pub mode: String,
    #[serde(default = "default_google_ip")]
    pub google_ip: String,
    #[serde(default = "default_front_domain")]
    pub front_domain: String,
    #[serde(default = "default_listen_host")]
    pub listen_host: String,
    #[serde(default = "default_listen_port")]
    pub listen_port: u16,
    #[serde(default)]
    pub socks5_port: Option<u16>,
    #[serde(default = "default_log_level")]
    pub log_level: String,
    #[serde(default = "default_verify_ssl")]
    pub verify_ssl: bool,
    #[serde(default)]
    pub hosts: HashMap<String, String>,
    #[serde(default)]
    pub enable_batching: bool,
    /// Optional upstream SOCKS5 proxy for non-HTTP / raw-TCP traffic
    /// (e.g. `"127.0.0.1:50529"` pointing at a local xray / v2ray instance).
    /// When set, the SOCKS5 listener forwards raw-TCP flows through it
    /// instead of connecting directly. HTTP/HTTPS traffic (which goes
    /// through the Apps Script relay) and SNI-rewrite tunnels are
    /// unaffected.
    #[serde(default)]
    pub upstream_socks5: Option<String>,
    /// Fan-out factor for non-cached relay requests when multiple
    /// `script_id`s are configured. `0` or `1` = off (round-robin, the
    /// default). `2` or more = fire that many Apps Script instances in
    /// parallel per request and return the first successful response —
    /// kills long-tail latency caused by a single slow Apps Script
    /// instance, at the cost of using that much more daily quota.
    /// Value is clamped to the number of available (non-blacklisted)
    /// script IDs.
    #[serde(default)]
    pub parallel_relay: u8,

    /// Adaptive full-mode batch coalescing: after each tunnel op arrives,
    /// wait this many milliseconds for more ops before firing the batch.
    /// The timer resets on each arrival. 0 = compiled default.
    #[serde(default)]
    pub coalesce_step_ms: u16,
    /// Hard cap on total adaptive coalescing wait. 0 = compiled default.
    #[serde(default)]
    pub coalesce_max_ms: u16,

    /// Optional client-side rate limit for Apps Script relay calls.
    /// Useful as a soft resource governor (avoid bursty quota spikes).
    ///
    /// Units: requests per second. When unset, no rate limiting is applied.
    #[serde(default)]
    pub relay_rate_limit_qps: Option<f64>,
    /// Optional burst size for the relay rate limiter (token bucket capacity).
    /// When unset, defaults to `max(1, qps.ceil() as u32)`.
    #[serde(default)]
    pub relay_rate_limit_burst: Option<u32>,

    /// Adaptive runtime profile selector. Only takes effect when
    /// `runtime_auto_tune=true`. When unset, defaults to `balanced`.
    ///
    /// - `eco`: minimize quota + reduce concurrency
    /// - `balanced`: sensible defaults
    /// - `max_speed`: prioritize latency/throughput (higher quota usage)
    #[serde(default)]
    pub runtime_profile: Option<String>,

    /// When true, apply profile-driven tuning for a few hot-path knobs
    /// (parallel_relay, range-parallelism, request timeouts).
    ///
    /// When false (default), explicit config values are used as-is.
    #[serde(default)]
    pub runtime_auto_tune: bool,

    /// Optional override: maximum number of concurrent range chunk fetches
    /// in `relay_parallel_range`. When omitted, derived from runtime profile.
    #[serde(default)]
    pub range_parallelism: Option<u8>,

    /// Optional override: chunk size in bytes used by `relay_parallel_range`.
    /// When omitted, derived from runtime profile.
    #[serde(default)]
    pub range_chunk_bytes: Option<u64>,

    /// Optional override: relay request timeout (seconds). When omitted,
    /// derived from runtime profile.
    #[serde(default)]
    pub relay_request_timeout_secs: Option<u64>,

    /// Optional override: full-mode batch HTTP timeout (seconds). This is
    /// separate from `relay_request_timeout_secs`, which controls ordinary
    /// Apps Script HTTP relay calls. Full mode batches can carry many active
    /// TCP/UDP sessions at once, so slow networks may need `45` or `60`, while
    /// fail-fast multi-deployment setups may prefer lower values. Clamped to
    /// `[5, 300]`.
    #[serde(default)]
    pub request_timeout_secs: Option<u64>,

    /// Full-mode auto-blacklist tuning. Timeout is a noisy signal on flaky
    /// networks, so batch timeouts use a strike window before cooling down a
    /// deployment. Defaults match the upstream v1.9.1 operator guidance:
    /// 3 strikes / 30s window / 120s cooldown.
    #[serde(default)]
    pub auto_blacklist_strikes: Option<u32>,
    #[serde(default)]
    pub auto_blacklist_window_secs: Option<u64>,
    #[serde(default)]
    pub auto_blacklist_cooldown_secs: Option<u64>,

    /// Optional explicit SNI rotation pool for outbound TLS to `google_ip`.
    /// Empty / missing = auto-expand from `front_domain` using the built-in
    /// Google edge candidate pool. Set to an explicit list
    /// to pick exactly which SNI names get rotated through. Test per-name via
    /// the UI or `mhrv-f test-sni`.
    #[serde(default)]
    pub sni_hosts: Option<Vec<String>>,
    #[serde(default = "default_fetch_ips_from_api")]
    pub fetch_ips_from_api: bool,

    #[serde(default = "default_max_ips_to_scan")]
    pub max_ips_to_scan: usize,

    #[serde(default = "default_scan_batch_size")]
    pub scan_batch_size: usize,

    #[serde(default = "default_google_ip_validation")]
    pub google_ip_validation: bool,
    /// When true, GET requests to `x.com`/`twitter.com`
    /// `/i/api/graphql/<hash>/<op>?variables=…` have their query trimmed to
    /// just the `variables=` param before being relayed. The `features` /
    /// `fieldToggles` params that X ships with these requests change
    /// frequently and bust the response cache — stripping them dramatically
    /// improves hit rate on Twitter/X browsing.
    ///
    /// Based on the community X GraphQL cache pattern.
    ///
    /// Off by default — some X endpoints may reject calls that omit
    /// features. Turn on and observe.
    #[serde(default)]
    pub normalize_x_graphql: bool,

    /// Route YouTube HTML/API traffic through the Apps Script relay instead
    /// of the direct SNI-rewrite tunnel.
    ///
    /// Why this exists: when YouTube is SNI-rewritten to `google_ip`
    /// with `SNI=www.google.com`, Google's frontend can enforce
    /// SafeSearch / Restricted Mode based on the SNI → some videos show
    /// as "restricted." Routing through Apps Script bypasses that check
    /// (it hits YouTube from Google's own backend, not via www.google.com
    /// SNI) but introduces the UrlFetchApp User-Agent and quota costs.
    ///
    /// Trade-off: enabling removes SafeSearch-on-SNI for YouTube page/API
    /// surfaces, adds `User-Agent: Google-Apps-Script` header, and counts
    /// those calls against your Apps Script quota. CDN assets such as
    /// `ytimg.com` stay on SNI-rewrite to avoid wasting quota; `googlevideo.com`
    /// still uses the normal relay path because it is served by separate Google
    /// video edges, not the regular GFE `google_ip`. Off by default.
    #[serde(default)]
    pub youtube_via_relay: bool,

    /// User-configurable passthrough list. Any host whose name matches
    /// one of these entries bypasses the Apps Script relay entirely and
    /// is plain-TCP-passthroughed (optionally through `upstream_socks5`).
    ///
    /// Accepts exact hostnames ("example.com"), leading-dot suffixes
    /// (".internal.example"), and wildcard suffix aliases
    /// ("*.internal.example"). Suffix rules also match the bare parent.
    /// Matches are case-insensitive.
    ///
    /// Dispatched BEFORE SNI-rewrite and Apps Script, so a passthrough
    /// entry wins over the default Google-edge routing. Useful for
    /// sites where you already have reachability without the relay
    /// (saving Apps Script quota) or for hosts that break under MITM.
    #[serde(default)]
    pub passthrough_hosts: Vec<String>,

    /// Block SOCKS5 UDP datagrams to port 443 before they enter the full
    /// tunnel. This forces QUIC/HTTP3 clients to fall back to TCP/HTTPS
    /// without spending Apps Script batches on UDP/443 probes. Off by default
    /// because some full-mode users intentionally want UDP/443 carried.
    #[serde(default)]
    pub block_quic: bool,

    /// Opt-out for the DoH bypass. Default true keeps known browser DoH
    /// endpoints inside the selected tunnel instead of sending them direct.
    /// Set false only on networks where direct DoH reliably works and you
    /// prefer the latency win.
    #[serde(default = "default_tunnel_doh")]
    pub tunnel_doh: bool,
    /// Extra DNS-over-HTTPS hostnames to bypass in addition to the built-in
    /// list. Entries match exact hosts and dot-anchored subdomains; unlike
    /// `passthrough_hosts`, a leading dot is optional for suffix matching.
    #[serde(default)]
    pub bypass_doh_hosts: Vec<String>,

    /// Optional multi-edge domain-fronting groups. Matched hosts are handled
    /// by the same local-MITM/SNI-rewrite machinery as the Google edge path,
    /// but use the group's configured `(ip, sni)` pair.
    ///
    /// Empty / missing means feature off. In `full` mode these groups are
    /// inert because full mode preserves end-to-end TLS through tunnel-node.
    #[serde(default)]
    pub fronting_groups: Vec<FrontingGroup>,

    /// Per-domain overrides for routing and performance knobs.
    ///
    /// Matches are case-insensitive. Order matters: the first matching rule wins.
    /// Host match supports exact and leading-dot suffix, same style as
    /// `passthrough_hosts`.
    #[serde(default)]
    pub domain_overrides: Vec<DomainOverride>,

    /// Optional multi-account Apps Script pools.
    ///
    /// Canonical configuration: Apps Script routing always uses these pools.
    /// The runtime picks between groups for quota resilience.
    ///
    /// In `direct` / legacy `google_only` bootstrap mode this may be omitted.
    #[serde(default)]
    pub account_groups: Option<Vec<AccountGroup>>,

    /// Serverless Edge JSON relay settings, used only in `vercel_edge` mode.
    #[serde(default)]
    pub vercel: VercelConfig,

    /// Optional access token required when binding to LAN (listen_host=0.0.0.0/::).
    /// When set, the HTTP proxy requires `X-MHRV-F-Token: <token>` on plain HTTP
    /// requests and on CONNECT preface. SOCKS5 has no header preface, so protect
    /// LAN-exposed SOCKS5 with `lan_allowlist` instead. This is a basic
    /// guardrail, not a full authentication system.
    #[serde(default)]
    pub lan_token: Option<String>,

    /// Optional allowlist of client IPs/CIDRs allowed to connect when LAN-bound.
    /// Accepts exact IP addresses and CIDR ranges.
    #[serde(default)]
    pub lan_allowlist: Option<Vec<String>>,

    /// Transport self-heal: if we observe N eligible relay failures within a short
    /// rolling window, we proactively reset the fronted connection pool so we
    /// don't get stuck reusing a poisoned keep-alive path.
    ///
    /// Inspired by the Go dual-relay's `OutageResetTracker` (windowed failure
    /// threshold + cooldown). Eligible failure categories are:
    /// - timeout
    /// - unreachable
    /// - overloaded
    ///
    /// Defaults are conservative and safe to omit.
    #[serde(default)]
    pub outage_reset_enabled: Option<bool>,
    #[serde(default)]
    pub outage_reset_failure_threshold: Option<u32>,
    #[serde(default)]
    pub outage_reset_window_ms: Option<u64>,
    #[serde(default)]
    pub outage_reset_cooldown_ms: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeProfile {
    Eco,
    Balanced,
    MaxSpeed,
}

impl RuntimeProfile {
    pub fn as_str(self) -> &'static str {
        match self {
            RuntimeProfile::Eco => "eco",
            RuntimeProfile::Balanced => "balanced",
            RuntimeProfile::MaxSpeed => "max_speed",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "eco" => Some(RuntimeProfile::Eco),
            "balanced" => Some(RuntimeProfile::Balanced),
            "max_speed" | "max-speed" | "maxspeed" => Some(RuntimeProfile::MaxSpeed),
            _ => None,
        }
    }
}

fn default_fetch_ips_from_api() -> bool {
    false
}
fn default_max_ips_to_scan() -> usize {
    100
}
fn default_scan_batch_size() -> usize {
    500
}
fn default_google_ip_validation() -> bool {
    true
}
fn default_tunnel_doh() -> bool {
    true
}

fn default_google_ip() -> String {
    "216.239.38.120".into()
}
fn default_front_domain() -> String {
    "www.google.com".into()
}
fn default_listen_host() -> String {
    "127.0.0.1".into()
}
fn default_listen_port() -> u16 {
    8085
}
fn default_log_level() -> String {
    "warn".into()
}
fn default_verify_ssl() -> bool {
    true
}

impl Config {
    pub fn from_json_str(data: &str) -> Result<Self, ConfigError> {
        let mut value: Value = serde_json::from_str(data)?;
        migrate_legacy_android_account_groups(&mut value);
        Self::from_json_value(value)
    }

    fn from_json_value(value: Value) -> Result<Self, ConfigError> {
        let cfg: Config = serde_json::from_value(value)?;
        cfg.validate_config_version()?;
        cfg.validate()?;
        Ok(cfg)
    }

    pub fn load(path: &Path) -> Result<Self, ConfigError> {
        let data = std::fs::read_to_string(path)
            .map_err(|e| ConfigError::Read(path.display().to_string(), e))?;
        let mut value: Value = serde_json::from_str(&data)?;
        migrate_legacy_android_account_groups(&mut value);
        Self::from_json_value(value)
    }

    fn validate_config_version(&self) -> Result<(), ConfigError> {
        if self.config_version > CURRENT_CONFIG_VERSION {
            return Err(ConfigError::Invalid(format!(
                "config_version {} is newer than this binary supports (max {}). Please update mhrv-f.",
                self.config_version, CURRENT_CONFIG_VERSION
            )));
        }
        Ok(())
    }

    fn validate(&self) -> Result<(), ConfigError> {
        let mode = self.mode_kind()?;
        if mode == Mode::AppsScript || mode == Mode::Full {
            let Some(groups) = &self.account_groups else {
                return Err(ConfigError::Invalid(
                    "account_groups is required in apps_script/full mode".into(),
                ));
            };
            if groups.is_empty() {
                return Err(ConfigError::Invalid("account_groups is empty".into()));
            }
            let mut any_enabled = false;
            for (idx, g) in groups.iter().enumerate() {
                if !g.enabled {
                    continue;
                }
                any_enabled = true;
                if g.auth_key.trim().is_empty() {
                    return Err(ConfigError::Invalid(format!(
                        "account_groups[{}].auth_key is required",
                        idx
                    )));
                }
                let ids = g.script_ids.clone().into_vec();
                if ids.is_empty() {
                    return Err(ConfigError::Invalid(format!(
                        "account_groups[{}].script_ids is required",
                        idx
                    )));
                }
                for id in &ids {
                    if id.is_empty() {
                        return Err(ConfigError::Invalid(format!(
                            "account_groups[{}].script_ids contains an empty id",
                            idx
                        )));
                    }
                }
            }
            if !any_enabled {
                return Err(ConfigError::Invalid(
                    "all account_groups are disabled; enable at least one".into(),
                ));
            }
        }
        if mode == Mode::VercelEdge {
            let base = self.vercel.base_url.trim();
            if base.is_empty() {
                return Err(ConfigError::Invalid(
                    "vercel.base_url is required in vercel_edge mode".into(),
                ));
            }
            let parsed = url::Url::parse(base).map_err(|e| {
                ConfigError::Invalid(format!("vercel.base_url is not a valid URL: {e}"))
            })?;
            if !matches!(parsed.scheme(), "https" | "http") {
                return Err(ConfigError::Invalid(
                    "vercel.base_url must start with https:// or http://".into(),
                ));
            }
            if parsed.host_str().unwrap_or("").trim().is_empty() {
                return Err(ConfigError::Invalid(
                    "vercel.base_url must include a hostname".into(),
                ));
            }
            let auth = self.vercel.auth_key.trim();
            if auth.is_empty()
                || auth.eq_ignore_ascii_case("change-me")
                || auth.eq_ignore_ascii_case("your_auth_key")
                || auth.eq_ignore_ascii_case("your-auth-key")
                || auth.eq_ignore_ascii_case("same_value_as_vercel_auth_key")
            {
                return Err(ConfigError::Invalid(
                    "vercel.auth_key must be set to a non-placeholder AUTH_KEY".into(),
                ));
            }
            if self.vercel.relay_path.trim().is_empty()
                || !self.vercel.relay_path.trim().starts_with('/')
            {
                return Err(ConfigError::Invalid(
                    "vercel.relay_path must start with '/' (default: /api/api)".into(),
                ));
            }
            if self.vercel.max_body_bytes < 1024 {
                return Err(ConfigError::Invalid(
                    "vercel.max_body_bytes must be at least 1024".into(),
                ));
            }
        }
        if self.scan_batch_size == 0 {
            return Err(ConfigError::Invalid(
                "scan_batch_size must be greater than 0".into(),
            ));
        }
        if self.socks5_port == Some(self.listen_port) {
            return Err(ConfigError::Invalid(format!(
                "listen_port and socks5_port must differ on the same host (both set to {} on {}). Change one of them in config.json.",
                self.listen_port, self.listen_host
            )));
        }
        if let Some(qps) = self.relay_rate_limit_qps {
            if !(qps.is_finite()) || qps <= 0.0 {
                return Err(ConfigError::Invalid(
                    "relay_rate_limit_qps must be a positive finite number".into(),
                ));
            }
        }
        for (idx, o) in self.domain_overrides.iter().enumerate() {
            if o.host.trim().trim_end_matches('.').is_empty() {
                return Err(ConfigError::Invalid(format!(
                    "domain_overrides[{}].host is required",
                    idx
                )));
            }
            if let Some(route) = o.force_route.as_deref() {
                let r = route.trim().to_ascii_lowercase();
                let ok = matches!(
                    r.as_str(),
                    "direct" | "sni_rewrite" | "relay" | "full_tunnel"
                );
                if !ok {
                    return Err(ConfigError::Invalid(format!(
                        "domain_overrides[{}].force_route must be one of: direct, sni_rewrite, relay, full_tunnel",
                        idx
                    )));
                }
            }
        }
        for (i, group) in self.fronting_groups.iter().enumerate() {
            if group.name.trim().is_empty() {
                return Err(ConfigError::Invalid(format!(
                    "fronting_groups[{}]: name is empty",
                    i
                )));
            }
            if group.ip.trim().is_empty() {
                return Err(ConfigError::Invalid(format!(
                    "fronting_groups[{}] ('{}'): ip is empty",
                    i, group.name
                )));
            }
            if group.sni.trim().is_empty() {
                return Err(ConfigError::Invalid(format!(
                    "fronting_groups[{}] ('{}'): sni is empty",
                    i, group.name
                )));
            }
            if let Err(e) = ServerName::try_from(group.sni.clone()) {
                return Err(ConfigError::Invalid(format!(
                    "fronting_groups[{}] ('{}'): invalid sni '{}': {}",
                    i, group.name, group.sni, e
                )));
            }
            if group.domains.is_empty() {
                return Err(ConfigError::Invalid(format!(
                    "fronting_groups[{}] ('{}'): domains list is empty",
                    i, group.name
                )));
            }
            for domain in &group.domains {
                if domain.trim().is_empty() {
                    return Err(ConfigError::Invalid(format!(
                        "fronting_groups[{}] ('{}'): empty domain entry",
                        i, group.name
                    )));
                }
            }
        }
        Ok(())
    }

    pub fn mode_kind(&self) -> Result<Mode, ConfigError> {
        match self.mode.as_str() {
            "apps_script" => Ok(Mode::AppsScript),
            "vercel_edge" => Ok(Mode::VercelEdge),
            "direct" | "google_only" => Ok(Mode::Direct),
            "full" => Ok(Mode::Full),
            other => Err(ConfigError::Invalid(format!(
                "unknown mode '{}' (expected 'apps_script', 'vercel_edge', 'direct', or 'full')",
                other
            ))),
        }
    }

    /// Resolve enabled account pools (canonical).
    pub fn account_groups_resolved(&self) -> Vec<AccountGroup> {
        self.account_groups
            .clone()
            .unwrap_or_default()
            .into_iter()
            .filter(|g| g.enabled)
            .collect()
    }

    /// Human-facing warnings for unsafe/misleading settings.
    /// These do not block startup; they are surfaced in UI/CLI.
    pub fn unsafe_warnings(&self) -> Vec<String> {
        let mut out = Vec::new();

        // LAN exposure: binding 0.0.0.0 opens the proxy to the local network.
        let host = self.listen_host.trim();
        if host == "0.0.0.0" || host == "::" {
            out.push(
                "listen_host is 0.0.0.0 / :: (LAN-exposed). Anyone on your network can use your proxy. Prefer 127.0.0.1 unless you add access controls."
                    .into(),
            );
            if self.lan_token.as_deref().unwrap_or("").trim().is_empty()
                && self
                    .lan_allowlist
                    .as_ref()
                    .map(|v| v.is_empty())
                    .unwrap_or(true)
            {
                out.push(
                    "LAN-exposed without lan_token or lan_allowlist. Set at least one to reduce risk."
                        .into(),
                );
            }
            if !self.lan_token.as_deref().unwrap_or("").trim().is_empty()
                && self
                    .lan_allowlist
                    .as_ref()
                    .map(|v| v.is_empty())
                    .unwrap_or(true)
            {
                out.push(
                    "LAN-exposed SOCKS5 cannot use lan_token headers; set lan_allowlist too if you need SOCKS5 over LAN."
                        .into(),
                );
            }
        }

        if !self.verify_ssl {
            out.push("verify_ssl=false disables TLS certificate verification. This is insecure and can hide MITM attacks.".into());
        }

        // Weak/default auth key.
        if matches!(self.mode_kind(), Ok(Mode::AppsScript | Mode::Full)) {
            if let Some(groups) = &self.account_groups {
                for g in groups {
                    if !g.enabled {
                        continue;
                    }
                    if g.auth_key.trim().len() < 12 {
                        out.push("An account_groups auth_key looks short. Use a long random secret (must match AUTH_KEY in Code.gs).".into());
                        break;
                    }
                }
            }
        }
        if matches!(self.mode_kind(), Ok(Mode::VercelEdge)) {
            if self.vercel.auth_key.trim().len() < 12 {
                out.push("vercel.auth_key looks short. Use a long random secret and set the same value as AUTH_KEY in the serverless relay.".into());
            }
            if !self.vercel.verify_tls {
                out.push("vercel.verify_tls=false disables TLS certificate verification for the serverless relay. Keep it true unless you are testing a local endpoint.".into());
            }
            if self.vercel.base_url.trim().starts_with("http://") {
                out.push(
                    "vercel.base_url uses plain HTTP. Use https:// for real serverless deployments."
                        .into(),
                );
            }
            if self.vercel.enable_batching {
                out.push("vercel.enable_batching is experimental; if your deployed relay is old or protected by platform auth, the client will fall back to single JSON fetches.".into());
            }
        }

        out
    }

    pub fn runtime_profile_kind(&self) -> RuntimeProfile {
        self.runtime_profile
            .as_deref()
            .and_then(RuntimeProfile::parse)
            .unwrap_or(RuntimeProfile::Balanced)
    }

    pub fn effective_parallel_relay(&self) -> usize {
        // Preserve historical semantics unless runtime_auto_tune is enabled.
        if !self.runtime_auto_tune {
            return self.parallel_relay as usize;
        }
        // If user explicitly set >=2, honour it. If 0/1, derive by profile.
        if self.parallel_relay >= 2 {
            return self.parallel_relay as usize;
        }
        match self.runtime_profile_kind() {
            RuntimeProfile::Eco => 1,
            RuntimeProfile::Balanced => 2,
            RuntimeProfile::MaxSpeed => 3,
        }
    }

    pub fn effective_range_parallelism(&self) -> usize {
        if let Some(v) = self.range_parallelism {
            return v.max(1) as usize;
        }
        match self.runtime_profile_kind() {
            RuntimeProfile::Eco => 6,
            RuntimeProfile::Balanced => 12,
            RuntimeProfile::MaxSpeed => 16,
        }
    }

    pub fn effective_range_chunk_bytes(&self) -> u64 {
        if let Some(v) = self.range_chunk_bytes {
            return v.max(16 * 1024);
        }
        match self.runtime_profile_kind() {
            RuntimeProfile::Eco => 384 * 1024,
            RuntimeProfile::Balanced => 256 * 1024,
            RuntimeProfile::MaxSpeed => 256 * 1024,
        }
    }

    pub fn effective_relay_request_timeout_secs(&self) -> u64 {
        if let Some(v) = self.relay_request_timeout_secs {
            return v.clamp(5, 120);
        }
        match self.runtime_profile_kind() {
            RuntimeProfile::Eco => 20,
            RuntimeProfile::Balanced => 25,
            RuntimeProfile::MaxSpeed => 30,
        }
    }

    pub fn effective_batch_request_timeout_secs(&self) -> u64 {
        self.request_timeout_secs.unwrap_or(30).clamp(5, 300)
    }

    pub fn effective_auto_blacklist_strikes(&self) -> u32 {
        self.auto_blacklist_strikes.unwrap_or(3).clamp(1, 100)
    }

    pub fn effective_auto_blacklist_window_secs(&self) -> u64 {
        self.auto_blacklist_window_secs
            .unwrap_or(30)
            .clamp(1, 86_400)
    }

    pub fn effective_auto_blacklist_cooldown_secs(&self) -> u64 {
        self.auto_blacklist_cooldown_secs
            .unwrap_or(120)
            .clamp(1, 86_400)
    }
}

fn migrate_legacy_android_account_groups(value: &mut Value) {
    let Some(obj) = value.as_object_mut() else {
        return;
    };
    if obj
        .get("account_groups")
        .is_some_and(|groups| !groups.is_null())
    {
        return;
    }

    let has_legacy_ids = obj.get("script_ids").is_some_and(|ids| !ids.is_null());
    let has_legacy_auth = obj.get("auth_key").is_some_and(|auth| !auth.is_null());
    if !has_legacy_ids && !has_legacy_auth {
        return;
    }

    let mut group = serde_json::Map::new();
    group.insert(
        "label".into(),
        Value::String("legacy-android-primary".into()),
    );
    group.insert(
        "auth_key".into(),
        obj.get("auth_key")
            .cloned()
            .unwrap_or_else(|| Value::String(String::new())),
    );
    group.insert(
        "script_ids".into(),
        obj.get("script_ids")
            .cloned()
            .unwrap_or_else(|| Value::Array(Vec::new())),
    );
    group.insert("weight".into(), Value::from(default_weight()));
    group.insert("enabled".into(), Value::Bool(default_enabled()));
    obj.insert(
        "account_groups".into(),
        Value::Array(vec![Value::Object(group)]),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apps_script_requires_account_groups() {
        let s = r#"{
            "mode": "apps_script"
        }"#;
        let cfg: Config = serde_json::from_str(s).unwrap();
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn parses_account_groups() {
        let s = r#"{
            "mode": "apps_script",
            "account_groups": [{
                "label": "primary",
                "auth_key": "test-auth-key-please-change-32chars",
                "script_ids": ["A", "B", "C"],
                "weight": 1,
                "enabled": true
            }]
        }"#;
        let cfg: Config = serde_json::from_str(s).unwrap();
        cfg.validate().unwrap();
    }

    #[test]
    fn migrates_legacy_android_root_script_ids() {
        let s = r#"{
            "mode": "apps_script",
            "script_ids": ["AKfycb_legacy_1", "https://script.google.com/macros/s/AKfycb_legacy_2/exec"],
            "auth_key": "test-auth-key-please-change-32chars"
        }"#;
        let cfg = Config::from_json_str(s).expect("legacy Android config should load");
        let groups = cfg.account_groups.expect("legacy fields should migrate");

        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].label.as_deref(), Some("legacy-android-primary"));
        assert_eq!(groups[0].auth_key, "test-auth-key-please-change-32chars");
        assert_eq!(
            groups[0].script_ids.clone().into_vec(),
            vec!["AKfycb_legacy_1", "AKfycb_legacy_2"]
        );
    }

    #[test]
    fn canonical_account_groups_take_precedence_over_legacy_android_fields() {
        let s = r#"{
            "mode": "apps_script",
            "script_ids": ["AKfycb_legacy"],
            "auth_key": "legacy-auth-key",
            "account_groups": [{
                "label": "canonical",
                "auth_key": "test-auth-key-please-change-32chars",
                "script_ids": "AKfycb_canonical"
            }]
        }"#;
        let cfg = Config::from_json_str(s).expect("canonical config should load");
        let groups = cfg.account_groups.expect("canonical groups should remain");

        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].label.as_deref(), Some("canonical"));
        assert_eq!(groups[0].auth_key, "test-auth-key-please-change-32chars");
        assert_eq!(
            groups[0].script_ids.clone().into_vec(),
            vec!["AKfycb_canonical"]
        );
    }

    #[test]
    fn normalizes_script_ids_from_urls_and_deduplicates() {
        let ids = ScriptId::Many(vec![
            " https://script.google.com/macros/s/AKfyc_URL_1/exec ".into(),
            "AKfyc_URL_1".into(),
            "https://script.google.com/macros/s/AKfyc_URL_2/dev?foo=bar".into(),
        ])
        .into_vec();
        assert_eq!(ids, vec!["AKfyc_URL_1", "AKfyc_URL_2"]);
    }

    #[test]
    fn rejects_wrong_mode() {
        let s = r#"{
            "mode": "domain_fronting",
            "account_groups": [{
                "auth_key": "x",
                "script_ids": "A"
            }]
        }"#;
        let cfg: Config = serde_json::from_str(s).unwrap();
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn parses_direct_without_script_id() {
        // Direct mode: no relay config needed.
        let s = r#"{
            "mode": "direct"
        }"#;
        let cfg: Config = serde_json::from_str(s).unwrap();
        cfg.validate()
            .expect("direct must validate without account_groups");
        assert_eq!(cfg.mode_kind().unwrap(), Mode::Direct);
    }

    #[test]
    fn google_only_alias_parses_as_direct() {
        // Compatibility alias: old configs keep loading.
        let s = r#"{
            "mode": "google_only"
        }"#;
        let cfg: Config = serde_json::from_str(s).unwrap();
        cfg.validate()
            .expect("google_only alias must validate without account_groups");
        assert_eq!(cfg.mode_kind().unwrap(), Mode::Direct);
    }

    #[test]
    fn parses_full_mode() {
        let s = r#"{
            "mode": "full",
            "account_groups": [{
                "auth_key": "test-auth-key-please-change-32chars",
                "script_ids": "ABCDEF"
            }]
        }"#;
        let cfg: Config = serde_json::from_str(s).unwrap();
        cfg.validate().unwrap();
        assert_eq!(cfg.mode_kind().unwrap(), Mode::Full);
    }

    #[test]
    fn parses_vercel_edge_mode() {
        let s = r#"{
            "mode": "vercel_edge",
            "vercel": {
                "base_url": "https://example.vercel.app",
                "relay_path": "/api/api",
                "auth_key": "test-auth-key-please-change-32chars"
            }
        }"#;
        let cfg = Config::from_json_str(s).unwrap();
        assert_eq!(cfg.mode_kind().unwrap(), Mode::VercelEdge);
        assert_eq!(cfg.vercel.relay_path, "/api/api");
        assert!(cfg.vercel.verify_tls);
    }

    #[test]
    fn fronting_groups_parse_and_validate() {
        let s = r#"{
            "mode": "direct",
            "fronting_groups": [
                {
                    "name": "vercel",
                    "ip": "76.76.21.21",
                    "sni": "react.dev",
                    "domains": ["vercel.com", "nextjs.org"]
                }
            ]
        }"#;
        let cfg: Config = serde_json::from_str(s).unwrap();
        cfg.validate().unwrap();
        assert_eq!(cfg.fronting_groups.len(), 1);
        assert_eq!(cfg.fronting_groups[0].name, "vercel");
    }

    #[test]
    fn fronting_group_rejects_invalid_sni_at_validate() {
        let s = r#"{
            "mode": "direct",
            "fronting_groups": [{
                "name": "bad",
                "ip": "1.2.3.4",
                "sni": "not a valid hostname",
                "domains": ["x.com"]
            }]
        }"#;
        let cfg: Config = serde_json::from_str(s).unwrap();
        let err = cfg.validate().expect_err("invalid sni must fail validate");
        assert!(format!("{}", err).contains("invalid sni"));
    }

    #[test]
    fn full_mode_requires_account_groups() {
        let s = r#"{
            "mode": "full"
        }"#;
        let cfg: Config = serde_json::from_str(s).unwrap();
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn rejects_unknown_mode_value() {
        let s = r#"{
            "mode": "hybrid",
            "account_groups": [{
                "auth_key": "test-auth-key-please-change-32chars",
                "script_ids": "X"
            }]
        }"#;
        let cfg: Config = serde_json::from_str(s).unwrap();
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn rejects_zero_scan_batch_size() {
        let s = r#"{
            "mode": "apps_script",
            "account_groups": [{
                "auth_key": "test-auth-key-please-change-32chars",
                "script_ids": "X"
            }],
            "scan_batch_size": 0
        }"#;
        let cfg: Config = serde_json::from_str(s).unwrap();
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn rejects_same_http_and_socks5_port() {
        let s = r#"{
            "mode": "apps_script",
            "account_groups": [{
                "auth_key": "test-auth-key-please-change-32chars",
                "script_ids": "X"
            }],
            "listen_port": 8085,
            "socks5_port": 8085
        }"#;
        let cfg: Config = serde_json::from_str(s).unwrap();
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn missing_or_null_socks5_port_disables_socks5() {
        let base = r#"{
            "mode": "apps_script",
            "account_groups": [{
                "auth_key": "test-auth-key-please-change-32chars",
                "script_ids": "X"
            }],
            "listen_port": 9090
        }"#;
        let cfg = Config::from_json_str(base).unwrap();
        assert_eq!(cfg.socks5_port, None);

        let disabled = r#"{
            "mode": "apps_script",
            "account_groups": [{
                "auth_key": "test-auth-key-please-change-32chars",
                "script_ids": "X"
            }],
            "listen_port": 9090,
            "socks5_port": null
        }"#;
        let cfg = Config::from_json_str(disabled).unwrap();
        assert_eq!(cfg.socks5_port, None);
    }
}

#[cfg(test)]
mod rt_tests {
    use super::*;

    #[test]
    fn round_trip_all_current_fields() {
        // Regression guard: make sure a config written by the UI (all current
        // optional fields present and populated) loads back cleanly.
        let json = r#"{
  "mode": "apps_script",
  "config_version": 1,
  "google_ip": "216.239.38.120",
  "front_domain": "www.google.com",
  "account_groups": [{
    "label": "primary",
    "auth_key": "testtesttest-testtesttest",
    "script_ids": "AKfyc_TEST",
    "weight": 1,
    "enabled": true
  }],
  "listen_host": "127.0.0.1",
  "listen_port": 8085,
  "socks5_port": 8086,
  "log_level": "info",
  "verify_ssl": true,
  "upstream_socks5": "127.0.0.1:50529",
  "parallel_relay": 2,
  "range_parallelism": 12,
  "range_chunk_bytes": 262144,
  "relay_request_timeout_secs": 25,
  "request_timeout_secs": 45,
  "auto_blacklist_strikes": 5,
  "auto_blacklist_window_secs": 60,
  "auto_blacklist_cooldown_secs": 30,
  "sni_hosts": ["www.google.com", "drive.google.com"],
  "fetch_ips_from_api": true,
  "max_ips_to_scan": 50,
  "scan_batch_size": 100,
  "google_ip_validation": true,
  "outage_reset_enabled": true,
  "outage_reset_failure_threshold": 4,
  "outage_reset_window_ms": 6000,
  "outage_reset_cooldown_ms": 20000,
  "relay_rate_limit_qps": 2.5,
  "relay_rate_limit_burst": 5
}"#;
        let tmp = std::env::temp_dir().join("mhrv-rt-test.json");
        std::fs::write(&tmp, json).unwrap();
        let cfg = Config::load(&tmp).expect("config should load");
        assert_eq!(cfg.mode, "apps_script");
        assert_eq!(cfg.listen_port, 8085);
        assert_eq!(cfg.upstream_socks5.as_deref(), Some("127.0.0.1:50529"));
        assert_eq!(cfg.parallel_relay, 2);
        assert_eq!(
            cfg.sni_hosts.as_ref().unwrap(),
            &vec!["www.google.com".to_string(), "drive.google.com".to_string()]
        );
        assert!(cfg.fetch_ips_from_api);
        assert_eq!(cfg.range_parallelism, Some(12));
        assert_eq!(cfg.relay_request_timeout_secs, Some(25));
        assert_eq!(cfg.request_timeout_secs, Some(45));
        assert_eq!(cfg.auto_blacklist_strikes, Some(5));
        assert_eq!(cfg.auto_blacklist_window_secs, Some(60));
        assert_eq!(cfg.auto_blacklist_cooldown_secs, Some(30));
        assert_eq!(cfg.outage_reset_failure_threshold, Some(4));
        assert_eq!(cfg.relay_rate_limit_qps, Some(2.5));
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn round_trip_minimal_fields_only() {
        // User saves with defaults for everything optional. This is what the
        // UI's save button actually writes for a first-run user.
        let json = r#"{
  "mode": "apps_script",
  "google_ip": "216.239.38.120",
  "front_domain": "www.google.com",
  "account_groups": [{
    "auth_key": "test-auth-key-please-change-32chars",
    "script_ids": "A"
  }],
  "listen_host": "127.0.0.1",
  "listen_port": 8085,
  "log_level": "info",
  "verify_ssl": true
}"#;
        let tmp = std::env::temp_dir().join("mhrv-rt-min.json");
        std::fs::write(&tmp, json).unwrap();
        let cfg = Config::load(&tmp).expect("minimal config should load");
        assert_eq!(cfg.mode, "apps_script");
        let _ = std::fs::remove_file(&tmp);
    }
}
