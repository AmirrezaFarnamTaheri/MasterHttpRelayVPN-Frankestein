use serde::{Deserialize, Serialize};
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
/// `GoogleOnly` is a bootstrap: no relay, no Apps Script config needed,
/// only the SNI-rewrite tunnel to the Google edge is active. Intended for
/// users who need to reach `script.google.com` to deploy `Code.gs` in the
/// first place.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    AppsScript,
    GoogleOnly,
    Full,
}

impl Mode {
    pub fn as_str(self) -> &'static str {
        match self {
            Mode::AppsScript => "apps_script",
            Mode::GoogleOnly => "google_only",
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
        match self {
            ScriptId::One(s) => vec![s],
            ScriptId::Many(v) => v,
        }
    }
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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    /// Schema version for forward-compatible config migrations.
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
    /// Optional explicit SNI rotation pool for outbound TLS to `google_ip`.
    /// Empty / missing = auto-expand from `front_domain` (current default of
    /// {www, mail, drive, docs, calendar}.google.com). Set to an explicit list
    /// to pick exactly which SNI names get rotated through — useful when one
    /// of the defaults is locally blocked (e.g. mail.google.com in Iran at
    /// various times). Can be tested per-name via the UI or `mhrv-f test-sni`.
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
    /// When true, GET requests to `x.com/i/api/graphql/<hash>/<op>?variables=…`
    /// have their query trimmed to just the `variables=` param before being
    /// relayed. The `features` / `fieldToggles` params that X ships with
    /// these requests change frequently and bust the response cache —
    /// stripping them dramatically improves hit rate on Twitter/X browsing.
    ///
    /// Credit: idea from seramo_ir, originally adapted to the Python
    /// MasterHttpRelayVPN by the Persian community
    /// (https://gist.github.com/seramo/0ae9e5d30ac23a73d5eb3bd2710fcd67).
    ///
    /// Off by default — some X endpoints may reject calls that omit
    /// features. Turn on and observe.
    #[serde(default)]
    pub normalize_x_graphql: bool,

    /// Route YouTube traffic through the Apps Script relay instead of
    /// the direct SNI-rewrite tunnel. Ported from upstream Python
    /// `youtube_via_relay`.
    ///
    /// Why this exists: when YouTube is SNI-rewritten to `google_ip`
    /// with `SNI=www.google.com`, Google's frontend can enforce
    /// SafeSearch / Restricted Mode based on the SNI → some videos show
    /// as "restricted." Routing through Apps Script bypasses that check
    /// (it hits YouTube from Google's own backend, not via www.google.com
    /// SNI) but introduces the UrlFetchApp User-Agent and quota costs.
    ///
    /// Trade-off: enabling removes SafeSearch-on-SNI, adds `User-Agent:
    /// Google-Apps-Script` header and counts YouTube traffic against
    /// your Apps Script quota. Off by default.
    #[serde(default)]
    pub youtube_via_relay: bool,

    /// User-configurable passthrough list. Any host whose name matches
    /// one of these entries bypasses the Apps Script relay entirely and
    /// is plain-TCP-passthroughed (optionally through `upstream_socks5`).
    ///
    /// Accepts exact hostnames ("example.com") and leading-dot suffixes
    /// (".internal.example" matches "a.b.internal.example"). Matches are
    /// case-insensitive.
    ///
    /// Dispatched BEFORE SNI-rewrite and Apps Script, so a passthrough
    /// entry wins over the default Google-edge routing. Useful for
    /// sites where you already have reachability without the relay
    /// (saving Apps Script quota) or for hosts that break under MITM.
    #[serde(default)]
    pub passthrough_hosts: Vec<String>,

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
    /// In `google_only` bootstrap mode this may be omitted.
    #[serde(default)]
    pub account_groups: Option<Vec<AccountGroup>>,

