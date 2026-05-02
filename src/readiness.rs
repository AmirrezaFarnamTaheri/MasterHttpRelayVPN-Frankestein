use crate::config::{Config, Mode};
use rustls::pki_types::ServerName;
use std::net::IpAddr;

pub type ReadinessId = &'static str;

pub const CONFIG_VERSION: ReadinessId = "config.version";
pub const CONFIG_MODE: ReadinessId = "config.mode";
pub const ACCOUNT_GROUPS_ENABLED: ReadinessId = "account_groups.enabled";
pub const ACCOUNT_GROUPS_SCRIPT_IDS: ReadinessId = "account_groups.script_ids";
pub const ACCOUNT_GROUPS_AUTH_KEY: ReadinessId = "account_groups.auth_key";
pub const VERCEL_BASE_URL: ReadinessId = "vercel.base_url";
pub const VERCEL_RELAY_PATH: ReadinessId = "vercel.relay_path";
pub const VERCEL_AUTH_KEY: ReadinessId = "vercel.auth_key";
pub const VERCEL_MAX_BODY_BYTES: ReadinessId = "vercel.max_body_bytes";
pub const DIRECT_GOOGLE_IP: ReadinessId = "direct.google_ip";
pub const DIRECT_FRONT_DOMAIN: ReadinessId = "direct.front_domain";
pub const LOCAL_LISTENER: ReadinessId = "local.listener";
pub const LOCAL_PORTS: ReadinessId = "local.ports";
pub const CA_TRUST: ReadinessId = "ca.trust";
pub const ANDROID_APP_CA_TRUST: ReadinessId = "ca.android_app_trust";
pub const LAN_EXPOSURE: ReadinessId = "lan.exposure";
pub const LAN_TOKEN: ReadinessId = "lan.token";
pub const LAN_ALLOWLIST: ReadinessId = "lan.allowlist";
pub const FULL_CODEFULL_DEPLOYMENT: ReadinessId = "full.codefull_deployment";
pub const FULL_TUNNEL_NODE_URL: ReadinessId = "full.tunnel_node_url";
pub const FULL_TUNNEL_AUTH: ReadinessId = "full.tunnel_auth";
pub const FULL_UDP_SUPPORT: ReadinessId = "full.udp_support";
pub const FULL_TUNNEL_HEALTH: ReadinessId = "full.tunnel_health";
pub const SCAN_BATCH_SIZE: ReadinessId = "scan.batch_size";
pub const RELAY_RATE_LIMIT_QPS: ReadinessId = "relay.rate_limit_qps";
pub const DOMAIN_OVERRIDES_HOST: ReadinessId = "domain_overrides.host";
pub const DOMAIN_OVERRIDES_FORCE_ROUTE: ReadinessId = "domain_overrides.force_route";
pub const FRONTING_GROUPS_NAME: ReadinessId = "fronting_groups.name";
pub const FRONTING_GROUPS_IP: ReadinessId = "fronting_groups.ip";
pub const FRONTING_GROUPS_SNI: ReadinessId = "fronting_groups.sni";
pub const FRONTING_GROUPS_DOMAINS: ReadinessId = "fronting_groups.domains";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReadinessRepair {
    pub label: &'static str,
    pub target: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReadinessSeverity {
    Blocker,
    Warning,
    Hint,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReadinessRule {
    pub id: ReadinessId,
    pub severity: ReadinessSeverity,
    pub applies_to: &'static str,
    pub ok_when: &'static str,
    pub not_ok_when: &'static str,
    pub repair_target: Option<&'static str>,
}

pub const READINESS_RULES: &[ReadinessRule] = &[
    ReadinessRule {
        id: CONFIG_VERSION,
        severity: ReadinessSeverity::Blocker,
        applies_to: "Config validation",
        ok_when: "config_version is supported by this binary",
        not_ok_when: "config_version is newer than the binary understands",
        repair_target: Some("config.version"),
    },
    ReadinessRule {
        id: CONFIG_MODE,
        severity: ReadinessSeverity::Blocker,
        applies_to: "All modes",
        ok_when: "mode is apps_script, vercel_edge, direct, or full",
        not_ok_when: "mode is missing, misspelled, or a removed legacy value",
        repair_target: Some("setup.mode"),
    },
    ReadinessRule {
        id: ACCOUNT_GROUPS_ENABLED,
        severity: ReadinessSeverity::Blocker,
        applies_to: "apps_script, full",
        ok_when: "at least one account group is enabled",
        not_ok_when: "all account groups are disabled or absent",
        repair_target: Some("setup.account_groups"),
    },
    ReadinessRule {
        id: ACCOUNT_GROUPS_SCRIPT_IDS,
        severity: ReadinessSeverity::Blocker,
        applies_to: "apps_script, full",
        ok_when: "enabled account groups contain at least one deployment ID",
        not_ok_when: "enabled account groups have no deployment IDs",
        repair_target: Some("setup.account_groups.script_ids"),
    },
    ReadinessRule {
        id: ACCOUNT_GROUPS_AUTH_KEY,
        severity: ReadinessSeverity::Blocker,
        applies_to: "apps_script, full",
        ok_when: "every enabled account group has an AUTH_KEY",
        not_ok_when: "an enabled account group has an empty AUTH_KEY",
        repair_target: Some("setup.account_groups.auth_key"),
    },
    ReadinessRule {
        id: VERCEL_BASE_URL,
        severity: ReadinessSeverity::Blocker,
        applies_to: "vercel_edge",
        ok_when: "vercel.base_url is an http(s) origin with a host",
        not_ok_when: "base URL is empty, malformed, or includes the relay path",
        repair_target: Some("setup.serverless.base_url"),
    },
    ReadinessRule {
        id: VERCEL_RELAY_PATH,
        severity: ReadinessSeverity::Blocker,
        applies_to: "vercel_edge",
        ok_when: "vercel.relay_path starts with /",
        not_ok_when: "relay path is empty or missing the leading /",
        repair_target: Some("setup.serverless.relay_path"),
    },
    ReadinessRule {
        id: VERCEL_AUTH_KEY,
        severity: ReadinessSeverity::Blocker,
        applies_to: "vercel_edge",
        ok_when: "vercel.auth_key is non-empty and not a placeholder",
        not_ok_when: "AUTH_KEY is empty or still uses a known placeholder",
        repair_target: Some("setup.serverless.auth_key"),
    },
    ReadinessRule {
        id: VERCEL_MAX_BODY_BYTES,
        severity: ReadinessSeverity::Blocker,
        applies_to: "vercel_edge",
        ok_when: "vercel.max_body_bytes is at least 1024",
        not_ok_when: "max_body_bytes is too small for a valid relay response",
        repair_target: Some("advanced.serverless.max_body_bytes"),
    },
    ReadinessRule {
        id: DIRECT_GOOGLE_IP,
        severity: ReadinessSeverity::Blocker,
        applies_to: "direct",
        ok_when: "google_ip is empty for auto-detect or parses as an IP",
        not_ok_when: "google_ip is non-empty and not an IP address",
        repair_target: Some("setup.direct.google_ip"),
    },
    ReadinessRule {
        id: DIRECT_FRONT_DOMAIN,
        severity: ReadinessSeverity::Blocker,
        applies_to: "direct",
        ok_when: "front_domain is empty or a valid hostname",
        not_ok_when: "front_domain is an IP address or invalid SNI name",
        repair_target: Some("setup.direct.front_domain"),
    },
    ReadinessRule {
        id: LOCAL_LISTENER,
        severity: ReadinessSeverity::Blocker,
        applies_to: "all modes",
        ok_when: "listen_host is non-empty and listen_port is valid",
        not_ok_when: "local HTTP listener host or port is unusable",
        repair_target: Some("setup.local_listener"),
    },
    ReadinessRule {
        id: LOCAL_PORTS,
        severity: ReadinessSeverity::Blocker,
        applies_to: "Config validation",
        ok_when: "HTTP and SOCKS5 listeners use different ports",
        not_ok_when: "listen_port and socks5_port collide",
        repair_target: Some("setup.local_listener.ports"),
    },
    ReadinessRule {
        id: CA_TRUST,
        severity: ReadinessSeverity::Warning,
        applies_to: "apps_script, vercel_edge, direct",
        ok_when: "generated local CA is installed and trusted by the client",
        not_ok_when: "HTTPS clients may reject locally generated MITM leaves",
        repair_target: Some("setup.ca_trust"),
    },
    ReadinessRule {
        id: ANDROID_APP_CA_TRUST,
        severity: ReadinessSeverity::Warning,
        applies_to: "Android apps_script, vercel_edge, direct",
        ok_when: "target Android app/browser trusts user-installed CAs",
        not_ok_when: "Android 7+ app ignores user CAs unless it opts in",
        repair_target: Some("help.android.ca_trust"),
    },
    ReadinessRule {
        id: LAN_EXPOSURE,
        severity: ReadinessSeverity::Warning,
        applies_to: "LAN-bound listeners",
        ok_when: "LAN exposure is intentional and firewall scope is understood",
        not_ok_when: "proxy is bound to 0.0.0.0 or :: unexpectedly",
        repair_target: Some("network.lan.listen_host"),
    },
    ReadinessRule {
        id: LAN_TOKEN,
        severity: ReadinessSeverity::Warning,
        applies_to: "LAN-bound HTTP/CONNECT",
        ok_when: "lan_token or lan_allowlist is configured",
        not_ok_when: "LAN clients can reach HTTP/CONNECT without an access guard",
        repair_target: Some("network.lan.access_control"),
    },
    ReadinessRule {
        id: LAN_ALLOWLIST,
        severity: ReadinessSeverity::Warning,
        applies_to: "LAN-bound SOCKS5",
        ok_when: "lan_allowlist contains at least one entry",
        not_ok_when: "SOCKS5 is exposed on LAN without token-capable access control",
        repair_target: Some("network.lan.allowlist"),
    },
    ReadinessRule {
        id: FULL_CODEFULL_DEPLOYMENT,
        severity: ReadinessSeverity::Warning,
        applies_to: "full",
        ok_when: "each Apps Script deployment uses CodeFull.gs",
        not_ok_when: "deployment may still run classic Code.gs",
        repair_target: Some("help.full.codefull"),
    },
    ReadinessRule {
        id: FULL_TUNNEL_NODE_URL,
        severity: ReadinessSeverity::Warning,
        applies_to: "full",
        ok_when: "CodeFull.gs TUNNEL_SERVER_URL points to the tunnel-node origin",
        not_ok_when: "CodeFull.gs points at the wrong origin or includes /tunnel",
        repair_target: Some("help.full.tunnel_node_url"),
    },
    ReadinessRule {
        id: FULL_TUNNEL_AUTH,
        severity: ReadinessSeverity::Warning,
        applies_to: "full",
        ok_when: "CodeFull.gs and tunnel-node share the same TUNNEL_AUTH_KEY",
        not_ok_when: "Apps Script and tunnel-node secrets differ",
        repair_target: Some("help.full.tunnel_auth"),
    },
    ReadinessRule {
        id: FULL_UDP_SUPPORT,
        severity: ReadinessSeverity::Warning,
        applies_to: "full",
        ok_when: "socks5_port is configured when app-level UDP is required",
        not_ok_when: "UDP-capable clients have no SOCKS5 UDP ASSOCIATE path",
        repair_target: Some("network.local_socks5"),
    },
    ReadinessRule {
        id: FULL_TUNNEL_HEALTH,
        severity: ReadinessSeverity::Warning,
        applies_to: "full",
        ok_when: "tunnel-node health/details passes and egress IP smoke test matches",
        not_ok_when: "tunnel-node is unverified, version-skewed, or egress differs",
        repair_target: Some("help.full.tunnel_health"),
    },
    ReadinessRule {
        id: SCAN_BATCH_SIZE,
        severity: ReadinessSeverity::Blocker,
        applies_to: "Config validation",
        ok_when: "scan_batch_size is at least 1",
        not_ok_when: "scan batch size is zero",
        repair_target: Some("advanced.scan.batch_size"),
    },
    ReadinessRule {
        id: RELAY_RATE_LIMIT_QPS,
        severity: ReadinessSeverity::Blocker,
        applies_to: "Config validation",
        ok_when: "relay_rate_limit_qps is unset or a positive finite number",
        not_ok_when: "relay_rate_limit_qps is zero, negative, NaN, or infinite",
        repair_target: Some("advanced.relay_rate_limit_qps"),
    },
    ReadinessRule {
        id: DOMAIN_OVERRIDES_HOST,
        severity: ReadinessSeverity::Blocker,
        applies_to: "Config validation",
        ok_when: "domain override host or suffix is non-empty",
        not_ok_when: "domain override has no host/suffix match",
        repair_target: Some("advanced.domain_overrides.host"),
    },
    ReadinessRule {
        id: DOMAIN_OVERRIDES_FORCE_ROUTE,
        severity: ReadinessSeverity::Blocker,
        applies_to: "Config validation",
        ok_when: "force_route is direct, sni_rewrite, relay, or full_tunnel",
        not_ok_when: "force_route uses an unknown route name",
        repair_target: Some("advanced.domain_overrides.force_route"),
    },
    ReadinessRule {
        id: FRONTING_GROUPS_NAME,
        severity: ReadinessSeverity::Blocker,
        applies_to: "Config validation",
        ok_when: "fronting group name is non-empty",
        not_ok_when: "fronting group name is blank",
        repair_target: Some("advanced.fronting_groups.name"),
    },
    ReadinessRule {
        id: FRONTING_GROUPS_IP,
        severity: ReadinessSeverity::Blocker,
        applies_to: "Config validation",
        ok_when: "fronting group IP parses as an IP address",
        not_ok_when: "fronting group IP is missing or malformed",
        repair_target: Some("advanced.fronting_groups.ip"),
    },
    ReadinessRule {
        id: FRONTING_GROUPS_SNI,
        severity: ReadinessSeverity::Blocker,
        applies_to: "Config validation",
        ok_when: "fronting group SNI is a valid hostname",
        not_ok_when: "fronting group SNI is missing, an IP address, or invalid",
        repair_target: Some("advanced.fronting_groups.sni"),
    },
    ReadinessRule {
        id: FRONTING_GROUPS_DOMAINS,
        severity: ReadinessSeverity::Blocker,
        applies_to: "Config validation",
        ok_when: "fronting group contains at least one routed domain",
        not_ok_when: "fronting group domain list is empty",
        repair_target: Some("advanced.fronting_groups.domains"),
    },
];

pub fn readiness_rules() -> &'static [ReadinessRule] {
    READINESS_RULES
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReadinessRepairAnchor {
    pub target: &'static str,
    pub desktop: &'static str,
    pub android: &'static str,
}

pub const READINESS_REPAIR_ANCHORS: &[ReadinessRepairAnchor] = &[
    ReadinessRepairAnchor {
        target: "config.version",
        desktop: "Config file -> config_version compatibility",
        android: "Imported config file -> config_version compatibility",
    },
    ReadinessRepairAnchor {
        target: "setup.mode",
        desktop: "Setup -> Mode",
        android: "Mode section -> Mode dropdown",
    },
    ReadinessRepairAnchor {
        target: "setup.account_groups",
        desktop: "Advanced -> Multi-account pools -> Add group",
        android: "Apps Script relay -> Deployment IDs",
    },
    ReadinessRepairAnchor {
        target: "setup.account_groups.script_ids",
        desktop: "Advanced -> Multi-account pools -> IDs",
        android: "Apps Script relay -> Deployment IDs",
    },
    ReadinessRepairAnchor {
        target: "setup.account_groups.auth_key",
        desktop: "Advanced -> Multi-account pools -> Auth key",
        android: "Apps Script relay -> AUTH_KEY",
    },
    ReadinessRepairAnchor {
        target: "setup.serverless.base_url",
        desktop: "Setup -> Serverless JSON relay -> Base URL",
        android: "Serverless JSON relay -> Base URL",
    },
    ReadinessRepairAnchor {
        target: "setup.serverless.relay_path",
        desktop: "Setup -> Serverless JSON relay -> Relay path",
        android: "Serverless JSON relay -> Relay path",
    },
    ReadinessRepairAnchor {
        target: "setup.serverless.auth_key",
        desktop: "Setup -> Serverless JSON relay -> Auth key",
        android: "Serverless JSON relay -> AUTH_KEY",
    },
    ReadinessRepairAnchor {
        target: "advanced.serverless.max_body_bytes",
        desktop: "Setup -> Serverless JSON relay -> Max body MB",
        android: "Advanced config import -> vercel.max_body_bytes",
    },
    ReadinessRepairAnchor {
        target: "setup.direct.google_ip",
        desktop: "Network -> Google IP",
        android: "Network -> Google edge IP",
    },
    ReadinessRepairAnchor {
        target: "setup.direct.front_domain",
        desktop: "Network -> Front domain",
        android: "Network -> Front SNI",
    },
    ReadinessRepairAnchor {
        target: "setup.local_listener",
        desktop: "Network -> Listen host / Ports",
        android: "Advanced -> LAN sharing plus built-in local listener defaults",
    },
    ReadinessRepairAnchor {
        target: "setup.local_listener.ports",
        desktop: "Network -> Ports",
        android: "Advanced config import -> listen_port / socks5_port",
    },
    ReadinessRepairAnchor {
        target: "setup.ca_trust",
        desktop: "Setup header -> Install CA or CLI --install-cert",
        android: "Install MITM certificate button",
    },
    ReadinessRepairAnchor {
        target: "help.android.ca_trust",
        desktop: "Help & docs -> Android CA trust",
        android: "Help -> Certificate section",
    },
    ReadinessRepairAnchor {
        target: "network.lan.listen_host",
        desktop: "Network -> Sharing and per-app routing -> Local only / Share on LAN",
        android: "Advanced -> LAN sharing",
    },
    ReadinessRepairAnchor {
        target: "network.lan.access_control",
        desktop: "Network -> Sharing and per-app routing -> LAN token / Allowed IPs",
        android: "Advanced -> LAN sharing -> LAN token / Allowed IPs",
    },
    ReadinessRepairAnchor {
        target: "network.lan.allowlist",
        desktop: "Network -> Sharing and per-app routing -> Allowed IPs",
        android: "Advanced -> LAN sharing -> Allowed IPs",
    },
    ReadinessRepairAnchor {
        target: "help.full.codefull",
        desktop: "Help & docs -> Full tunnel -> CodeFull.gs",
        android: "Help -> Full tunnel setup",
    },
    ReadinessRepairAnchor {
        target: "help.full.tunnel_node_url",
        desktop: "Help & docs -> Full tunnel -> TUNNEL_SERVER_URL",
        android: "Help -> Full tunnel setup -> tunnel-node URL",
    },
    ReadinessRepairAnchor {
        target: "help.full.tunnel_auth",
        desktop: "Help & docs -> Full tunnel -> TUNNEL_AUTH_KEY",
        android: "Help -> Full tunnel setup -> matching auth",
    },
    ReadinessRepairAnchor {
        target: "network.local_socks5",
        desktop: "Network -> Ports -> SOCKS5",
        android: "Advanced config import -> socks5_port",
    },
    ReadinessRepairAnchor {
        target: "help.full.tunnel_health",
        desktop: "Help & docs -> Full tunnel -> health/details and IP-check smoke test",
        android: "Help -> Full tunnel setup -> health and public-IP verification",
    },
    ReadinessRepairAnchor {
        target: "advanced.scan.batch_size",
        desktop: "Advanced -> Scan settings -> scan_batch_size",
        android: "Advanced config import -> scan_batch_size",
    },
    ReadinessRepairAnchor {
        target: "advanced.relay_rate_limit_qps",
        desktop: "Advanced -> Relay tuning -> relay_rate_limit_qps",
        android: "Advanced config import -> relay_rate_limit_qps",
    },
    ReadinessRepairAnchor {
        target: "advanced.domain_overrides.host",
        desktop: "Advanced -> Domain overrides -> host/suffix",
        android: "Advanced config import -> domain_overrides",
    },
    ReadinessRepairAnchor {
        target: "advanced.domain_overrides.force_route",
        desktop: "Advanced -> Domain overrides -> force_route",
        android: "Advanced config import -> domain_overrides",
    },
    ReadinessRepairAnchor {
        target: "advanced.fronting_groups.name",
        desktop: "Advanced -> Fronting groups -> name",
        android: "Advanced config import -> fronting_groups",
    },
    ReadinessRepairAnchor {
        target: "advanced.fronting_groups.ip",
        desktop: "Advanced -> Fronting groups -> IP",
        android: "Advanced config import -> fronting_groups",
    },
    ReadinessRepairAnchor {
        target: "advanced.fronting_groups.sni",
        desktop: "Advanced -> Fronting groups -> SNI",
        android: "Advanced config import -> fronting_groups",
    },
    ReadinessRepairAnchor {
        target: "advanced.fronting_groups.domains",
        desktop: "Advanced -> Fronting groups -> routed domains",
        android: "Advanced config import -> fronting_groups",
    },
];

pub fn repair_anchor_for_target(target: &str) -> Option<ReadinessRepairAnchor> {
    READINESS_REPAIR_ANCHORS
        .iter()
        .copied()
        .find(|anchor| anchor.target == target)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReadinessItem {
    pub id: ReadinessId,
    pub label: &'static str,
    pub ok: bool,
    pub severity: ReadinessSeverity,
    pub detail: String,
    pub repair: Option<ReadinessRepair>,
}

impl ReadinessItem {
    fn blocker(id: ReadinessId, label: &'static str, ok: bool, detail: impl Into<String>) -> Self {
        Self {
            id,
            label,
            ok,
            severity: ReadinessSeverity::Blocker,
            detail: detail.into(),
            repair: repair_for(id),
        }
    }

    fn warning(id: ReadinessId, label: &'static str, ok: bool, detail: impl Into<String>) -> Self {
        Self {
            id,
            label,
            ok,
            severity: ReadinessSeverity::Warning,
            detail: detail.into(),
            repair: repair_for(id),
        }
    }

    pub fn blocks_start(&self) -> bool {
        !self.ok && self.severity == ReadinessSeverity::Blocker
    }
}

pub fn validation_message(item: &ReadinessItem) -> String {
    let repair = item
        .repair
        .as_ref()
        .map(|repair| format!(" Next: {} [{}].", repair.label, repair.target))
        .unwrap_or_default();
    format!("{}: {}{}", item.id, item.detail, repair)
}

pub fn validation_failure(
    id: ReadinessId,
    label: &'static str,
    detail: impl Into<String>,
) -> String {
    validation_message(&ReadinessItem {
        id,
        label,
        ok: false,
        severity: ReadinessSeverity::Blocker,
        detail: detail.into(),
        repair: repair_for(id),
    })
}

pub fn first_blocker_id(items: &[ReadinessItem]) -> Option<ReadinessId> {
    items
        .iter()
        .find(|item| item.blocks_start())
        .map(|item| item.id)
}

pub fn first_blocker(items: &[ReadinessItem]) -> Option<&ReadinessItem> {
    items.iter().find(|item| item.blocks_start())
}

pub fn mode_readiness(cfg: &Config) -> Vec<ReadinessItem> {
    let mode = cfg.mode_kind().unwrap_or(Mode::AppsScript);
    let mut items = Vec::new();
    match mode {
        Mode::AppsScript | Mode::Full => {
            let groups = cfg.account_groups.as_deref().unwrap_or(&[]);
            let enabled: Vec<_> = groups.iter().filter(|group| group.enabled).collect();
            let deployment_count: usize = enabled
                .iter()
                .flat_map(|group| group.script_ids.clone().into_vec())
                .filter(|id| !id.trim().is_empty())
                .count();
            let auth_ready = !enabled.is_empty()
                && enabled
                    .iter()
                    .all(|group| !group.auth_key.trim().is_empty());

            items.push(ReadinessItem::blocker(
                ACCOUNT_GROUPS_ENABLED,
                "Enabled groups",
                !enabled.is_empty(),
                format!("{} enabled group(s)", enabled.len()),
            ));
            items.push(ReadinessItem::blocker(
                ACCOUNT_GROUPS_SCRIPT_IDS,
                "Deployment IDs",
                deployment_count > 0,
                if deployment_count > 0 {
                    format!("{deployment_count} deployment ID(s)")
                } else if enabled.is_empty() {
                    "Enable at least one Apps Script account group".into()
                } else {
                    "Add at least one deployment ID to an enabled account group".into()
                },
            ));
            items.push(ReadinessItem::blocker(
                ACCOUNT_GROUPS_AUTH_KEY,
                "Group AUTH_KEY",
                auth_ready,
                if enabled.is_empty() {
                    "Enable at least one Apps Script account group".into()
                } else {
                    format!(
                        "{}/{} enabled group(s) with AUTH_KEY",
                        enabled
                            .iter()
                            .filter(|group| !group.auth_key.trim().is_empty())
                            .count(),
                        enabled.len()
                    )
                },
            ));
        }
        Mode::VercelEdge => {
            let base = cfg.vercel.base_url.trim();
            let base_ready = parse_relay_origin(base).is_ok();
            items.push(ReadinessItem::blocker(
                VERCEL_BASE_URL,
                "Relay origin",
                base_ready,
                if base_ready {
                    base.to_string()
                } else if base.is_empty() {
                    "Paste https://your-project.vercel.app or Netlify site URL".into()
                } else {
                    "Use an http(s) URL with a hostname and no relay path".into()
                },
            ));
            let relay_path = cfg.vercel.relay_path.trim();
            items.push(ReadinessItem::blocker(
                VERCEL_RELAY_PATH,
                "Relay path",
                relay_path.starts_with('/'),
                if relay_path.starts_with('/') {
                    relay_path.to_string()
                } else {
                    "Path must start with /, usually /api/api".into()
                },
            ));
            let auth = cfg.vercel.auth_key.trim();
            let auth_ready = !auth.is_empty()
                && !auth.eq_ignore_ascii_case("change-me")
                && !auth.eq_ignore_ascii_case("your_auth_key")
                && !auth.eq_ignore_ascii_case("your-auth-key")
                && !auth.eq_ignore_ascii_case("same_value_as_vercel_auth_key");
            items.push(ReadinessItem::blocker(
                VERCEL_AUTH_KEY,
                "AUTH_KEY",
                auth_ready,
                if auth_ready {
                    "Configured".to_string()
                } else {
                    "Set a non-placeholder AUTH_KEY matching the serverless deployment".into()
                },
            ));
            items.push(ReadinessItem::blocker(
                VERCEL_MAX_BODY_BYTES,
                "Max body",
                cfg.vercel.max_body_bytes >= 1024,
                if cfg.vercel.max_body_bytes >= 1024 {
                    format!("{} bytes", cfg.vercel.max_body_bytes)
                } else {
                    "Set max_body_bytes to at least 1024".into()
                },
            ));
        }
        Mode::Direct => {
            let google_ip = cfg.google_ip.trim();
            items.push(ReadinessItem::blocker(
                DIRECT_GOOGLE_IP,
                "Google edge IP",
                google_ip.is_empty() || google_ip.parse::<IpAddr>().is_ok(),
                if google_ip.is_empty() {
                    "Auto-detected on connect when possible".into()
                } else {
                    google_ip.to_string()
                },
            ));
            let front_domain = cfg.front_domain.trim();
            let front_domain_ready = front_domain.is_empty()
                || (front_domain.parse::<IpAddr>().is_err()
                    && ServerName::try_from(front_domain.to_string()).is_ok());
            items.push(ReadinessItem::blocker(
                DIRECT_FRONT_DOMAIN,
                "Front SNI",
                front_domain_ready,
                if front_domain.is_empty() {
                    "Defaults to www.google.com on connect".into()
                } else if front_domain.parse::<IpAddr>().is_ok() {
                    "Use a hostname for SNI, not an IP address".into()
                } else {
                    front_domain.to_string()
                },
            ));
        }
    }

    items.push(ReadinessItem::blocker(
        LOCAL_LISTENER,
        "Local listener",
        !cfg.listen_host.trim().is_empty() && cfg.listen_port > 0,
        format!("{}:{}", cfg.listen_host.trim(), cfg.listen_port),
    ));
    add_operational_readiness(cfg, mode, &mut items);
    items
}

fn add_operational_readiness(cfg: &Config, mode: Mode, items: &mut Vec<ReadinessItem>) {
    if mode == Mode::Full {
        items.push(ReadinessItem::warning(
            FULL_CODEFULL_DEPLOYMENT,
            "CodeFull deployment",
            false,
            "Verify each full-mode Apps Script deployment uses CodeFull.gs, not the classic Code.gs relay",
        ));
        items.push(ReadinessItem::warning(
            FULL_TUNNEL_NODE_URL,
            "Tunnel-node URL",
            false,
            "Verify CodeFull.gs TUNNEL_SERVER_URL points at the public tunnel-node origin",
        ));
        items.push(ReadinessItem::warning(
            FULL_TUNNEL_AUTH,
            "Tunnel auth",
            false,
            "Verify CodeFull.gs TUNNEL_AUTH_KEY matches the tunnel-node TUNNEL_AUTH_KEY environment variable",
        ));
        items.push(ReadinessItem::warning(
            FULL_UDP_SUPPORT,
            "UDP/SOCKS5 path",
            cfg.socks5_port.is_some(),
            if cfg.socks5_port.is_some() {
                String::from("SOCKS5 listener configured for clients that use UDP ASSOCIATE")
            } else {
                String::from("Set socks5_port if you expect app-level UDP through full mode")
            },
        ));
        items.push(ReadinessItem::warning(
            FULL_TUNNEL_HEALTH,
            "Tunnel health",
            false,
            "Verify /healthz on tunnel-node, then start full mode and confirm the public IP matches the VPS",
        ));
    }

    if requires_local_ca(mode) {
        items.push(ReadinessItem::warning(
            CA_TRUST,
            "Local CA trust",
            false,
            "Install and trust the generated CA before routing HTTPS clients through the local proxy",
        ));
        items.push(ReadinessItem::warning(
            ANDROID_APP_CA_TRUST,
            "Android app CA trust",
            false,
            "On Android 7+, many apps ignore user-installed CAs unless the app opts in; browsers and apps vary",
        ));
    }

    let host = cfg.listen_host.trim();
    if is_lan_bound(host) {
        let has_token = cfg
            .lan_token
            .as_deref()
            .map(|token| !token.trim().is_empty())
            .unwrap_or(false);
        let allowlist_count = cfg
            .lan_allowlist
            .as_ref()
            .map(|allowlist| {
                allowlist
                    .iter()
                    .filter(|entry| !entry.trim().is_empty())
                    .count()
            })
            .unwrap_or(0);
        let has_allowlist = allowlist_count > 0;

        items.push(ReadinessItem::warning(
            LAN_EXPOSURE,
            "LAN exposure",
            false,
            format!(
                "Proxy is bound to {host}; local-network devices can reach it when firewall rules allow"
            ),
        ));
        items.push(ReadinessItem::warning(
            LAN_TOKEN,
            "LAN token",
            has_token || has_allowlist,
            if has_token {
                "HTTP/CONNECT token configured".into()
            } else if has_allowlist {
                format!("{allowlist_count} allowlist entrie(s) configured")
            } else {
                "Set lan_token for HTTP/CONNECT or lan_allowlist before sharing on LAN".into()
            },
        ));
        if cfg.socks5_port.is_some() {
            items.push(ReadinessItem::warning(
                LAN_ALLOWLIST,
                "SOCKS5 LAN allowlist",
                has_allowlist,
                if has_allowlist {
                    format!("{allowlist_count} allowlist entrie(s) configured")
                } else {
                    "SOCKS5 has no token header; add lan_allowlist before exposing SOCKS5 on LAN"
                        .into()
                },
            ));
        }
    }
}

fn requires_local_ca(mode: Mode) -> bool {
    matches!(mode, Mode::AppsScript | Mode::VercelEdge | Mode::Direct)
}

fn is_lan_bound(host: &str) -> bool {
    matches!(host, "0.0.0.0" | "::")
}

fn parse_relay_origin(base: &str) -> Result<(), ()> {
    let parsed = url::Url::parse(base).map_err(|_| ())?;
    if !matches!(parsed.scheme(), "https" | "http") {
        return Err(());
    }
    if parsed.host_str().unwrap_or("").trim().is_empty() {
        return Err(());
    }
    Ok(())
}

fn repair_for(id: ReadinessId) -> Option<ReadinessRepair> {
    match id {
        CONFIG_VERSION => Some(ReadinessRepair {
            label: "Update this binary or lower config_version after confirming compatibility",
            target: "config.version",
        }),
        CONFIG_MODE => Some(ReadinessRepair {
            label: "Choose apps_script, vercel_edge, direct, or full",
            target: "setup.mode",
        }),
        ACCOUNT_GROUPS_ENABLED => Some(ReadinessRepair {
            label: "Enable or add an Apps Script account group",
            target: "setup.account_groups",
        }),
        ACCOUNT_GROUPS_SCRIPT_IDS => Some(ReadinessRepair {
            label: "Paste at least one Apps Script deployment ID",
            target: "setup.account_groups.script_ids",
        }),
        ACCOUNT_GROUPS_AUTH_KEY => Some(ReadinessRepair {
            label: "Set the AUTH_KEY for every enabled account group",
            target: "setup.account_groups.auth_key",
        }),
        VERCEL_BASE_URL => Some(ReadinessRepair {
            label: "Paste the Vercel or Netlify site origin",
            target: "setup.serverless.base_url",
        }),
        VERCEL_RELAY_PATH => Some(ReadinessRepair {
            label: "Set the relay path, usually /api/api",
            target: "setup.serverless.relay_path",
        }),
        VERCEL_AUTH_KEY => Some(ReadinessRepair {
            label: "Set the serverless AUTH_KEY",
            target: "setup.serverless.auth_key",
        }),
        VERCEL_MAX_BODY_BYTES => Some(ReadinessRepair {
            label: "Raise max_body_bytes to at least 1024",
            target: "advanced.serverless.max_body_bytes",
        }),
        DIRECT_GOOGLE_IP => Some(ReadinessRepair {
            label: "Clear the field for auto-detect or enter a valid IP address",
            target: "setup.direct.google_ip",
        }),
        DIRECT_FRONT_DOMAIN => Some(ReadinessRepair {
            label: "Use a hostname for SNI, not an IP address",
            target: "setup.direct.front_domain",
        }),
        LOCAL_LISTENER => Some(ReadinessRepair {
            label: "Set a non-empty listen host and valid HTTP port",
            target: "setup.local_listener",
        }),
        LOCAL_PORTS => Some(ReadinessRepair {
            label: "Use different HTTP and SOCKS5 ports",
            target: "setup.local_listener.ports",
        }),
        CA_TRUST => Some(ReadinessRepair {
            label: "Install and trust the generated local CA",
            target: "setup.ca_trust",
        }),
        ANDROID_APP_CA_TRUST => Some(ReadinessRepair {
            label: "Check whether the target Android app trusts user CAs",
            target: "help.android.ca_trust",
        }),
        LAN_EXPOSURE => Some(ReadinessRepair {
            label: "Bind to 127.0.0.1 unless you intentionally share on LAN",
            target: "network.lan.listen_host",
        }),
        LAN_TOKEN => Some(ReadinessRepair {
            label: "Set lan_token or lan_allowlist for LAN sharing",
            target: "network.lan.access_control",
        }),
        LAN_ALLOWLIST => Some(ReadinessRepair {
            label: "Set lan_allowlist before exposing SOCKS5 on LAN",
            target: "network.lan.allowlist",
        }),
        FULL_CODEFULL_DEPLOYMENT => Some(ReadinessRepair {
            label: "Deploy CodeFull.gs for every full-mode Apps Script group",
            target: "help.full.codefull",
        }),
        FULL_TUNNEL_NODE_URL => Some(ReadinessRepair {
            label: "Set CodeFull.gs TUNNEL_SERVER_URL to the tunnel-node origin",
            target: "help.full.tunnel_node_url",
        }),
        FULL_TUNNEL_AUTH => Some(ReadinessRepair {
            label: "Match CodeFull.gs TUNNEL_AUTH_KEY with tunnel-node TUNNEL_AUTH_KEY",
            target: "help.full.tunnel_auth",
        }),
        FULL_UDP_SUPPORT => Some(ReadinessRepair {
            label: "Configure a SOCKS5 port when full-mode UDP is required",
            target: "network.local_socks5",
        }),
        FULL_TUNNEL_HEALTH => Some(ReadinessRepair {
            label: "Check tunnel-node /healthz, logs, and public-IP verification",
            target: "help.full.tunnel_health",
        }),
        SCAN_BATCH_SIZE => Some(ReadinessRepair {
            label: "Set scan_batch_size to at least 1",
            target: "advanced.scan.batch_size",
        }),
        RELAY_RATE_LIMIT_QPS => Some(ReadinessRepair {
            label: "Set relay_rate_limit_qps to a positive finite number or clear it",
            target: "advanced.relay_rate_limit_qps",
        }),
        DOMAIN_OVERRIDES_HOST => Some(ReadinessRepair {
            label: "Set a hostname or suffix match for the domain override",
            target: "advanced.domain_overrides.host",
        }),
        DOMAIN_OVERRIDES_FORCE_ROUTE => Some(ReadinessRepair {
            label: "Use direct, sni_rewrite, relay, or full_tunnel",
            target: "advanced.domain_overrides.force_route",
        }),
        FRONTING_GROUPS_NAME => Some(ReadinessRepair {
            label: "Name the fronting group",
            target: "advanced.fronting_groups.name",
        }),
        FRONTING_GROUPS_IP => Some(ReadinessRepair {
            label: "Set the fronting edge IP",
            target: "advanced.fronting_groups.ip",
        }),
        FRONTING_GROUPS_SNI => Some(ReadinessRepair {
            label: "Set a valid hostname for the fronting SNI",
            target: "advanced.fronting_groups.sni",
        }),
        FRONTING_GROUPS_DOMAINS => Some(ReadinessRepair {
            label: "Add at least one non-empty routed domain",
            target: "advanced.fronting_groups.domains",
        }),
        _ => None,
    }
}

pub fn repair_for_id(id: ReadinessId) -> Option<ReadinessRepair> {
    repair_for(id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn readiness_ids(cfg: &Config) -> Vec<ReadinessId> {
        mode_readiness(cfg)
            .into_iter()
            .map(|item| item.id)
            .collect()
    }

    #[test]
    fn apps_script_readiness_uses_stable_ids() {
        let cfg = Config::from_json_str(
            r#"{
                "mode": "apps_script",
                "account_groups": [{
                    "auth_key": "test-auth-key-please-change-32chars",
                    "script_ids": ["AKfycb_primary"],
                    "enabled": true
                }]
            }"#,
        )
        .expect("config should load");

        let items = mode_readiness(&cfg);
        assert_eq!(
            readiness_ids(&cfg),
            vec![
                ACCOUNT_GROUPS_ENABLED,
                ACCOUNT_GROUPS_SCRIPT_IDS,
                ACCOUNT_GROUPS_AUTH_KEY,
                LOCAL_LISTENER,
                CA_TRUST,
                ANDROID_APP_CA_TRUST
            ]
        );
        assert!(first_blocker_id(&items).is_none());
    }

    #[test]
    fn serverless_readiness_reports_first_blocker_id() {
        let cfg = Config::from_json_str(
            r#"{
                "mode": "direct",
                "vercel": {
                    "base_url": "",
                    "relay_path": "api/api",
                    "auth_key": "change-me",
                    "max_body_bytes": 1
                }
            }"#,
        )
        .expect("direct config should load with inactive serverless fields");
        let mut serverless = cfg.clone();
        serverless.mode = "vercel_edge".into();

        let items = mode_readiness(&serverless);
        assert_eq!(first_blocker_id(&items), Some(VERCEL_BASE_URL));
        assert!(items
            .iter()
            .any(|item| item.id == VERCEL_RELAY_PATH && !item.ok));
        assert!(items
            .iter()
            .any(|item| item.id == VERCEL_AUTH_KEY && !item.ok));
        assert!(items
            .iter()
            .any(|item| item.id == VERCEL_MAX_BODY_BYTES && !item.ok));
    }

    #[test]
    fn direct_readiness_allows_auto_defaults_but_rejects_ip_sni() {
        let cfg = Config::from_json_str(r#"{"mode": "direct"}"#)
            .expect("direct config should load with defaults");
        assert!(mode_readiness(&cfg).iter().all(|item| !item.blocks_start()));

        let cfg: Config = serde_json::from_str(
            r#"{
                "mode": "direct",
                "google_ip": "not-an-ip",
                "front_domain": "8.8.8.8"
            }"#,
        )
        .expect("raw config should deserialize before readiness validation");
        let items = mode_readiness(&cfg);
        assert_eq!(first_blocker_id(&items), Some(DIRECT_GOOGLE_IP));
        assert!(items
            .iter()
            .any(|item| item.id == DIRECT_FRONT_DOMAIN && !item.ok));
    }

    #[test]
    fn readiness_blockers_include_repair_metadata() {
        let cfg: Config = serde_json::from_str(r#"{"mode": "apps_script"}"#)
            .expect("raw config should deserialize");
        let items = mode_readiness(&cfg);
        let blocker = first_blocker(&items).expect("missing account group should block");

        assert_eq!(blocker.id, ACCOUNT_GROUPS_ENABLED);
        assert_eq!(
            blocker.repair.as_ref().map(|repair| repair.target),
            Some("setup.account_groups")
        );
        assert!(validation_message(blocker).contains("account_groups.enabled"));
        assert!(validation_message(blocker).contains("Next:"));
    }

    #[test]
    fn readiness_adds_ca_warning_for_local_mitm_modes_only() {
        let cfg = Config::from_json_str(
            r#"{
                "mode": "vercel_edge",
                "vercel": {
                    "base_url": "https://relay.example",
                    "relay_path": "/api/api",
                    "auth_key": "long-random-serverless-key"
                }
            }"#,
        )
        .expect("serverless config should load");
        let items = mode_readiness(&cfg);
        let ca = items
            .iter()
            .find(|item| item.id == CA_TRUST)
            .expect("local MITM mode should include CA warning");
        assert_eq!(ca.severity, ReadinessSeverity::Warning);
        assert!(!ca.blocks_start());

        let full = Config::from_json_str(
            r#"{
                "mode": "full",
                "account_groups": [{
                    "auth_key": "test-auth-key-please-change-32chars",
                    "script_ids": ["AKfycb_primary"],
                    "enabled": true
                }]
            }"#,
        )
        .expect("full config should load");
        assert!(!mode_readiness(&full).iter().any(|item| item.id == CA_TRUST));
    }

    #[test]
    fn readiness_reports_lan_exposure_controls_without_blocking_start() {
        let cfg = Config::from_json_str(
            r#"{
                "mode": "direct",
                "listen_host": "0.0.0.0",
                "socks5_port": 8086
            }"#,
        )
        .expect("lan-exposed direct config should load with warnings");
        let items = mode_readiness(&cfg);

        assert!(items.iter().any(|item| item.id == LAN_EXPOSURE && !item.ok));
        assert!(items.iter().any(|item| item.id == LAN_TOKEN && !item.ok));
        assert!(items
            .iter()
            .any(|item| item.id == LAN_ALLOWLIST && !item.ok));
        assert_eq!(first_blocker_id(&items), None);
        assert_eq!(
            repair_for_id(LAN_ALLOWLIST).map(|repair| repair.target),
            Some("network.lan.allowlist")
        );
    }

    #[test]
    fn readiness_adds_full_mode_external_tunnel_checks() {
        let cfg = Config::from_json_str(
            r#"{
                "mode": "full",
                "socks5_port": 8086,
                "account_groups": [{
                    "auth_key": "test-auth-key-please-change-32chars",
                    "script_ids": ["AKfycb_full"],
                    "enabled": true
                }]
            }"#,
        )
        .expect("full config should load");
        let items = mode_readiness(&cfg);

        for id in [
            FULL_CODEFULL_DEPLOYMENT,
            FULL_TUNNEL_NODE_URL,
            FULL_TUNNEL_AUTH,
            FULL_UDP_SUPPORT,
            FULL_TUNNEL_HEALTH,
        ] {
            let item = items
                .iter()
                .find(|item| item.id == id)
                .expect("full-mode readiness row should exist");
            assert_eq!(item.severity, ReadinessSeverity::Warning);
            assert!(!item.blocks_start());
            assert!(repair_for_id(id).is_some());
        }

        assert!(items
            .iter()
            .any(|item| item.id == FULL_UDP_SUPPORT && item.ok));
        assert_eq!(first_blocker_id(&items), None);
        assert!(!items.iter().any(|item| item.id == CA_TRUST));
    }

    #[test]
    fn readiness_rule_catalog_is_complete_and_matches_repairs() {
        let mut seen = HashSet::new();
        for rule in readiness_rules() {
            assert!(seen.insert(rule.id), "duplicate readiness rule {}", rule.id);
            assert!(!rule.applies_to.trim().is_empty(), "missing applies_to");
            assert!(!rule.ok_when.trim().is_empty(), "missing ok_when");
            assert!(!rule.not_ok_when.trim().is_empty(), "missing not_ok_when");
            assert_eq!(
                repair_for_id(rule.id).map(|repair| repair.target),
                rule.repair_target,
                "repair target drift for {}",
                rule.id
            );
        }

        let all_ids = [
            CONFIG_VERSION,
            CONFIG_MODE,
            ACCOUNT_GROUPS_ENABLED,
            ACCOUNT_GROUPS_SCRIPT_IDS,
            ACCOUNT_GROUPS_AUTH_KEY,
            VERCEL_BASE_URL,
            VERCEL_RELAY_PATH,
            VERCEL_AUTH_KEY,
            VERCEL_MAX_BODY_BYTES,
            DIRECT_GOOGLE_IP,
            DIRECT_FRONT_DOMAIN,
            LOCAL_LISTENER,
            LOCAL_PORTS,
            CA_TRUST,
            ANDROID_APP_CA_TRUST,
            LAN_EXPOSURE,
            LAN_TOKEN,
            LAN_ALLOWLIST,
            FULL_CODEFULL_DEPLOYMENT,
            FULL_TUNNEL_NODE_URL,
            FULL_TUNNEL_AUTH,
            FULL_UDP_SUPPORT,
            FULL_TUNNEL_HEALTH,
            SCAN_BATCH_SIZE,
            RELAY_RATE_LIMIT_QPS,
            DOMAIN_OVERRIDES_HOST,
            DOMAIN_OVERRIDES_FORCE_ROUTE,
            FRONTING_GROUPS_NAME,
            FRONTING_GROUPS_IP,
            FRONTING_GROUPS_SNI,
            FRONTING_GROUPS_DOMAINS,
        ];
        assert_eq!(readiness_rules().len(), all_ids.len());
        for id in all_ids {
            assert!(seen.contains(id), "missing readiness rule for {id}");
        }

        let mut anchor_targets = HashSet::new();
        for anchor in READINESS_REPAIR_ANCHORS {
            assert!(
                anchor_targets.insert(anchor.target),
                "duplicate repair anchor {}",
                anchor.target
            );
            assert!(
                !anchor.desktop.trim().is_empty(),
                "missing desktop anchor for {}",
                anchor.target
            );
            assert!(
                !anchor.android.trim().is_empty(),
                "missing android anchor for {}",
                anchor.target
            );
        }
        for id in all_ids {
            if let Some(repair) = repair_for_id(id) {
                assert!(
                    anchor_targets.contains(repair.target),
                    "missing repair anchor for target {}",
                    repair.target
                );
            }
        }
    }
}