    /// Optional access token required when binding to LAN (listen_host=0.0.0.0/::).
    /// When set, the HTTP proxy requires `X-MHRV-F-Token: <token>` on plain HTTP
    /// requests and on CONNECT preface. SOCKS5 has no header preface, so protect
    /// LAN-exposed SOCKS5 with `lan_allowlist` instead. This is a basic
    /// guardrail, not a full authentication system.
    #[serde(default)]
    pub lan_token: Option<String>,

    /// Optional allowlist of client IPs/CIDRs allowed to connect when LAN-bound.
    /// Minimal implementation: exact IP match only (CIDR parsing may be added later).
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
        let mut cfg: Config = serde_json::from_str(data)?;
        cfg.migrate_in_place()?;
        cfg.validate()?;
        Ok(cfg)
    }

    pub fn load(path: &Path) -> Result<Self, ConfigError> {
        let data = std::fs::read_to_string(path)
            .map_err(|e| ConfigError::Read(path.display().to_string(), e))?;
        Self::from_json_str(&data)
    }

    fn migrate_in_place(&mut self) -> Result<(), ConfigError> {
        // Fail closed on future versions so old binaries don't silently
        // misinterpret new schema.
        if self.config_version > CURRENT_CONFIG_VERSION {
            return Err(ConfigError::Invalid(format!(
                "config_version {} is newer than this binary supports (max {}). Please update mhrv-f.",
                self.config_version, CURRENT_CONFIG_VERSION
            )));
        }
        // v0/v1 (implicit) -> v1: nothing to rewrite yet; keep hook for future.
        if self.config_version == 0 {
            self.config_version = 1;
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
        if self.scan_batch_size == 0 {
            return Err(ConfigError::Invalid(
                "scan_batch_size must be greater than 0".into(),
            ));
        }
        if self.socks5_port == Some(self.listen_port) {
            return Err(ConfigError::Invalid(
                "listen_port and socks5_port must be different".into(),
            ));
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
        Ok(())
    }

    pub fn mode_kind(&self) -> Result<Mode, ConfigError> {
        match self.mode.as_str() {
            "apps_script" => Ok(Mode::AppsScript),
            "google_only" => Ok(Mode::GoogleOnly),
            "full" => Ok(Mode::Full),
            other => Err(ConfigError::Invalid(format!(
                "unknown mode '{}' (expected 'apps_script', 'google_only', or 'full')",
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
                "auth_key": "supersecretkey-123456",
                "script_ids": ["A", "B", "C"],
                "weight": 1,
                "enabled": true
            }]
        }"#;
        let cfg: Config = serde_json::from_str(s).unwrap();
        cfg.validate().unwrap();
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
    fn parses_google_only_without_script_id() {
        // Bootstrap mode: no relay config needed.
        let s = r#"{
            "mode": "google_only"
        }"#;
        let cfg: Config = serde_json::from_str(s).unwrap();
        cfg.validate()
            .expect("google_only must validate without account_groups");
        assert_eq!(cfg.mode_kind().unwrap(), Mode::GoogleOnly);
    }

    #[test]
    fn parses_full_mode() {
        let s = r#"{
            "mode": "full",
            "account_groups": [{
                "auth_key": "supersecretkey-123456",
                "script_ids": "ABCDEF"
            }]
        }"#;
        let cfg: Config = serde_json::from_str(s).unwrap();
        cfg.validate().unwrap();
        assert_eq!(cfg.mode_kind().unwrap(), Mode::Full);
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
                "auth_key": "supersecretkey-123456",
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
                "auth_key": "supersecretkey-123456",
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
                "auth_key": "supersecretkey-123456",
                "script_ids": "X"
            }],
            "listen_port": 8085,
            "socks5_port": 8085
        }"#;
        let cfg: Config = serde_json::from_str(s).unwrap();
        assert!(cfg.validate().is_err());
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
  "sni_hosts": ["www.google.com", "drive.google.com"],
  "fetch_ips_from_api": true,
  "max_ips_to_scan": 50,
  "scan_batch_size": 100,
  "google_ip_validation": true
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
    "auth_key": "secretkey123-secretkey123",
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
