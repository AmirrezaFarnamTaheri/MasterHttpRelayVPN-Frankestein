use std::collections::{HashMap, VecDeque};
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream, UdpSocket};
use tokio::sync::{mpsc, Mutex, Semaphore};
use tokio::task::JoinSet;
use tokio_rustls::rustls::client::danger::{
    HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier,
};
use tokio_rustls::rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use tokio_rustls::rustls::server::Acceptor;
use tokio_rustls::rustls::{ClientConfig, DigitallySignedStruct, SignatureScheme};
use tokio_rustls::{LazyConfigAcceptor, TlsAcceptor, TlsConnector};

use crate::config::{Config, DomainOverride, FrontingGroup, Mode};
use crate::domain_fronter::DomainFronter;
use crate::mitm::MitmCertManager;
use crate::policy::{decide_route, RouteDecision};
use crate::relay_transport::RelayTransport;
use crate::tunnel_client::{decode_udp_packets, TunnelMux};

const MAX_HTTP_HEADER_BYTES: usize = 64 * 1024;
const MAX_REQUEST_BODY_BYTES: usize = 100 * 1024 * 1024;

// Domains that are served from Google's core frontend IP pool and therefore
// respond correctly when we connect to `google_ip` with SNI=`front_domain`
// and Host=<the real domain>. Routing these via the tunnel instead of the
// Apps Script relay also avoids Apps Script's fixed "Google-Apps-Script"
// User-Agent, which makes Google serve the bot/no-JS fallback for search.
// Kept conservative: only add suffixes that are known to work through the
// Google Front End. A host on the wrong backend will 404 or fail TLS instead
// of loading.
// Domains that are hosted on the Google Front End and therefore reachable via
// the same SNI-rewrite tunnel used for www.google.com itself. Adding a suffix
// here means "TLS CONNECT to google_ip, SNI = front_domain, Host = real name"
// for requests to it — bypassing the Apps Script relay entirely, so there's no
// User-Agent locking and no Apps Script quota.
// When in doubt leave it out: sites that aren't actually on GFE will 404 or
// return a wrong-cert error instead of loading.
const SNI_REWRITE_SUFFIXES: &[&str] = &[
    // Core Google
    "google.com",
    "gstatic.com",
    "googleusercontent.com",
    "googleapis.com",
    "ggpht.com",
    // YouTube family
    "youtube.com",
    "youtu.be",
    "youtube-nocookie.com",
    "ytimg.com",
    // Google Video Transport CDN — video chunks, updates, store payloads.
    // Listing these keeps large downloads on the GFE path instead of the relay.
    "gvt1.com",
    "gvt2.com",
    // Ad + analytics infra. All on GFE, all previously broken the
    // same way YouTube was: SNI-blocked on Iranian DPI, but reachable
    // via `google_ip` with SNI rewritten.
    "doubleclick.net",
    "googlesyndication.com",
    "googleadservices.com",
    "google-analytics.com",
    "googletagmanager.com",
    "googletagservices.com",
    // fonts.googleapis.com is covered by the googleapis.com suffix; listed
    // explicitly for clarity.
    "fonts.googleapis.com",
    // Blogger / Blog.google
    "blogspot.com",
    "blogger.com",
];

/// YouTube HTML/API surfaces that `youtube_via_relay` deliberately moves from
/// SNI-rewrite to the relay path. CDN assets such as `ytimg.com` stay on SNI
/// rewrite so thumbnails/channel art do not burn relay quota; `googlevideo.com`
/// is intentionally not in `SNI_REWRITE_SUFFIXES` because it uses separate EVA
/// edges and breaks when pointed at a normal GFE `google_ip`.
const YOUTUBE_RELAY_WHEN_TOGGLE_SUFFIXES: &[&str] = &[
    "youtube.com",
    "youtu.be",
    "youtube-nocookie.com",
    "youtubei.googleapis.com",
];

/// Built-in DNS-over-HTTPS endpoints. With the default `tunnel_doh=false`,
/// CONNECTs to these hosts on TCP/443 bypass Apps Script and go through plain
/// TCP (or the configured upstream SOCKS5). DoH payloads are already encrypted;
/// this avoids paying an Apps Script round-trip for every browser DNS lookup.
const DEFAULT_DOH_HOSTS: &[&str] = &[
    "cloudflare-dns.com",
    "chrome.cloudflare-dns.com",
    "mozilla.cloudflare-dns.com",
    "1dot1dot1dot1.cloudflare-dns.com",
    "dns.google",
    "dns.google.com",
    "dns.quad9.net",
    "dns11.quad9.net",
    "dns.adguard-dns.com",
    "unfiltered.adguard-dns.com",
    "family.adguard-dns.com",
    "dns.nextdns.io",
    "doh.opendns.com",
    "doh.cleanbrowsing.org",
    "doh.dns.sb",
    "dns0.eu",
    "dns.alidns.com",
    "doh.pub",
    "dns.mullvad.net",
];

fn matches_sni_rewrite(host: &str, youtube_via_relay: bool) -> bool {
    let h = host.to_ascii_lowercase();
    let h = h.trim_end_matches('.');
    if youtube_via_relay
        && YOUTUBE_RELAY_WHEN_TOGGLE_SUFFIXES
            .iter()
            .any(|s| h == *s || h.ends_with(&format!(".{}", s)))
    {
        return false;
    }
    SNI_REWRITE_SUFFIXES
        .iter()
        .any(|s| h == *s || h.ends_with(&format!(".{}", s)))
}

fn hosts_override<'a>(
    hosts: &'a std::collections::HashMap<String, String>,
    host: &str,
) -> Option<&'a str> {
    let h = host.to_ascii_lowercase();
    let h = h.trim_end_matches('.');
    if let Some(ip) = hosts.get(h) {
        return Some(ip.as_str());
    }
    let parts: Vec<&str> = h.split('.').collect();
    for i in 1..parts.len() {
        let parent = parts[i..].join(".");
        if let Some(ip) = hosts.get(&parent) {
            return Some(ip.as_str());
        }
    }
    None
}

#[derive(Debug, thiserror::Error)]
pub enum ProxyError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
}

pub struct ProxyServer {
    host: String,
    port: u16,
    socks5_port: Option<u16>,
    /// `None` in direct mode: no Apps Script relay is wired up, only the
    /// SNI-rewrite tunnel path is live.
    fronter: Option<Arc<DomainFronter>>,
    /// HTTP(S) JSON relay transport. Apps Script and serverless JSON both expose
    /// the same high-level `{s,h,b}` envelope to the proxy dispatch.
    relay: Option<Arc<RelayTransport>>,
    mitm: Arc<Mutex<MitmCertManager>>,
    rewrite_ctx: Arc<RewriteCtx>,
    tunnel_mux: Option<Arc<TunnelMux>>,
    coalesce_step_ms: u64,
    coalesce_max_ms: u64,
    status_port: u16,
}

pub struct RewriteCtx {
    pub google_ip: String,
    pub front_domain: String,
    pub hosts: std::collections::HashMap<String, String>,
    pub tls_connector: TlsConnector,
    pub upstream_socks5: Option<String>,
    pub mode: Mode,
    /// If true, YouTube traffic bypasses the SNI-rewrite tunnel and
    /// goes through the Apps Script relay instead. See config.rs for
    /// the trade-off; see `youtube_via_relay` in `config`.
    pub youtube_via_relay: bool,
    /// User-configured hostnames that should skip the relay entirely
    /// and pass through as plain TCP (optionally via upstream_socks5).
    /// See config.rs `passthrough_hosts` for matching rules.
    pub passthrough_hosts: Vec<String>,
    pub block_quic: bool,
    pub bypass_doh: bool,
    pub bypass_doh_hosts: Vec<String>,
    /// User-configured multi-edge fronting groups. Empty means feature off.
    pub fronting_groups: Vec<Arc<FrontingGroupResolved>>,
    /// Per-domain overrides (first match wins).
    pub domain_overrides: Vec<DomainOverride>,
    pub lan_token: Option<String>,
    pub lan_allowlist: Option<Vec<String>>,
    pub listen_host: String,
}

#[derive(Debug, Clone)]
pub struct FrontingGroupResolved {
    pub name: String,
    pub ip: String,
    pub sni: String,
    pub server_name: ServerName<'static>,
    domains_normalized: Vec<String>,
}

impl FrontingGroupResolved {
    fn from_config(group: &FrontingGroup) -> Result<Self, String> {
        let server_name = ServerName::try_from(group.sni.clone())
            .map_err(|e| format!("invalid sni '{}': {}", group.sni, e))?;
        let domains_normalized = group
            .domains
            .iter()
            .map(|d| d.trim().trim_end_matches('.').to_ascii_lowercase())
            .filter(|d| !d.is_empty())
            .collect();
        Ok(Self {
            name: group.name.clone(),
            ip: group.ip.clone(),
            sni: group.sni.clone(),
            server_name,
            domains_normalized,
        })
    }
}

#[inline]
fn is_dot_anchored_match(host: &str, entry: &str) -> bool {
    if host == entry {
        return true;
    }
    let hb = host.as_bytes();
    let eb = entry.as_bytes();
    hb.len() > eb.len() && hb.ends_with(eb) && hb[hb.len() - eb.len() - 1] == b'.'
}

pub fn match_fronting_group<'a>(
    host: &str,
    groups: &'a [Arc<FrontingGroupResolved>],
) -> Option<&'a Arc<FrontingGroupResolved>> {
    if groups.is_empty() {
        return None;
    }
    let h = host.to_ascii_lowercase();
    let h = h.trim_end_matches('.');
    if h.is_empty() {
        return None;
    }
    groups.iter().find(|group| {
        group
            .domains_normalized
            .iter()
            .any(|entry| is_dot_anchored_match(h, entry))
    })
}

fn first_matching_domain_override<'a>(
    host: &str,
    overrides: &'a [DomainOverride],
) -> Option<&'a DomainOverride> {
    if overrides.is_empty() {
        return None;
    }
    let h = host.to_ascii_lowercase();
    let h = h.trim_end_matches('.');
    overrides.iter().find(|o| {
        let e = o.host.trim().trim_end_matches('.').to_ascii_lowercase();
        if e.is_empty() {
            return false;
        }
        let suffix = e.strip_prefix("*.").or_else(|| e.strip_prefix('.'));
        if let Some(suffix) = suffix {
            h == suffix || h.ends_with(&format!(".{}", suffix))
        } else {
            h == e
        }
    })
}

fn forced_route_for_host(
    mode: Mode,
    host: &str,
    overrides: &[DomainOverride],
) -> Option<RouteDecision> {
    let o = first_matching_domain_override(host, overrides)?;
    let r = o.force_route.as_deref()?.trim().to_ascii_lowercase();
    match r.as_str() {
        "direct" => Some(RouteDecision::Passthrough),
        "sni_rewrite" => Some(RouteDecision::SniRewrite),
        "full_tunnel" => Some(RouteDecision::FullTunnel),
        // "relay" is intentionally not mapped here because the policy still needs to
        // differentiate MitmRelayTls vs RelayPlainHttp based on payload peek. Leaving
        // it as None means "don't force; let normal relay decision happen".
        "relay" => {
            if mode == Mode::Direct {
                Some(RouteDecision::Passthrough)
            } else {
                None
            }
        }
        _ => None,
    }
}

fn host_matches_doh_entry(h: &str, entry: &str) -> bool {
    let e = entry.trim().trim_end_matches('.').to_ascii_lowercase();
    let e = e.strip_prefix('.').unwrap_or(&e);
    if e.is_empty() {
        return false;
    }
    h == e || h.ends_with(&format!(".{}", e))
}

pub fn matches_doh_host(host: &str, extra: &[String]) -> bool {
    let h = host.to_ascii_lowercase();
    let h = h.trim_end_matches('.');
    if h.is_empty() {
        return false;
    }
    DEFAULT_DOH_HOSTS
        .iter()
        .any(|s| host_matches_doh_entry(h, s))
        || extra.iter().any(|s| host_matches_doh_entry(h, s))
}

/// True if `host` matches any entry in the user's passthrough list.
/// `*.example.com` is accepted as an alias for `.example.com`.
/// Match is case-insensitive. Entries match either exactly, or as a
/// suffix if they start with "." (e.g. ".internal.example" matches
/// "a.b.internal.example" and the bare "internal.example"). Bare
/// entries like "example.com" only match the exact hostname — users
/// who want subdomains included should use ".example.com".
pub fn matches_passthrough(host: &str, list: &[String]) -> bool {
    if list.is_empty() {
        return false;
    }
    let h = host.to_ascii_lowercase();
    let h = h.trim_end_matches('.');
    list.iter().any(|entry| {
        let e = entry.trim().trim_end_matches('.').to_ascii_lowercase();
        if e.is_empty() {
            return false;
        }
        if let Some(suffix) = e.strip_prefix("*.").or_else(|| e.strip_prefix('.')) {
            h == suffix || h.ends_with(&format!(".{}", suffix))
        } else {
            h == e
        }
    })
}

const MAX_CONCURRENT_CLIENTS: usize = 512;

impl ProxyServer {
    pub fn new(config: &Config, mitm: Arc<Mutex<MitmCertManager>>) -> Result<Self, ProxyError> {
        let mode = config
            .mode_kind()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, format!("{e}")))?;

        // `direct` mode skips the Apps Script relay entirely, so we must
        // not try to construct the DomainFronter — it errors on a missing
        // `script_id`, which is exactly the state a bootstrapping user is in.
        let fronter = match mode {
            Mode::AppsScript | Mode::Full => {
                let f = DomainFronter::new(config)
                    .map_err(|e| std::io::Error::other(format!("{e}")))?;
                Some(Arc::new(f))
            }
            Mode::VercelEdge | Mode::Direct => None,
        };
        let relay = RelayTransport::new(config, fronter.clone())
            .map_err(|e| std::io::Error::other(format!("{e}")))?;

        let tls_config = if config.verify_ssl {
            let mut roots = tokio_rustls::rustls::RootCertStore::empty();
            roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
            ClientConfig::builder()
                .with_root_certificates(roots)
                .with_no_client_auth()
        } else {
            ClientConfig::builder()
                .dangerous()
                .with_custom_certificate_verifier(Arc::new(NoVerify))
                .with_no_client_auth()
        };
        let tls_connector = TlsConnector::from(Arc::new(tls_config));

        if config.tunnel_doh && !config.bypass_doh_hosts.is_empty() {
            tracing::warn!(
                "config: bypass_doh_hosts has {} entries but tunnel_doh=true; DoH bypass is disabled, so the extras have no effect",
                config.bypass_doh_hosts.len()
            );
        }
        if mode == Mode::Full && !config.fronting_groups.is_empty() {
            tracing::warn!(
                "config: fronting_groups has {} entries but mode=full; full mode preserves end-to-end TLS, so groups never fire",
                config.fronting_groups.len()
            );
        }

        let mut fronting_groups: Vec<Arc<FrontingGroupResolved>> =
            Vec::with_capacity(config.fronting_groups.len());
        let mut seen_names = std::collections::HashSet::new();
        for group in &config.fronting_groups {
            let resolved = FrontingGroupResolved::from_config(group).map_err(|e| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("fronting_groups['{}']: {}", group.name, e),
                )
            })?;
            if !seen_names.insert(resolved.name.clone()) {
                tracing::warn!(
                    "fronting group name '{}' is used more than once; logs may be ambiguous",
                    resolved.name
                );
            }
            tracing::info!(
                "fronting group '{}': sni={} ip={} domains={}",
                resolved.name,
                resolved.sni,
                resolved.ip,
                resolved.domains_normalized.len()
            );
            fronting_groups.push(Arc::new(resolved));
        }

        let rewrite_ctx = Arc::new(RewriteCtx {
            google_ip: config.google_ip.clone(),
            front_domain: config.front_domain.clone(),
            hosts: config.hosts.clone(),
            tls_connector,
            upstream_socks5: config.upstream_socks5.clone(),
            mode,
            youtube_via_relay: config.youtube_via_relay,
            passthrough_hosts: config.passthrough_hosts.clone(),
            block_quic: config.block_quic,
            bypass_doh: !config.tunnel_doh,
            bypass_doh_hosts: config.bypass_doh_hosts.clone(),
            fronting_groups,
            domain_overrides: config.domain_overrides.clone(),
            lan_token: config.lan_token.clone(),
            lan_allowlist: config.lan_allowlist.clone(),
            listen_host: config.listen_host.clone(),
        });

        Ok(Self {
            host: config.listen_host.clone(),
            port: config.listen_port,
            socks5_port: config.socks5_port,
            fronter,
            relay,
            mitm,
            rewrite_ctx,
            tunnel_mux: None, // initialized in run() inside the tokio runtime
            coalesce_step_ms: if config.coalesce_step_ms > 0 {
                config.coalesce_step_ms as u64
            } else {
                40
            },
            coalesce_max_ms: if config.coalesce_max_ms > 0 {
                config.coalesce_max_ms as u64
            } else {
                1000
            },
            status_port: config.listen_port + 2,
        })
    }

    pub fn fronter(&self) -> Option<Arc<DomainFronter>> {
        self.fronter.clone()
    }
    pub async fn run(
        mut self,
        mut shutdown_rx: tokio::sync::oneshot::Receiver<()>,
    ) -> Result<(), ProxyError> {
        // Initialize TunnelMux inside the runtime (tokio::spawn requires it).
        if self.rewrite_ctx.mode == Mode::Full {
            if let Some(f) = self.fronter.as_ref() {
                self.tunnel_mux = Some(TunnelMux::start(
                    f.clone(),
                    self.coalesce_step_ms,
                    self.coalesce_max_ms,
                ));
            }
        }

        let http_addr = format!("{}:{}", self.host, self.port);
        let http_listener = TcpListener::bind(&http_addr).await?;
        let socks_listener = if let Some(socks5_port) = self.socks5_port {
            Some(TcpListener::bind(format!("{}:{}", self.host, socks5_port)).await?)
        } else {
            None
        };
        tracing::warn!(
            "Listening HTTP   on {} — set your browser HTTP proxy to this address.",
            http_addr
        );
        if let Some(socks5_port) = self.socks5_port {
            tracing::warn!(
                "Listening SOCKS5 on {}:{} — xray / Telegram / app-level SOCKS5 clients use this.",
                self.host,
                socks5_port
            );
        } else {
            tracing::warn!("SOCKS5 listener disabled.");
        }
        log_lan_access_hints(&self.host, self.port, self.socks5_port);
        // Pre-warm the outbound Google-edge TLS pool so the user's first
        // request burst does not pay fresh handshakes. This opens fronted TLS
        // sockets only; it does not spend Apps Script executions by itself.
        // Best-effort and skipped in direct mode because there is no fronter.
        if let Some(warm_fronter) = self.fronter.clone() {
            tokio::spawn(async move {
                // Warm fronted TLS sockets only; this does not spend Apps
                // Script executions. A larger pool helps absorb browser
                // startup bursts without spending Apps Script executions.
                warm_fronter.warm(24).await;
            });
        }

        let keepalive_task = if let Some(keepalive_fronter) = self.fronter.clone() {
            tokio::spawn(async move {
                keepalive_fronter.run_h1_keepalive().await;
            })
        } else {
            tokio::spawn(async move { std::future::pending::<()>().await })
        };

        // Local status API (best-effort).
        {
            let bind_host = if self.host.trim() == "0.0.0.0" || self.host.trim() == "::" {
                // Still bind status on localhost only, even if proxy is LAN-bound.
                "127.0.0.1".to_string()
            } else {
                "127.0.0.1".to_string()
            };
            let mode = self.rewrite_ctx.mode.as_str().to_string();
            let http_listen = (self.host.clone(), self.port);
            let socks_listen = self.socks5_port.map(|p| (self.host.clone(), p));
            let fronter = self.fronter.clone();
            let port = self.status_port;
            tokio::spawn(async move {
                let _ = crate::status_api::serve_status_api(
                    &bind_host,
                    port,
                    mode,
                    http_listen,
                    socks_listen,
                    fronter,
                )
                .await;
            });
        }

        let stats_task = if let Some(stats_fronter) = self.fronter.clone() {
            tokio::spawn(async move {
                let mut ticker = tokio::time::interval(std::time::Duration::from_secs(60));
                ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
                ticker.tick().await;
                loop {
                    ticker.tick().await;
                    let s = stats_fronter.snapshot_stats();
                    if s.relay_calls > 0 || s.cache_hits > 0 {
                        tracing::info!("{}", s.fmt_line());
                    }
                }
            })
        } else {
            tokio::spawn(async move { std::future::pending::<()>().await })
        };

        let http_relay = self.relay.clone();
        let http_mitm = self.mitm.clone();
        let http_ctx = self.rewrite_ctx.clone();
        let http_mux = self.tunnel_mux.clone();
        let mut http_task = tokio::spawn(async move {
            let mut fd_exhaust_count: u64 = 0;
            let limiter = Arc::new(Semaphore::new(MAX_CONCURRENT_CLIENTS));
            // Track per-client work in a JoinSet so a listener shutdown aborts
            // in-flight clients and drops the old `DomainFronter` (fresh config
            // applies on the next Start).
            let mut children: tokio::task::JoinSet<()> = tokio::task::JoinSet::new();
            loop {
                // Opportunistic reap so completed children don't pile up
                // memory on long-running proxies.
                while children.try_join_next().is_some() {}

                let (sock, peer) = match http_listener.accept().await {
                    Ok(x) => {
                        fd_exhaust_count = 0;
                        x
                    }
                    Err(e) => {
                        accept_backoff("http", &e, &mut fd_exhaust_count).await;
                        continue;
                    }
                };
                let _ = sock.set_nodelay(true);
                let relay = http_relay.clone();
                let mitm = http_mitm.clone();
                let rewrite_ctx = http_ctx.clone();
                let mux = http_mux.clone();
                let permit = limiter.clone().acquire_owned().await;
                children.spawn(async move {
                    let _permit = permit;
                    if let Err(e) = handle_http_client(sock, relay, mitm, rewrite_ctx, mux).await {
                        tracing::debug!("http client {} closed: {}", peer, e);
                    }
                });
            }
        });

        let mut socks_task = if let Some(socks_listener) = socks_listener {
            let socks_relay = self.relay.clone();
            let socks_mitm = self.mitm.clone();
            let socks_ctx = self.rewrite_ctx.clone();
            let socks_mux = self.tunnel_mux.clone();
            tokio::spawn(async move {
                let mut fd_exhaust_count: u64 = 0;
                let limiter = Arc::new(Semaphore::new(MAX_CONCURRENT_CLIENTS));
                // Same pattern as http_task above — JoinSet so shutdown
                // drops in-flight SOCKS5 clients instead of leaving them to
                // keep running on the stale config.
                let mut children: tokio::task::JoinSet<()> = tokio::task::JoinSet::new();
                loop {
                    while children.try_join_next().is_some() {}

                    let (sock, peer) = match socks_listener.accept().await {
                        Ok(x) => {
                            fd_exhaust_count = 0;
                            x
                        }
                        Err(e) => {
                            accept_backoff("socks", &e, &mut fd_exhaust_count).await;
                            continue;
                        }
                    };
                    let _ = sock.set_nodelay(true);
                    let relay = socks_relay.clone();
                    let mitm = socks_mitm.clone();
                    let rewrite_ctx = socks_ctx.clone();
                    let mux = socks_mux.clone();
                    let permit = limiter.clone().acquire_owned().await;
                    children.spawn(async move {
                        let _permit = permit;
                        if let Err(e) =
                            handle_socks5_client(sock, relay, mitm, rewrite_ctx, mux).await
                        {
                            tracing::debug!("socks client {} closed: {}", peer, e);
                        }
                    });
                }
            })
        } else {
            tokio::spawn(async move { std::future::pending::<()>().await })
        };

        tokio::select! {
            biased;
            _ = &mut shutdown_rx => {
                tracing::info!("Shutdown signal received, stopping listeners");
                keepalive_task.abort();
                stats_task.abort();
                http_task.abort();
                socks_task.abort();
            }
            _ = &mut http_task => {}
            _ = &mut socks_task => {}
        }

        Ok(())
    }
}

fn log_lan_access_hints(bind_host: &str, http_port: u16, socks5_port: Option<u16>) {
    if !is_lan_bind_host(bind_host) {
        return;
    }
    let Some(ip) = primary_lan_ipv4() else {
        tracing::warn!("LAN sharing is enabled, but no LAN IPv4 address was detected.");
        return;
    };
    tracing::info!("LAN HTTP proxy   : {}:{}", ip, http_port);
    if let Some(port) = socks5_port {
        tracing::info!("LAN SOCKS5 proxy : {}:{}", ip, port);
    }
}

fn is_lan_bind_host(host: &str) -> bool {
    let h = host.trim();
    h == "0.0.0.0" || h == "::"
}

fn primary_lan_ipv4() -> Option<std::net::Ipv4Addr> {
    let sock = std::net::UdpSocket::bind((std::net::Ipv4Addr::UNSPECIFIED, 0)).ok()?;
    let _ = sock.set_read_timeout(Some(std::time::Duration::from_millis(500)));
    sock.connect((std::net::Ipv4Addr::new(192, 0, 2, 1), 80))
        .ok()?;
    let ip = match sock.local_addr().ok()?.ip() {
        std::net::IpAddr::V4(ip) => ip,
        std::net::IpAddr::V6(_) => return None,
    };
    lan_access_ipv4(ip).then_some(ip)
}

fn lan_access_ipv4(ip: std::net::Ipv4Addr) -> bool {
    !ip.is_loopback() && !ip.is_unspecified() && (ip.is_private() || ip.is_link_local())
}

/// Back-off helper for the accept() loop.
///
/// When the process hits its file-descriptor limit
/// (EMFILE — `No file descriptors available`), `accept()` returns that
/// error synchronously and is immediately ready to fire again. The old
/// loop just `continue`'d, producing a wall of identical ERROR lines
/// thousands per second and starving the tokio runtime of CPU that
/// existing connections would have used to drain and close.
///
/// Two things this does right:
///   1. Sleeps when `EMFILE` / `ENFILE` are seen, proportional to how long
///      the problem has been going on (exponential-ish, capped at 2s).
///      Gives existing connections a chance to finish and free fds.
///   2. Rate-limits the log line: first occurrence logs a full warning
///      with fix instructions, subsequent ones log once per 100 errors
///      so the log doesn't fill up.
async fn accept_backoff(kind: &str, err: &std::io::Error, count: &mut u64) {
    let is_fd_limit = matches!(
        err.raw_os_error(),
        Some(libc_emfile) if libc_emfile == 24 || libc_emfile == 23
    );

    *count = count.saturating_add(1);

    if is_fd_limit {
        if *count == 1 {
            tracing::warn!(
                "accept ({}) hit RLIMIT_NOFILE: {}. Backing off. Raise the fd limit: \
                 `ulimit -n 65536` before starting, or (OpenWRT) use the shipped procd \
                 init which sets nofile=16384. The listener will keep retrying.",
                kind,
                err
            );
        } else if (*count).is_multiple_of(100) {
            tracing::warn!(
                "accept ({}) still fd-limited after {} retries. Current connections \
                 need to finish before we can accept new ones.",
                kind,
                *count
            );
        }
        // Back off exponentially-ish up to 2s. First hit: 50ms, 10th hit:
        // ~500ms, 50th+: 2s cap.
        let backoff_ms = (50u64 * (*count).min(40)).min(2000);
        tokio::time::sleep(std::time::Duration::from_millis(backoff_ms)).await;
    } else {
        // Transient non-EMFILE error (e.g. ECONNABORTED from a client that
        // went away during the handshake). One-line log, short sleep to
        // avoid a tight loop in case it repeats.
        tracing::error!("accept ({}): {}", kind, err);
        tokio::time::sleep(std::time::Duration::from_millis(5)).await;
    }
}

async fn handle_http_client(
    mut sock: TcpStream,
    relay: Option<Arc<RelayTransport>>,
    mitm: Arc<Mutex<MitmCertManager>>,
    rewrite_ctx: Arc<RewriteCtx>,
    tunnel_mux: Option<Arc<TunnelMux>>,
) -> std::io::Result<()> {
    let (head, leftover) = match read_http_head(&mut sock).await? {
        HeadReadResult::Got { head, leftover } => (head, leftover),
        HeadReadResult::Closed => return Ok(()),
        HeadReadResult::Oversized => {
            tracing::warn!(
                "request head exceeds {} bytes; refusing with 431",
                MAX_HTTP_HEADER_BYTES
            );
            let _ = sock
                .write_all(
                    b"HTTP/1.1 431 Request Header Fields Too Large\r\n\
                      Connection: close\r\n\
                      Content-Length: 0\r\n\r\n",
                )
                .await;
            let _ = sock.flush().await;
            return Ok(());
        }
    };

    let (method, target, _version, headers) = parse_request_head(&head)
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "bad request"))?;

    // Safer LAN mode: if we bound to LAN and a token/allowlist is configured,
    // enforce it on the initial proxy request (CONNECT or plain HTTP).
    if lan_auth_configured(&rewrite_ctx)
        && rewrite_ctx.listen_host_is_lan_bound()
        && !lan_auth_ok(&sock, &headers, &rewrite_ctx)
    {
        let _ = sock
            .write_all(
                b"HTTP/1.1 401 Unauthorized\r\n\
                  Content-Type: text/plain; charset=utf-8\r\n\
                  Content-Length: 63\r\n\
                  Connection: close\r\n\r\n\
                  Unauthorized (LAN mode): missing/invalid token or IP.",
            )
            .await;
        let _ = sock.flush().await;
        return Ok(());
    }

    if method.eq_ignore_ascii_case("CONNECT") {
        let (host, port) = parse_host_port(&target);
        sock.write_all(b"HTTP/1.1 200 Connection Established\r\n\r\n")
            .await?;
        sock.flush().await?;
        dispatch_tunnel(sock, host, port, relay, mitm, rewrite_ctx, tunnel_mux).await
    } else {
        // Plain HTTP proxy request (e.g. `GET http://…`).
        //
        // apps_script mode: relay through Apps Script fronter.
        // direct mode: no fronter exists; passthrough as raw TCP (direct
        // or via upstream_socks5) so a bare `http://example.com` doesn't 502.
        match relay {
            Some(r) => do_plain_http(sock, &head, &leftover, r).await,
            None => do_plain_http_passthrough(sock, &head, &leftover, &rewrite_ctx).await,
        }
    }
}

// ---------- SOCKS5 ----------

async fn handle_socks5_client(
    mut sock: TcpStream,
    relay: Option<Arc<RelayTransport>>,
    mitm: Arc<Mutex<MitmCertManager>>,
    rewrite_ctx: Arc<RewriteCtx>,
    tunnel_mux: Option<Arc<TunnelMux>>,
) -> std::io::Result<()> {
    // RFC 1928 handshake: VER=5, NMETHODS, METHODS...
    let mut hdr = [0u8; 2];
    sock.read_exact(&mut hdr).await?;
    if hdr[0] != 0x05 {
        return Ok(());
    }
    let nmethods = hdr[1] as usize;
    let mut methods = vec![0u8; nmethods];
    sock.read_exact(&mut methods).await?;
    // Only "no auth" (0x00) is supported.
    if !methods.contains(&0x00) {
        sock.write_all(&[0x05, 0xff]).await?;
        return Ok(());
    }
    sock.write_all(&[0x05, 0x00]).await?;

    // Request: VER=5, CMD, RSV=0, ATYP, DST.ADDR, DST.PORT
    let mut req = [0u8; 4];
    sock.read_exact(&mut req).await?;
    if req[0] != 0x05 {
        return Ok(());
    }
    let cmd = req[1];
    if cmd != 0x01 && cmd != 0x03 {
        // CONNECT and UDP ASSOCIATE only.
        sock.write_all(&[0x05, 0x07, 0x00, 0x01, 0, 0, 0, 0, 0, 0])
            .await?;
        return Ok(());
    }
    let atyp = req[3];
    let host: String = match atyp {
        0x01 => {
            let mut ip = [0u8; 4];
            sock.read_exact(&mut ip).await?;
            format!("{}.{}.{}.{}", ip[0], ip[1], ip[2], ip[3])
        }
        0x03 => {
            let mut len = [0u8; 1];
            sock.read_exact(&mut len).await?;
            let mut name = vec![0u8; len[0] as usize];
            sock.read_exact(&mut name).await?;
            String::from_utf8_lossy(&name).into_owned()
        }
        0x04 => {
            let mut ip = [0u8; 16];
            sock.read_exact(&mut ip).await?;
            let addr = std::net::Ipv6Addr::from(ip);
            addr.to_string()
        }
        _ => {
            sock.write_all(&[0x05, 0x08, 0x00, 0x01, 0, 0, 0, 0, 0, 0])
                .await?;
            return Ok(());
        }
    };
    let mut port_buf = [0u8; 2];
    sock.read_exact(&mut port_buf).await?;
    let port = u16::from_be_bytes(port_buf);

    if rewrite_ctx.listen_host_is_lan_bound() && !lan_socks_allowed(&sock, &rewrite_ctx) {
        tracing::warn!(
            "SOCKS5 LAN auth rejected peer={:?} target={}:{}",
            sock.peer_addr().ok(),
            host,
            port
        );
        // REP=0x02: connection not allowed by ruleset.
        sock.write_all(&[0x05, 0x02, 0x00, 0x01, 0, 0, 0, 0, 0, 0])
            .await?;
        return Ok(());
    }

    if cmd == 0x03 {
        tracing::info!("SOCKS5 UDP ASSOCIATE requested for {}:{}", host, port);
        return handle_socks5_udp_associate(sock, rewrite_ctx, tunnel_mux).await;
    }

    tracing::info!("SOCKS5 CONNECT -> {}:{}", host, port);

    // Success reply with zeroed BND.
    sock.write_all(&[0x05, 0x00, 0x00, 0x01, 0, 0, 0, 0, 0, 0])
        .await?;
    sock.flush().await?;

    dispatch_tunnel(sock, host, port, relay, mitm, rewrite_ctx, tunnel_mux).await
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct SocksUdpTarget {
    host: String,
    port: u16,
    atyp: u8,
    addr: Vec<u8>,
}

/// Per-target relay session state shared between the dispatch loop and
/// the per-session task. The dispatch loop pushes uplink datagrams via
/// `uplink`; the task drains the upstream and serializes both directions
/// onto a single tunnel-mux call at a time. `sid` is held here so the
/// dispatch teardown path can issue close_session for any task it has
/// to abort mid-await.
struct UdpRelaySession {
    sid: String,
    uplink: mpsc::Sender<Vec<u8>>,
}

/// All per-ASSOCIATE UDP relay state behind a single mutex so insertion
/// order, the live-session map, and per-task self-removal can all stay
/// consistent. Wrapping each separately invited a slow leak: the
/// previous design's `insertion_order` deque was only pruned on
/// overflow eviction, so a long-lived ASSOCIATE that opened many
/// short-lived sessions accumulated dead `SocksUdpTarget` entries.
struct UdpRelayState {
    sessions: HashMap<SocksUdpTarget, UdpRelaySession>,
    /// Insertion-order log for FIFO eviction. NOT a real LRU — repeated
    /// uplinks to a hot session do not move it to the back. We keep it
    /// in lockstep with `sessions` (insert appends; remove scans and
    /// erases the matching entry — O(N) but N ≤ 256).
    order: VecDeque<SocksUdpTarget>,
}

impl UdpRelayState {
    fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            order: VecDeque::new(),
        }
    }

    fn get_uplink(&self, target: &SocksUdpTarget) -> Option<mpsc::Sender<Vec<u8>>> {
        self.sessions.get(target).map(|s| s.uplink.clone())
    }

    fn insert(&mut self, target: SocksUdpTarget, session: UdpRelaySession) {
        self.order.push_back(target.clone());
        self.sessions.insert(target, session);
    }

    fn remove(&mut self, target: &SocksUdpTarget) {
        if let Some(pos) = self.order.iter().position(|t| t == target) {
            self.order.remove(pos);
        }
        self.sessions.remove(target);
    }

    /// Pop the oldest session entries until `sessions.len() < cap`.
    /// Stale `order` entries (already removed by self-cleanup on a
    /// task's natural exit) are quietly skipped.
    fn evict_until_under(&mut self, cap: usize) -> Vec<SocksUdpTarget> {
        let mut evicted = Vec::new();
        while self.sessions.len() >= cap {
            let Some(victim) = self.order.pop_front() else {
                break;
            };
            if self.sessions.remove(&victim).is_some() {
                evicted.push(victim);
            }
        }
        evicted
    }

    /// Snapshot live sids for the teardown close_session sweep. We
    /// take a copy (not a drain) so the caller can decide whether to
    /// also clear the map.
    fn live_sids(&self) -> Vec<String> {
        self.sessions.values().map(|s| s.sid.clone()).collect()
    }

    fn clear(&mut self) {
        self.sessions.clear();
        self.order.clear();
    }
}

/// SOCKS5 UDP request frame: 4-byte header + atyp-specific address + 2-byte
/// port + payload. DOMAIN atyp uses a 1-byte length prefix + up to 255
/// bytes, so the largest header is `4 + 1 + 255 + 2 = 262`. Round to 300
/// for safety; payload itself can be a full 64 KB datagram.
const SOCKS5_UDP_RECV_BUF_BYTES: usize = 65535 + 300;

/// Bound on per-session uplink queue depth. UDP is lossy by design — if
/// the per-session task can't keep up, drop the newest datagram (caller
/// uses `try_send`) instead of stalling the whole UDP relay loop.
const UDP_UPLINK_QUEUE: usize = 64;

/// Initial poll spacing when a session is idle. Tunnel-node already
/// long-polls each empty `udp_data` for up to 5 s, so this is a
/// client-side floor — bursts of upstream packets reset back to this.
const UDP_INITIAL_POLL_DELAY: Duration = Duration::from_millis(500);

/// Cap on the exponential backoff for an idle session. After this many
/// seconds of zero traffic in either direction, polls happen at most
/// once per `UDP_MAX_POLL_DELAY` plus the tunnel-node long-poll window —
/// so an idle UDP destination costs roughly one batch slot every 35 s.
const UDP_MAX_POLL_DELAY: Duration = Duration::from_secs(30);

/// Cap on simultaneous UDP relay sessions per SOCKS5 ASSOCIATE. STUN
/// candidate gathering and DNS fanout produce dozens of distinct
/// targets; an abusive or runaway client could produce thousands.
/// 256 is generous for legitimate use and bounds tunnel-node UDP
/// sessions a single ASSOCIATE can hold open.
///
/// Eviction policy is FIFO by insertion time, not true LRU — repeated
/// uplinks to a hot session do not move it to the back. Real LRU
/// would need a touch on every uplink (extra lock acquisition per
/// datagram); the long-tail of dead targets gets cleaned up here just
/// fine without that cost, and live targets are typically also recently
/// inserted.
const MAX_UDP_SESSIONS_PER_ASSOCIATE: usize = 256;

/// Drop UDP datagrams larger than this (pre-base64). Standard MTU is
/// 1500B, jumbo frames are ~9000B; anything above that is either a
/// pathologically fragmented IP datagram or abusive traffic. Each
/// datagram carries ~33% base64 + JSON envelope overhead and consumes
/// Apps Script per-account quota, so a permissive ceiling here matters.
const MAX_UDP_PAYLOAD_BYTES: usize = 9 * 1024;

async fn handle_socks5_udp_associate(
    mut control: TcpStream,
    rewrite_ctx: Arc<RewriteCtx>,
    tunnel_mux: Option<Arc<TunnelMux>>,
) -> std::io::Result<()> {
    if rewrite_ctx.mode != Mode::Full {
        tracing::debug!("UDP ASSOCIATE rejected: only full mode supports UDP tunneling");
        write_socks5_reply(&mut control, 0x07, None).await?;
        return Ok(());
    }
    let Some(mux) = tunnel_mux else {
        tracing::debug!("UDP ASSOCIATE rejected: full mode has no tunnel mux");
        write_socks5_reply(&mut control, 0x01, None).await?;
        return Ok(());
    };

    // Per RFC 1928 §6 the UDP relay only accepts datagrams from the
    // SOCKS5 client. We pin the source IP to the control TCP peer up
    // front so a third party on the bind interface can't hijack the
    // session by sending the first datagram. THIS — not the bind IP
    // below — is what actually keeps unauthenticated traffic out.
    let client_peer_ip = control.peer_addr()?.ip();

    // Bind the UDP relay to the same local IP the SOCKS5 client used
    // to reach the control TCP socket. `TcpStream::local_addr()` on an
    // accepted socket returns the concrete terminating address (e.g.
    // 127.0.0.1 for a loopback client, 192.168.1.5 for a LAN client),
    // not the listener's bind specifier — so this naturally tracks the
    // path the client took. Source-IP filtering above is the security
    // boundary; the bind choice is just about reachability.
    let bind_ip = control.local_addr()?.ip();
    let udp = Arc::new(UdpSocket::bind(SocketAddr::new(bind_ip, 0)).await?);
    write_socks5_reply(&mut control, 0x00, Some(udp.local_addr()?)).await?;
    tracing::info!(
        "SOCKS5 UDP relay bound on {} for client {}",
        udp.local_addr()?,
        client_peer_ip
    );

    let mut buf = vec![0u8; SOCKS5_UDP_RECV_BUF_BYTES];
    let mut control_buf = [0u8; 1];
    let mut client_addr: Option<SocketAddr> = None;
    let state: Arc<Mutex<UdpRelayState>> = Arc::new(Mutex::new(UdpRelayState::new()));
    // Tracking per-target tasks here — instead of bare `tokio::spawn`
    // — lets the teardown path call `abort_all()`, cancelling any
    // in-flight `mux.udp_data` await. Without it, a task mid-poll
    // could keep paying tunnel-node round trips for up to 5 s after
    // the SOCKS5 client went away.
    let mut tasks: JoinSet<()> = JoinSet::new();
    let mut oversized_dropped: u64 = 0;
    let mut sessions_evicted: u64 = 0;
    let mut foreign_ip_drops: u64 = 0;

    loop {
        tokio::select! {
            recv = udp.recv_from(&mut buf) => {
                let (n, peer) = match recv {
                    Ok(v) => v,
                    Err(e) => {
                        tracing::debug!("udp associate recv failed: {}", e);
                        break;
                    }
                };
                // Source-IP check: anything not from the SOCKS5 client's
                // host is dropped silently.
                if peer.ip() != client_peer_ip {
                    foreign_ip_drops += 1;
                    if foreign_ip_drops == 1 || foreign_ip_drops.is_multiple_of(100) {
                        tracing::debug!(
                            "udp dropped from unauthorized source {}: count={}",
                            peer.ip(),
                            foreign_ip_drops,
                        );
                    }
                    continue;
                }

                // Parse BEFORE port-locking. A malformed datagram from
                // the right IP must not pin client_addr to its source
                // port — otherwise a co-tenant on the bind interface
                // can race one bad packet to DoS the legitimate client
                // (whose real datagram, sent from a different ephemeral
                // port, would then be silently rejected).
                let Some((target, payload)) = parse_socks5_udp_packet(&buf[..n]) else {
                    continue;
                };

                // RFC 1928 §6: lock to the first VALID datagram's source
                // port. Subsequent datagrams must come from the same
                // (ip, port) pair.
                if let Some(existing) = client_addr {
                    if existing != peer {
                        continue;
                    }
                } else {
                    tracing::info!("UDP relay locked to client {}", peer);
                    client_addr = Some(peer);
                }

                // Size guard: drop oversize datagrams before they reach
                // the mux. Each datagram costs ~payload * 1.33 in the
                // batched JSON envelope plus tunnel-node CPU; uncapped,
                // a runaway client can exhaust Apps Script quota.
                if payload.len() > MAX_UDP_PAYLOAD_BYTES {
                    oversized_dropped += 1;
                    if oversized_dropped == 1 || oversized_dropped.is_multiple_of(100) {
                        tracing::debug!(
                            "udp datagram dropped: {} B > {} B (count={})",
                            payload.len(),
                            MAX_UDP_PAYLOAD_BYTES,
                            oversized_dropped,
                        );
                    }
                    continue;
                }

                if rewrite_ctx.block_quic && target.port == 443 {
                    tracing::debug!(
                        "udp datagram dropped: block_quic=true, target {}:443",
                        target.host
                    );
                    continue;
                }
                let payload = payload.to_vec();

                // Fast path: existing session — push payload onto its
                // bounded uplink queue, drop on overflow (UDP semantics).
                {
                    let st = state.lock().await;
                    if let Some(uplink) = st.get_uplink(&target) {
                        let _ = uplink.try_send(payload);
                        continue;
                    }
                }

                // Cap reached → evict oldest sessions before opening a
                // new one. Each evicted entry drops its uplink Sender,
                // which causes the per-session task to exit its select
                // and tell tunnel-node to close. Any uplink already in
                // that channel is delivered before the task exits.
                {
                    let mut st = state.lock().await;
                    let evicted = st.evict_until_under(MAX_UDP_SESSIONS_PER_ASSOCIATE);
                    for victim in evicted {
                        sessions_evicted += 1;
                        if sessions_evicted == 1 || sessions_evicted.is_multiple_of(50) {
                            tracing::debug!(
                                "udp session cap {} reached; evicted {}:{} (total evicted={})",
                                MAX_UDP_SESSIONS_PER_ASSOCIATE,
                                victim.host,
                                victim.port,
                                sessions_evicted,
                            );
                        }
                    }
                }

                // New target: open via tunnel-node and spawn the per-session
                // task. The first datagram rides the udp_open op so we
                // save one round trip on session establishment.
                let resp = match mux.udp_open(&target.host, target.port, payload).await {
                    Ok(r) => r,
                    Err(e) => {
                        tracing::debug!(
                            "udp open {}:{} failed: {}",
                            target.host, target.port, e
                        );
                        continue;
                    }
                };
                if let Some(ref e) = resp.e {
                    tracing::debug!("udp open {}:{} failed: {}", target.host, target.port, e);
                    continue;
                }
                let Some(sid) = resp.sid.clone() else {
                    tracing::debug!(
                        "udp open {}:{} returned no sid",
                        target.host, target.port
                    );
                    continue;
                };
                send_udp_response_packets(&udp, peer, &target, &resp).await;

                // Tunnel-node may report eof on the open response if the
                // upstream socket died between bind and the first drain
                // (e.g., immediate ICMP unreachable). The session has
                // already been reaped on that side — skip insert/spawn
                // and let the next datagram from the client retry.
                if resp.eof.unwrap_or(false) {
                    tracing::debug!(
                        "udp open {}:{} returned eof; not tracking session",
                        target.host,
                        target.port,
                    );
                    continue;
                }

                let (uplink_tx, uplink_rx) = mpsc::channel::<Vec<u8>>(UDP_UPLINK_QUEUE);
                let task_mux = mux.clone();
                let task_udp = udp.clone();
                let task_target = target.clone();
                let task_state = state.clone();
                let task_sid = sid.clone();
                tasks.spawn(async move {
                    udp_session_task(
                        task_mux,
                        task_udp,
                        task_sid,
                        task_target.clone(),
                        peer,
                        uplink_rx,
                    )
                    .await;
                    // Natural-exit cleanup (eof / mux error / channel
                    // close): remove from shared state so a future
                    // packet to the same target opens a fresh session,
                    // and so insertion_order doesn't leak. Skipped on
                    // teardown since abort_all cancels this await point.
                    task_state.lock().await.remove(&task_target);
                });

                state.lock().await.insert(
                    target,
                    UdpRelaySession {
                        sid,
                        uplink: uplink_tx,
                    },
                );
            }
            read = control.read(&mut control_buf) => {
                match read {
                    Ok(0) | Err(_) => break,
                    Ok(_) => {}
                }
            }
        }
    }

    // Teardown. Snapshot live sids first; they're authoritative for
    // which tunnel-node sessions still exist. Then clear state — that
    // drops every uplink Sender, so any task waiting on `recv()` wakes
    // and exits naturally. Finally `abort_all` cancels tasks that were
    // mid-`mux.udp_data` await; for those the natural-exit close won't
    // run, so we send close_session here on their behalf.
    let live_sids: Vec<String>;
    {
        let mut st = state.lock().await;
        live_sids = st.live_sids();
        st.clear();
    }
    tasks.abort_all();
    for sid in live_sids {
        mux.close_session(&sid).await;
    }
    Ok(())
}

/// Per-target relay task. Owns one tunnel-node UDP session and shuttles
/// datagrams in both directions through a single in-flight tunnel call
/// at a time. Two cancellation points:
///   * `uplink_rx.recv()` returns `None` when the dispatch loop drops
///     the matching `Sender` (SOCKS5 client gone, or session evicted).
///   * `mux.udp_data` returns eof / error when the tunnel-node session
///     is reaped or the target is unreachable.
async fn udp_session_task(
    mux: Arc<TunnelMux>,
    udp: Arc<UdpSocket>,
    sid: String,
    target: SocksUdpTarget,
    client_addr: SocketAddr,
    mut uplink_rx: mpsc::Receiver<Vec<u8>>,
) {
    let mut backoff = UDP_INITIAL_POLL_DELAY;
    loop {
        // `biased;` prefers uplink so an active client doesn't get
        // shadowed by a long sleep. Both branches are cancel-safe.
        let resp = tokio::select! {
            biased;
            uplink = uplink_rx.recv() => {
                let Some(payload) = uplink else { break; };
                // Active uplink — reset the empty-poll backoff so the
                // next inbound poll happens promptly.
                backoff = UDP_INITIAL_POLL_DELAY;
                match mux.udp_data(&sid, payload).await {
                    Ok(r) => r,
                    Err(e) => {
                        tracing::debug!("udp data {} failed: {}", sid, e);
                        break;
                    }
                }
            }
            _ = tokio::time::sleep(backoff) => {
                match mux.udp_data(&sid, Vec::new()).await {
                    Ok(r) => r,
                    Err(e) => {
                        tracing::debug!("udp poll {} failed: {}", sid, e);
                        break;
                    }
                }
            }
        };
        if resp.e.is_some() || resp.eof.unwrap_or(false) {
            break;
        }
        let got_pkts = resp.pkts.as_ref().map(|p| !p.is_empty()).unwrap_or(false);
        if got_pkts {
            send_udp_response_packets(&udp, client_addr, &target, &resp).await;
            backoff = UDP_INITIAL_POLL_DELAY;
        } else {
            // Empty poll — back off so an idle destination doesn't
            // monopolize batch slots.
            backoff = (backoff * 2).min(UDP_MAX_POLL_DELAY);
        }
    }
    // Be polite even if the session is already gone server-side; the
    // tunnel-node tolerates close on an unknown sid.
    mux.close_session(&sid).await;
}

async fn send_udp_response_packets(
    udp: &UdpSocket,
    client_addr: SocketAddr,
    target: &SocksUdpTarget,
    resp: &crate::domain_fronter::TunnelResponse,
) {
    let packets = match decode_udp_packets(resp) {
        Ok(packets) => packets,
        Err(e) => {
            tracing::debug!("{}", e);
            return;
        }
    };
    for packet in packets {
        let framed = build_socks5_udp_packet(target, &packet);
        if let Err(e) = udp.send_to(&framed, client_addr).await {
            // Errors here mean the local socket can't reach the SOCKS5
            // client (ENETUNREACH, EHOSTDOWN, etc.). Surface at debug
            // so a "my UDP traffic isn't coming back" report has
            // something to grep for; volume is bounded by what we'd
            // have delivered anyway.
            tracing::debug!(
                "udp send to client {} failed for {}:{}: {}",
                client_addr,
                target.host,
                target.port,
                e,
            );
        }
    }
}

async fn write_socks5_reply(
    sock: &mut TcpStream,
    rep: u8,
    addr: Option<SocketAddr>,
) -> std::io::Result<()> {
    let mut out = vec![0x05, rep, 0x00];
    match addr {
        Some(SocketAddr::V4(v4)) => {
            out.push(0x01);
            out.extend_from_slice(&v4.ip().octets());
            out.extend_from_slice(&v4.port().to_be_bytes());
        }
        Some(SocketAddr::V6(v6)) => {
            out.push(0x04);
            out.extend_from_slice(&v6.ip().octets());
            out.extend_from_slice(&v6.port().to_be_bytes());
        }
        None => {
            out.push(0x01);
            out.extend_from_slice(&[0, 0, 0, 0]);
            out.extend_from_slice(&0u16.to_be_bytes());
        }
    }
    sock.write_all(&out).await?;
    sock.flush().await
}

fn parse_socks5_udp_packet(buf: &[u8]) -> Option<(SocksUdpTarget, &[u8])> {
    if buf.len() < 4 || buf[0] != 0 || buf[1] != 0 || buf[2] != 0 {
        return None;
    }
    let atyp = buf[3];
    let mut pos = 4usize;
    let (host, addr) = match atyp {
        0x01 => {
            if buf.len() < pos + 4 + 2 {
                return None;
            }
            let addr = buf[pos..pos + 4].to_vec();
            pos += 4;
            let ip = std::net::Ipv4Addr::new(addr[0], addr[1], addr[2], addr[3]);
            (ip.to_string(), addr)
        }
        0x03 => {
            if buf.len() < pos + 1 {
                return None;
            }
            let len = buf[pos] as usize;
            pos += 1;
            if len == 0 || buf.len() < pos + len + 2 {
                return None;
            }
            let addr = buf[pos..pos + len].to_vec();
            pos += len;
            // Reject non-UTF-8 hostnames at the parser. Lossy decoding
            // would forward U+FFFD into DNS and trigger an opaque
            // NXDOMAIN — failing fast here gives us a clean parse-level
            // drop that the test suite can assert on.
            let host = std::str::from_utf8(&addr).ok()?.to_owned();
            (host, addr)
        }
        0x04 => {
            if buf.len() < pos + 16 + 2 {
                return None;
            }
            let addr = buf[pos..pos + 16].to_vec();
            pos += 16;
            let mut octets = [0u8; 16];
            octets.copy_from_slice(&addr);
            (std::net::Ipv6Addr::from(octets).to_string(), addr)
        }
        _ => return None,
    };
    let port = u16::from_be_bytes([buf[pos], buf[pos + 1]]);
    pos += 2;
    Some((
        SocksUdpTarget {
            host,
            port,
            atyp,
            addr,
        },
        &buf[pos..],
    ))
}

fn build_socks5_udp_packet(target: &SocksUdpTarget, payload: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(4 + target.addr.len() + 2 + payload.len() + 1);
    out.extend_from_slice(&[0, 0, 0, target.atyp]);
    match target.atyp {
        0x03 => {
            out.push(target.addr.len() as u8);
            out.extend_from_slice(&target.addr);
        }
        _ => out.extend_from_slice(&target.addr),
    }
    out.extend_from_slice(&target.port.to_be_bytes());
    out.extend_from_slice(payload);
    out
}
// ---------- Smart dispatch (used by both HTTP CONNECT and SOCKS5) ----------

fn should_use_sni_rewrite(
    hosts: &std::collections::HashMap<String, String>,
    host: &str,
    port: u16,
    youtube_via_relay: bool,
) -> bool {
    // The SNI-rewrite path expects TLS from the client: it accepts inbound
    // TLS, then opens a second TLS connection to the Google edge with a front
    // SNI. Auto-forcing that path for non-TLS ports (for example a SOCKS5
    // CONNECT to google.com:80) makes the proxy wait for a ClientHello that
    // will never arrive.
    //
    // youtube_via_relay=true removes YouTube suffixes from the match so
    // YouTube traffic falls through to the Apps Script relay path instead
    // of the SNI-rewrite tunnel. An explicit hosts override still wins
    // over the config toggle.
    port == 443
        && (matches_sni_rewrite(host, youtube_via_relay) || hosts_override(hosts, host).is_some())
}

async fn dispatch_tunnel(
    sock: TcpStream,
    host: String,
    port: u16,
    relay: Option<Arc<RelayTransport>>,
    mitm: Arc<Mutex<MitmCertManager>>,
    rewrite_ctx: Arc<RewriteCtx>,
    tunnel_mux: Option<Arc<TunnelMux>>,
) -> std::io::Result<()> {
    let matches_pt = matches_passthrough(&host, &rewrite_ctx.passthrough_hosts);
    let matches_rewrite = should_use_sni_rewrite(
        &rewrite_ctx.hosts,
        &host,
        port,
        rewrite_ctx.youtube_via_relay,
    );

    if rewrite_ctx.bypass_doh
        && port == 443
        && matches_doh_host(&host, &rewrite_ctx.bypass_doh_hosts)
    {
        let via = rewrite_ctx.upstream_socks5.as_deref();
        tracing::info!(
            "dispatch {}:{} -> raw-tcp ({}) (doh bypass)",
            host,
            port,
            via.unwrap_or("direct")
        );
        plain_tcp_passthrough(sock, &host, port, via).await;
        return Ok(());
    }

    if port == 443 {
        let group_match = match_fronting_group(&host, &rewrite_ctx.fronting_groups).map(Arc::clone);
        if let Some(group) = group_match {
            tracing::info!(
                "dispatch {}:{} -> sni-rewrite tunnel (fronting group '{}', edge {} sni={})",
                host,
                port,
                group.name,
                group.ip,
                group.sni
            );
            return do_sni_rewrite_tunnel_from_tcp(
                sock,
                &host,
                port,
                mitm,
                rewrite_ctx,
                Some(group),
            )
            .await;
        }
    }

    // Peek at first bytes only when needed.
    let mut peek_buf = [0u8; 8];
    let peek_n = match tokio::time::timeout(
        std::time::Duration::from_millis(300),
        sock.peek(&mut peek_buf),
    )
    .await
    {
        Ok(Ok(n)) => n,
        Ok(Err(_)) => return Ok(()),
        Err(_) => {
            // Client silent: likely a server-first protocol.
            let via = rewrite_ctx.upstream_socks5.as_deref();
            tracing::info!(
                "dispatch {}:{} -> raw-tcp ({}) (client silent, likely server-first)",
                host,
                port,
                via.unwrap_or("direct")
            );
            plain_tcp_passthrough(sock, &host, port, via).await;
            return Ok(());
        }
    };
    let decision = decide_route(
        rewrite_ctx.mode,
        &host,
        port,
        rewrite_ctx.youtube_via_relay,
        matches_pt,
        matches_rewrite,
        forced_route_for_host(rewrite_ctx.mode, &host, &rewrite_ctx.domain_overrides),
        if peek_n > 0 { Some(peek_buf[0]) } else { None },
        peek_n > 0 && looks_like_http(&peek_buf[..peek_n]),
    );

    match decision {
        RouteDecision::Passthrough => {
            let via = rewrite_ctx.upstream_socks5.as_deref();
            tracing::info!(
                "dispatch {}:{} -> raw-tcp ({})",
                host,
                port,
                via.unwrap_or("direct")
            );
            plain_tcp_passthrough(sock, &host, port, via).await;
            Ok(())
        }
        RouteDecision::FullTunnel => {
            let mux = match tunnel_mux {
                Some(m) => m,
                None => {
                    tracing::error!(
                        "dispatch {}:{} -> full mode but no tunnel mux (should not happen)",
                        host,
                        port
                    );
                    return Ok(());
                }
            };
            tracing::info!("dispatch {}:{} -> full tunnel (via batch mux)", host, port);
            crate::tunnel_client::tunnel_connection(sock, &host, port, &mux).await?;
            Ok(())
        }
        RouteDecision::SniRewrite => {
            tracing::info!(
                "dispatch {}:{} -> sni-rewrite tunnel (Google edge direct)",
                host,
                port
            );
            do_sni_rewrite_tunnel_from_tcp(sock, &host, port, mitm, rewrite_ctx, None).await
        }
        RouteDecision::MitmRelayTls => {
            let relay = match relay {
                Some(r) => r,
                None => {
                    tracing::error!(
                        "dispatch {}:{} -> raw-tcp (unexpected: relay mode with no transport)",
                        host,
                        port
                    );
                    plain_tcp_passthrough(
                        sock,
                        &host,
                        port,
                        rewrite_ctx.upstream_socks5.as_deref(),
                    )
                    .await;
                    return Ok(());
                }
            };
            tracing::info!(
                "dispatch {}:{} -> MITM + {} relay (TLS)",
                host,
                port,
                relay.label()
            );
            run_mitm_then_relay(sock, &host, port, mitm, &relay).await;
            Ok(())
        }
        RouteDecision::RelayPlainHttp { scheme } => {
            let relay = match relay {
                Some(r) => r,
                None => {
                    plain_tcp_passthrough(
                        sock,
                        &host,
                        port,
                        rewrite_ctx.upstream_socks5.as_deref(),
                    )
                    .await;
                    return Ok(());
                }
            };
            tracing::info!(
                "dispatch {}:{} -> {} relay (plain HTTP, scheme={})",
                host,
                port,
                relay.label(),
                scheme
            );
            relay_http_stream_raw(sock, &host, port, scheme, &relay).await;
            Ok(())
        }
    }
}

// ---------- Plain TCP passthrough ----------

async fn plain_tcp_passthrough(
    mut sock: TcpStream,
    host: &str,
    port: u16,
    upstream_socks5: Option<&str>,
) {
    let target_host = host.trim_start_matches('[').trim_end_matches(']');
    // Shorter connect timeout for IP literals (4s vs 10s for hostnames).
    // When the target is an IP (i.e.
    // a raw Telegram DC, or an IP someone hardcoded), and that route is
    // DPI-dropped, the client speeds up its own DC-rotation / fallback if
    // we fail fast. Ten seconds of "waiting for a dead IP" translates
    // directly into Telegram's 10s-per-DC rotation delay — users see the
    // app sit on "connecting..." for nearly a minute as it walks through
    // DC1, DC2, DC3. At 4s we cut that in roughly half.
    // Hostnames still get 10s because DNS + first-hop TCP genuinely can
    // take that long on flaky links, and the resolver fallbacks already
    // trim the worst case.
    let connect_timeout = if looks_like_ip(target_host) {
        std::time::Duration::from_secs(4)
    } else {
        std::time::Duration::from_secs(10)
    };
    let upstream = if let Some(proxy) = upstream_socks5 {
        match socks5_connect_via(proxy, target_host, port).await {
            Ok(s) => {
                tracing::info!("tcp via upstream-socks5 {} -> {}:{}", proxy, host, port);
                s
            }
            Err(e) => {
                tracing::warn!(
                    "upstream-socks5 {} -> {}:{} failed: {} (falling back to direct)",
                    proxy,
                    host,
                    port,
                    e
                );
                match tokio::time::timeout(connect_timeout, TcpStream::connect((target_host, port)))
                    .await
                {
                    Ok(Ok(s)) => s,
                    _ => return,
                }
            }
        }
    } else {
        match tokio::time::timeout(connect_timeout, TcpStream::connect((target_host, port))).await {
            Ok(Ok(s)) => {
                tracing::info!("plain-tcp passthrough -> {}:{}", host, port);
                s
            }
            Ok(Err(e)) => {
                tracing::debug!("plain-tcp connect {}:{} failed: {}", host, port, e);
                return;
            }
            Err(_) => {
                tracing::debug!(
                    "plain-tcp connect {}:{} timeout (likely blocked; client should rotate)",
                    host,
                    port
                );
                return;
            }
        }
    };
    let _ = upstream.set_nodelay(true);
    let (mut ar, mut aw) = sock.split();
    let (mut br, mut bw) = {
        let (r, w) = upstream.into_split();
        (r, w)
    };
    let t1 = tokio::io::copy(&mut ar, &mut bw);
    let t2 = tokio::io::copy(&mut br, &mut aw);
    tokio::select! {
        _ = t1 => {}
        _ = t2 => {}
    }
}

/// Open a TCP stream to `(host, port)` through an upstream SOCKS5 proxy
/// (no-auth only). Returns the connected stream after SOCKS5 negotiation.
async fn socks5_connect_via(proxy: &str, host: &str, port: u16) -> std::io::Result<TcpStream> {
    use tokio::io::AsyncReadExt;
    use tokio::io::AsyncWriteExt;
    let mut s = tokio::time::timeout(std::time::Duration::from_secs(5), TcpStream::connect(proxy))
        .await
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::TimedOut, "connect timeout"))??;
    let _ = s.set_nodelay(true);

    // Greeting: VER=5, NMETHODS=1, METHOD=no-auth
    s.write_all(&[0x05, 0x01, 0x00]).await?;
    let mut reply = [0u8; 2];
    s.read_exact(&mut reply).await?;
    if reply[0] != 0x05 || reply[1] != 0x00 {
        return Err(std::io::Error::other(format!(
            "socks5 greet rejected: {:?}",
            reply
        )));
    }

    // CONNECT request: VER=5, CMD=1, RSV=0, ATYP=3 (domain) | 1 (IPv4) | 4 (IPv6)
    let mut req: Vec<u8> = Vec::with_capacity(8 + host.len());
    req.extend_from_slice(&[0x05, 0x01, 0x00]);
    if let Ok(v4) = host.parse::<std::net::Ipv4Addr>() {
        req.push(0x01);
        req.extend_from_slice(&v4.octets());
    } else if let Ok(v6) = host.parse::<std::net::Ipv6Addr>() {
        req.push(0x04);
        req.extend_from_slice(&v6.octets());
    } else {
        let hb = host.as_bytes();
        if hb.len() > 255 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "hostname > 255",
            ));
        }
        req.push(0x03);
        req.push(hb.len() as u8);
        req.extend_from_slice(hb);
    }
    req.extend_from_slice(&port.to_be_bytes());
    s.write_all(&req).await?;

    // Reply header: VER, REP, RSV, ATYP, BND.ADDR, BND.PORT
    let mut head = [0u8; 4];
    s.read_exact(&mut head).await?;
    if head[0] != 0x05 || head[1] != 0x00 {
        return Err(std::io::Error::other(format!(
            "socks5 connect rejected rep=0x{:02x}",
            head[1]
        )));
    }
    // Skip BND.ADDR + BND.PORT.
    match head[3] {
        0x01 => {
            let mut b = [0u8; 4 + 2];
            s.read_exact(&mut b).await?;
        }
        0x04 => {
            let mut b = [0u8; 16 + 2];
            s.read_exact(&mut b).await?;
        }
        0x03 => {
            let mut len = [0u8; 1];
            s.read_exact(&mut len).await?;
            let mut name = vec![0u8; len[0] as usize + 2];
            s.read_exact(&mut name).await?;
        }
        other => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("socks5 bad ATYP in reply: {}", other),
            ));
        }
    }
    Ok(s)
}

fn looks_like_http(first_bytes: &[u8]) -> bool {
    // Cheap sniff: must start with an ASCII HTTP method token followed by a space.
    for m in [
        "GET ", "POST ", "PUT ", "HEAD ", "DELETE ", "PATCH ", "OPTIONS ", "CONNECT ", "TRACE ",
    ] {
        if first_bytes.starts_with(m.as_bytes()) {
            return true;
        }
    }
    false
}

/// Read an HTTP head (request line + headers) up to the first \r\n\r\n.
/// Returns (head_bytes, leftover_after_head). The leftover may contain part
/// of the request body already received.
enum HeadReadResult {
    Got { head: Vec<u8>, leftover: Vec<u8> },
    Closed,
    Oversized,
}

async fn read_http_head(sock: &mut TcpStream) -> std::io::Result<HeadReadResult> {
    let mut buf = Vec::with_capacity(4096);
    let mut tmp = [0u8; 4096];
    loop {
        let n = sock.read(&mut tmp).await?;
        if n == 0 {
            return if buf.is_empty() {
                Ok(HeadReadResult::Closed)
            } else {
                Err(std::io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    "EOF mid-header",
                ))
            };
        }
        buf.extend_from_slice(&tmp[..n]);
        if let Some(pos) = find_headers_end(&buf) {
            let head = buf[..pos].to_vec();
            let leftover = buf[pos..].to_vec();
            return Ok(HeadReadResult::Got { head, leftover });
        }
        if buf.len() > MAX_HTTP_HEADER_BYTES {
            return Ok(HeadReadResult::Oversized);
        }
    }
}

fn find_headers_end(buf: &[u8]) -> Option<usize> {
    buf.windows(4).position(|w| w == b"\r\n\r\n").map(|p| p + 4)
}

type HttpHeaders = Vec<(String, String)>;
type RequestHead = (String, String, String, HttpHeaders);

fn parse_request_head(head: &[u8]) -> Option<RequestHead> {
    let s = std::str::from_utf8(head).ok()?;
    let mut lines = s.split("\r\n");
    let first = lines.next()?;
    let mut parts = first.splitn(3, ' ');
    let method = parts.next()?.to_string();
    let target = parts.next()?.to_string();
    let version = parts.next().unwrap_or("HTTP/1.1").to_string();

    if !is_valid_http_method(&method) {
        return None;
    }

    let mut headers = Vec::new();
    for l in lines {
        if l.is_empty() {
            break;
        }
        if let Some((k, v)) = l.split_once(':') {
            headers.push((k.trim().to_string(), v.trim().to_string()));
        }
    }
    Some((method, target, version, headers))
}

fn is_valid_http_method(m: &str) -> bool {
    matches!(
        m,
        "GET" | "POST" | "PUT" | "DELETE" | "HEAD" | "OPTIONS" | "PATCH" | "TRACE" | "CONNECT"
    )
}

// ---------- CONNECT handling ----------

async fn run_mitm_then_relay(
    sock: TcpStream,
    host: &str,
    port: u16,
    mitm: Arc<Mutex<MitmCertManager>>,
    relay: &RelayTransport,
) {
    // Peek the TLS ClientHello BEFORE minting the MITM cert. When the client
    // resolves the hostname itself (DoH in Chrome/Firefox) and hands us a raw
    // IP via SOCKS5, the only place the real hostname lives is the SNI. If we
    // mint a cert for the IP, Chrome rejects with ERR_CERT_COMMON_NAME_INVALID
    // — the IP isn't in the cert's SAN. Reading SNI up front and using it as
    // both the cert subject and the upstream Host for the Apps Script relay
    // is what unblocks Cloudflare-fronted sites and any browser on Android
    // where DoH is the default.
    let start = match LazyConfigAcceptor::new(Acceptor::default(), sock).await {
        Ok(s) => s,
        Err(e) => {
            tracing::debug!("TLS ClientHello peek failed for {}: {}", host, e);
            return;
        }
    };

    let sni_hostname = start.client_hello().server_name().map(String::from);

    // Effective host: SNI when present and looks like a hostname (anything
    // other than a bare IPv4 literal — IP SNIs exist for weird setups but
    // minting a cert for them still triggers ERR_CERT_COMMON_NAME_INVALID,
    // so we fall through to the raw host in that case).
    let effective_host: String = match sni_hostname.as_deref() {
        Some(s) if !looks_like_ip(s) && !s.is_empty() => s.to_string(),
        _ => host.to_string(),
    };

    tracing::info!(
        "MITM TLS -> {}:{} (socks_host={}, sni={})",
        effective_host,
        port,
        host,
        sni_hostname.as_deref().unwrap_or("<none>"),
    );

    let server_config = {
        let mut m = mitm.lock().await;
        match m.get_server_config(&effective_host) {
            Ok(c) => c,
            Err(e) => {
                tracing::error!("cert gen failed for {}: {}", effective_host, e);
                return;
            }
        }
    };

    let mut tls = match start.into_stream(server_config).await {
        Ok(t) => t,
        Err(e) => {
            tracing::debug!("TLS accept failed for {}: {}", effective_host, e);
            return;
        }
    };

    // Keep-alive loop: read HTTP requests from the decrypted stream. Pass the
    // SNI-derived hostname so the JSON relay fetches
    // `https://<real hostname>/path` instead of `https://<raw IP>/path` — the
    // latter would produce an IP-in-Host request that Cloudflare/etc. reject
    // outright.
    loop {
        match handle_mitm_request(&mut tls, &effective_host, port, relay, "https").await {
            Ok(true) => continue,
            Ok(false) => break,
            Err(e) => {
                tracing::debug!("MITM handler error for {}: {}", effective_host, e);
                break;
            }
        }
    }
}

/// True if `s` parses as an IPv4 or IPv6 literal. Used to decide whether
/// a string is a hostname we should mint a MITM leaf cert for — IP SANs
/// need their own cert extension and we don't bother emitting those,
/// so fall back to the SOCKS5-provided target in that case.
fn looks_like_ip(s: &str) -> bool {
    s.parse::<std::net::IpAddr>().is_ok()
}

// ---------- Plain HTTP relay on a raw TCP stream (port 80 targets) ----------

async fn relay_http_stream_raw(
    mut sock: TcpStream,
    host: &str,
    port: u16,
    scheme: &str,
    relay: &RelayTransport,
) {
    loop {
        match handle_mitm_request(&mut sock, host, port, relay, scheme).await {
            Ok(true) => continue,
            Ok(false) => break,
            Err(e) => {
                tracing::debug!("http relay error for {}: {}", host, e);
                break;
            }
        }
    }
}

async fn do_sni_rewrite_tunnel_from_tcp(
    sock: TcpStream,
    host: &str,
    port: u16,
    mitm: Arc<Mutex<MitmCertManager>>,
    rewrite_ctx: Arc<RewriteCtx>,
    group: Option<Arc<FrontingGroupResolved>>,
) -> std::io::Result<()> {
    let (target_ip, outbound_sni, server_name) = match &group {
        Some(group) => (
            group.ip.clone(),
            group.sni.clone(),
            group.server_name.clone(),
        ),
        None => {
            let ip = hosts_override(&rewrite_ctx.hosts, host)
                .map(|s| s.to_string())
                .unwrap_or_else(|| rewrite_ctx.google_ip.clone());
            let sni = rewrite_ctx.front_domain.clone();
            let server_name = match ServerName::try_from(sni.clone()) {
                Ok(n) => n,
                Err(e) => {
                    tracing::error!("invalid front_domain '{}': {}", sni, e);
                    return Ok(());
                }
            };
            (ip, sni, server_name)
        }
    };

    tracing::info!(
        "SNI-rewrite tunnel -> {}:{} via {} (outbound SNI={})",
        host,
        port,
        target_ip,
        outbound_sni
    );

    // Accept browser TLS with a cert we sign for `host`.
    let server_config = {
        let mut m = mitm.lock().await;
        match m.get_server_config(host) {
            Ok(c) => c,
            Err(e) => {
                tracing::error!("cert gen failed for {}: {}", host, e);
                return Ok(());
            }
        }
    };
    let inbound = match TlsAcceptor::from(server_config).accept(sock).await {
        Ok(t) => t,
        Err(e) => {
            tracing::debug!("inbound TLS accept failed for {}: {}", host, e);
            return Ok(());
        }
    };

    // Open outbound TLS to google_ip with SNI=front_domain.
    let upstream_tcp = match tokio::time::timeout(
        std::time::Duration::from_secs(10),
        TcpStream::connect((target_ip.as_str(), port)),
    )
    .await
    {
        Ok(Ok(s)) => {
            let _ = s.set_nodelay(true);
            s
        }
        Ok(Err(e)) => {
            tracing::debug!("upstream connect failed for {}: {}", host, e);
            return Ok(());
        }
        Err(_) => {
            tracing::debug!("upstream connect timeout for {}", host);
            return Ok(());
        }
    };
    let _ = upstream_tcp.set_nodelay(true);

    let outbound = match rewrite_ctx
        .tls_connector
        .connect(server_name, upstream_tcp)
        .await
    {
        Ok(t) => t,
        Err(e) => {
            tracing::debug!("outbound TLS connect failed for {}: {}", host, e);
            return Ok(());
        }
    };

    // Bridge decrypted bytes between the two TLS streams.
    let (mut ir, mut iw) = tokio::io::split(inbound);
    let (mut or, mut ow) = tokio::io::split(outbound);
    let client_to_server = async { tokio::io::copy(&mut ir, &mut ow).await };
    let server_to_client = async { tokio::io::copy(&mut or, &mut iw).await };
    tokio::select! {
        _ = client_to_server => {}
        _ = server_to_client => {}
    }
    Ok(())
}

#[derive(Debug)]
struct NoVerify;

impl ServerCertVerifier for NoVerify {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: UnixTime,
    ) -> Result<ServerCertVerified, tokio_rustls::rustls::Error> {
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, tokio_rustls::rustls::Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, tokio_rustls::rustls::Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        vec![
            SignatureScheme::RSA_PKCS1_SHA256,
            SignatureScheme::RSA_PKCS1_SHA384,
            SignatureScheme::RSA_PKCS1_SHA512,
            SignatureScheme::ECDSA_NISTP256_SHA256,
            SignatureScheme::ECDSA_NISTP384_SHA384,
            SignatureScheme::RSA_PSS_SHA256,
            SignatureScheme::RSA_PSS_SHA384,
            SignatureScheme::RSA_PSS_SHA512,
            SignatureScheme::ED25519,
        ]
    }
}

fn parse_host_port(target: &str) -> (String, u16) {
    if let Some((h, p)) = target.rsplit_once(':') {
        let port: u16 = p.parse().unwrap_or(443);
        (h.to_string(), port)
    } else {
        (target.to_string(), 443)
    }
}

impl RewriteCtx {
    fn listen_host_is_lan_bound(&self) -> bool {
        let h = self.listen_host.trim();
        h == "0.0.0.0" || h == "::"
    }
}

fn lan_auth_configured(ctx: &RewriteCtx) -> bool {
    !ctx.lan_token.as_deref().unwrap_or("").trim().is_empty()
        || ctx
            .lan_allowlist
            .as_ref()
            .map(|v| !v.is_empty())
            .unwrap_or(false)
}

fn lan_peer_allowed(sock: &TcpStream, ctx: &RewriteCtx) -> bool {
    let Some(list) = ctx.lan_allowlist.as_ref() else {
        return true;
    };
    if list.is_empty() {
        return true;
    }
    let Ok(peer) = sock.peer_addr() else {
        return false;
    };
    let ip = peer.ip();
    list.iter().any(|e| lan_allowlist_entry_matches(e, ip))
}

fn lan_socks_allowed(sock: &TcpStream, ctx: &RewriteCtx) -> bool {
    if !lan_auth_configured(ctx) {
        return true;
    }
    let token_set = !ctx.lan_token.as_deref().unwrap_or("").trim().is_empty();
    let allowlist = ctx.lan_allowlist.as_ref();
    let allowlist_nonempty = allowlist.map(|v| !v.is_empty()).unwrap_or(false);

    // SOCKS5 has no header preface where `lan_token` can be supplied. If a
    // LAN-bound deployment sets a token but no allowlist, fail closed for
    // SOCKS5 instead of silently exposing it to the whole LAN.
    if token_set && !allowlist_nonempty {
        return false;
    }

    // If an allowlist is configured (with or without a token), enforce it for
    // SOCKS5 the same way we do for HTTP.
    if allowlist_nonempty {
        return lan_peer_allowed(sock, ctx);
    }

    true
}

fn lan_auth_ok(sock: &TcpStream, headers: &[(String, String)], ctx: &RewriteCtx) -> bool {
    // Allowlist: exact IPs and CIDR ranges.
    if !lan_peer_allowed(sock, ctx) {
        return false;
    }
    // Token check.
    if let Some(tok) = ctx.lan_token.as_deref() {
        let tok = tok.trim();
        if !tok.is_empty() {
            let got = headers
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("x-mhrv-f-token"))
                .map(|(_, v)| v.trim())
                .unwrap_or("");
            if got != tok {
                return false;
            }
        }
    }
    true
}

fn lan_allowlist_entry_matches(entry: &str, peer: IpAddr) -> bool {
    let entry = entry.trim();
    if entry.is_empty() {
        return false;
    }
    if let Ok(ip) = entry.parse::<IpAddr>() {
        return ip == peer;
    }
    let Some((network, prefix)) = entry.split_once('/') else {
        return false;
    };
    let Ok(network) = network.trim().parse::<IpAddr>() else {
        return false;
    };
    let Ok(prefix) = prefix.trim().parse::<u8>() else {
        return false;
    };
    ip_in_cidr(peer, network, prefix)
}

fn ip_in_cidr(peer: IpAddr, network: IpAddr, prefix: u8) -> bool {
    match (peer, network) {
        (IpAddr::V4(peer), IpAddr::V4(network)) if prefix <= 32 => {
            let mask = if prefix == 0 {
                0
            } else {
                u32::MAX << (32 - prefix)
            };
            (u32::from(peer) & mask) == (u32::from(network) & mask)
        }
        (IpAddr::V6(peer), IpAddr::V6(network)) if prefix <= 128 => {
            let mask = if prefix == 0 {
                0
            } else {
                u128::MAX << (128 - prefix)
            };
            (u128::from(peer) & mask) == (u128::from(network) & mask)
        }
        _ => false,
    }
}

async fn handle_mitm_request<S>(
    stream: &mut S,
    host: &str,
    port: u16,
    relay: &RelayTransport,
    scheme: &str,
) -> std::io::Result<bool>
where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
{
    let (head, leftover) = match read_http_head_io(stream).await? {
        HeadReadResult::Got { head, leftover } => (head, leftover),
        HeadReadResult::Closed => return Ok(false),
        HeadReadResult::Oversized => {
            tracing::warn!(
                "MITM header block exceeds {} bytes; closing ({}:{})",
                MAX_HTTP_HEADER_BYTES,
                host,
                port
            );
            let _ = stream
                .write_all(
                    b"HTTP/1.1 431 Request Header Fields Too Large\r\n\
                      Connection: close\r\n\
                      Content-Length: 0\r\n\r\n",
                )
                .await;
            let _ = stream.flush().await;
            return Ok(false);
        }
    };

    let (method, path, _version, headers) = match parse_request_head(&head) {
        Some(v) => v,
        None => return Ok(false),
    };

    let body = read_body(stream, &leftover, &headers).await?;

    // ── Per-host URL fix-ups ──────────────────────────────────────────
    // x.com's GraphQL endpoints concatenate three huge JSON blobs into
    // the query string: `?variables=<json>&features=<json>&fieldToggles=<json>`.
    // The combined URL regularly exceeds Apps Script's URL length limit
    // (Apps Script returns "بیش از حد مجاز: طول نشانی وب URLFetch" /
    // "URLFetch URL length exceeded"). The `variables=` portion alone
    // is enough for x.com to serve the timeline — `features` /
    // `fieldToggles` are client-capability hints it tolerates being
    // absent. Truncating at the first `&` after `?variables=` ships a
    // working request that fits under the limit.
    //
    // Host matcher: browsers actually hit `www.x.com` (and sometimes
    // `api.x.com`), not bare `x.com`. The original check only matched
    // `x.com` exactly, so real traffic flew past the rewrite until
    // we matched the real `www.` / `api.` Host headers browsers send. Match every
    // subdomain of x.com here.
    let host_lower = host.to_ascii_lowercase();
    let is_x_com = host_lower == "x.com"
        || host_lower.ends_with(".x.com")
        || host_lower == "twitter.com"
        || host_lower.ends_with(".twitter.com");
    let path = if is_x_com && path.starts_with("/i/api/graphql/") && path.contains("?variables=") {
        match path.split_once('&') {
            Some((short, _)) => {
                tracing::debug!(
                    "x.com graphql URL truncated: {} chars -> {}",
                    path.len(),
                    short.len()
                );
                short.to_string()
            }
            None => path,
        }
    } else {
        path
    };

    let default_port = if scheme == "https" { 443 } else { 80 };
    let url = if port == default_port {
        format!("{}://{}{}", scheme, host, path)
    } else {
        format!("{}://{}:{}{}", scheme, host, port, path)
    };

    // Short-circuit CORS preflight at the MITM boundary.
    //
    // Apps Script's UrlFetchApp.fetch() only accepts methods {get, delete,
    // patch, post, put} — OPTIONS triggers the Swedish-localized
    // "Ett attribut med ogiltigt värde har angetts: method" error, which
    // kills every XHR/fetch preflight and is the root cause of "JS doesn't
    // load" on sites like Discord, Yahoo finance widgets, etc.
    //
    // Answering the preflight ourselves is safe: we already terminate the
    // TLS for the browser (we minted the cert), so it's legitimate for us
    // to own the wire-level conversation. CORS is a browser-side
    // protection, not a network security one — responding 204 with
    // permissive ACL headers just tells the browser the *subsequent* real
    // request is allowed, and that real request still goes through the
    // Apps Script relay where the origin server gets final say on content.
    // The origin header is echoed (not "*") so Credentials-true responses
    // stay spec-valid.
    if method.eq_ignore_ascii_case("OPTIONS") {
        tracing::info!("preflight 204 {} (short-circuit, no relay)", url);
        let origin = header_value(&headers, "origin").unwrap_or("*");
        let acrm = header_value(&headers, "access-control-request-method")
            .unwrap_or("GET, POST, PUT, DELETE, PATCH, OPTIONS, HEAD");
        let acrh = header_value(&headers, "access-control-request-headers").unwrap_or("*");
        let resp = format!(
            "HTTP/1.1 204 No Content\r\n\
             Access-Control-Allow-Origin: {origin}\r\n\
             Access-Control-Allow-Methods: {acrm}\r\n\
             Access-Control-Allow-Headers: {acrh}\r\n\
             Access-Control-Allow-Credentials: true\r\n\
             Access-Control-Max-Age: 86400\r\n\
             Vary: Origin, Access-Control-Request-Method, Access-Control-Request-Headers\r\n\
             Content-Length: 0\r\n\
             \r\n",
        );
        stream.write_all(resp.as_bytes()).await?;
        stream.flush().await?;
        let connection_close = headers
            .iter()
            .any(|(k, v)| k.eq_ignore_ascii_case("connection") && v.eq_ignore_ascii_case("close"));
        return Ok(!connection_close);
    }

    tracing::info!("relay {} {}", method, url);

    // For GETs without a body, take the range-parallel path — probes
    // with `Range: bytes=0-<chunk>`, and if the origin supports ranges,
    // fetches the rest in parallel 256 KB chunks. This is what lets
    // YouTube video streaming / gvt1.com Chrome-updates / big static
    // files not stall waiting on one ~2s Apps Script call per MB.
    // Anything with a body (POST/PUT/PATCH) goes through the normal
    // relay path — range semantics on mutating requests are undefined
    // and would break form submissions.
    let response = if method.eq_ignore_ascii_case("GET") && body.is_empty() {
        relay
            .relay_parallel_range(&method, &url, &headers, &body)
            .await
    } else {
        relay.relay(&method, &url, &headers, &body).await
    };
    stream.write_all(&response).await?;
    stream.flush().await?;

    // Keep-alive unless the client asked to close.
    let connection_close = headers
        .iter()
        .any(|(k, v)| k.eq_ignore_ascii_case("connection") && v.eq_ignore_ascii_case("close"));
    Ok(!connection_close)
}

async fn read_http_head_io<S>(stream: &mut S) -> std::io::Result<HeadReadResult>
where
    S: tokio::io::AsyncRead + Unpin,
{
    let mut buf = Vec::with_capacity(4096);
    let mut tmp = [0u8; 4096];
    loop {
        let n = stream.read(&mut tmp).await?;
        if n == 0 {
            return if buf.is_empty() {
                Ok(HeadReadResult::Closed)
            } else {
                Err(std::io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    "EOF mid-header",
                ))
            };
        }
        buf.extend_from_slice(&tmp[..n]);
        if let Some(pos) = find_headers_end(&buf) {
            let head = buf[..pos].to_vec();
            let leftover = buf[pos..].to_vec();
            return Ok(HeadReadResult::Got { head, leftover });
        }
        if buf.len() > MAX_HTTP_HEADER_BYTES {
            return Ok(HeadReadResult::Oversized);
        }
    }
}

fn header_value<'a>(headers: &'a [(String, String)], name: &str) -> Option<&'a str> {
    headers
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case(name))
        .map(|(_, v)| v.as_str())
}

fn expects_100_continue(headers: &[(String, String)]) -> bool {
    header_value(headers, "expect")
        .map(|v| {
            v.split(',')
                .any(|part| part.trim().eq_ignore_ascii_case("100-continue"))
        })
        .unwrap_or(false)
}

fn invalid_body(msg: impl Into<String>) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidData, msg.into())
}

async fn read_body<S>(
    stream: &mut S,
    leftover: &[u8],
    headers: &[(String, String)],
) -> std::io::Result<Vec<u8>>
where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
{
    let transfer_encoding = header_value(headers, "transfer-encoding");
    let is_chunked = transfer_encoding
        .map(|v| {
            v.split(',')
                .any(|part| part.trim().eq_ignore_ascii_case("chunked"))
        })
        .unwrap_or(false);

    let content_length = match header_value(headers, "content-length") {
        Some(v) => Some(
            v.parse::<usize>()
                .map_err(|_| invalid_body(format!("invalid Content-Length: {}", v)))?,
        ),
        None => None,
    };
    if let Some(n) = content_length {
        if n > MAX_REQUEST_BODY_BYTES {
            return Err(invalid_body(format!(
                "request body too large: {} bytes (max {})",
                n, MAX_REQUEST_BODY_BYTES
            )));
        }
    }

    if transfer_encoding.is_some() && !is_chunked {
        return Err(invalid_body(format!(
            "unsupported Transfer-Encoding: {}",
            transfer_encoding.unwrap_or_default()
        )));
    }

    if is_chunked && content_length.is_some() {
        return Err(invalid_body(
            "both Transfer-Encoding: chunked and Content-Length are present",
        ));
    }

    if expects_100_continue(headers) && (is_chunked || content_length.is_some()) {
        stream.write_all(b"HTTP/1.1 100 Continue\r\n\r\n").await?;
        stream.flush().await?;
    }

    if is_chunked {
        return read_chunked_request_body(stream, leftover.to_vec()).await;
    }

    let Some(content_length) = content_length else {
        return Ok(Vec::new());
    };

    let mut body = Vec::with_capacity(content_length);
    body.extend_from_slice(&leftover[..leftover.len().min(content_length)]);
    let mut tmp = [0u8; 8192];
    while body.len() < content_length {
        let n = stream.read(&mut tmp).await?;
        if n == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "EOF mid-body",
            ));
        }
        let need = content_length - body.len();
        body.extend_from_slice(&tmp[..n.min(need)]);
    }
    Ok(body)
}

async fn read_chunked_request_body<S>(stream: &mut S, mut buf: Vec<u8>) -> std::io::Result<Vec<u8>>
where
    S: tokio::io::AsyncRead + Unpin,
{
    let mut out = Vec::new();
    let mut tmp = [0u8; 8192];

    loop {
        let line = read_crlf_line(stream, &mut buf, &mut tmp).await?;
        if line.is_empty() {
            continue;
        }

        let line_str = std::str::from_utf8(&line)
            .map_err(|_| invalid_body("non-utf8 chunk size line"))?
            .trim();
        let size_hex = line_str.split(';').next().unwrap_or("");
        let size = usize::from_str_radix(size_hex, 16)
            .map_err(|_| invalid_body(format!("bad chunk size '{}'", line_str)))?;

        if size == 0 {
            loop {
                let trailer = read_crlf_line(stream, &mut buf, &mut tmp).await?;
                if trailer.is_empty() {
                    return Ok(out);
                }
            }
        }

        fill_buffer(stream, &mut buf, &mut tmp, size + 2).await?;
        if &buf[size..size + 2] != b"\r\n" {
            return Err(invalid_body("chunk missing trailing CRLF"));
        }
        if out.len().saturating_add(size) > MAX_REQUEST_BODY_BYTES {
            return Err(invalid_body(format!(
                "chunked request body too large (max {})",
                MAX_REQUEST_BODY_BYTES
            )));
        }
        out.extend_from_slice(&buf[..size]);
        buf.drain(..size + 2);
    }
}

async fn read_crlf_line<S>(
    stream: &mut S,
    buf: &mut Vec<u8>,
    tmp: &mut [u8],
) -> std::io::Result<Vec<u8>>
where
    S: tokio::io::AsyncRead + Unpin,
{
    loop {
        if let Some(idx) = buf.windows(2).position(|w| w == b"\r\n") {
            let line = buf[..idx].to_vec();
            buf.drain(..idx + 2);
            return Ok(line);
        }
        let n = stream.read(tmp).await?;
        if n == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "EOF in chunked body",
            ));
        }
        buf.extend_from_slice(&tmp[..n]);
    }
}

async fn fill_buffer<S>(
    stream: &mut S,
    buf: &mut Vec<u8>,
    tmp: &mut [u8],
    want: usize,
) -> std::io::Result<()>
where
    S: tokio::io::AsyncRead + Unpin,
{
    while buf.len() < want {
        let n = stream.read(tmp).await?;
        if n == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "EOF in chunked body",
            ));
        }
        buf.extend_from_slice(&tmp[..n]);
    }
    Ok(())
}

// ---------- Plain HTTP proxy ----------

async fn do_plain_http(
    mut sock: TcpStream,
    head: &[u8],
    leftover: &[u8],
    relay: Arc<RelayTransport>,
) -> std::io::Result<()> {
    let (method, target, _version, headers) = parse_request_head(head)
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "bad request"))?;

    let body = read_body(&mut sock, leftover, &headers).await?;

    // Browser sends `GET http://example.com/path HTTP/1.1` on plain proxy.
    let url = if target.starts_with("http://") || target.starts_with("https://") {
        target.clone()
    } else {
        // Fallback: stitch Host header with path.
        let host = headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case("host"))
            .map(|(_, v)| v.clone())
            .unwrap_or_default();
        format!("http://{}{}", host, target)
    };

    tracing::info!("HTTP {} {}", method, url);
    // Plain HTTP proxy path — same range-parallel strategy as the
    // MITM-HTTPS path above. Large downloads on port 80 (package
    // mirrors, video poster streams, etc.) need the same acceleration
    // or the relay stalls per-chunk.
    let response = if method.eq_ignore_ascii_case("GET") && body.is_empty() {
        relay
            .relay_parallel_range(&method, &url, &headers, &body)
            .await
    } else {
        relay.relay(&method, &url, &headers, &body).await
    };
    sock.write_all(&response).await?;
    sock.flush().await?;
    Ok(())
}

/// Direct-mode plain-HTTP passthrough.
///
/// The CONNECT path already falls through to direct TCP for non-Google-edge
/// hosts in direct mode; this does the same for the absolute-form proxy
/// request line (`GET http://host/path HTTP/1.1`) so users don't get a 502
/// when typing a bare `http://...` URL.
async fn do_plain_http_passthrough(
    mut sock: TcpStream,
    head: &[u8],
    leftover: &[u8],
    rewrite_ctx: &RewriteCtx,
) -> std::io::Result<()> {
    let (method, target, version, headers) = match parse_request_head(head) {
        Some(v) => v,
        None => return Ok(()),
    };
    let (host, port, path) = match resolve_plain_http_target(&target, &headers) {
        Some(v) => v,
        None => {
            tracing::debug!("plain-http passthrough: cannot parse target {}", target);
            return Ok(());
        }
    };
    tracing::info!(
        "dispatch http {}:{} -> raw-tcp ({}) (direct mode: no relay)",
        host,
        port,
        rewrite_ctx.upstream_socks5.as_deref().unwrap_or("direct"),
    );

    // Rewrite request line to origin form and drop hop-by-hop headers.
    let mut rewritten = Vec::with_capacity(head.len());
    rewritten.extend_from_slice(method.as_bytes());
    rewritten.push(b' ');
    rewritten.extend_from_slice(path.as_bytes());
    rewritten.push(b' ');
    rewritten.extend_from_slice(version.as_bytes());
    rewritten.extend_from_slice(b"\r\n");
    for (k, v) in &headers {
        let kl = k.to_ascii_lowercase();
        if kl == "proxy-connection" || kl == "connection" || kl == "keep-alive" {
            continue;
        }
        rewritten.extend_from_slice(k.as_bytes());
        rewritten.extend_from_slice(b": ");
        rewritten.extend_from_slice(v.as_bytes());
        rewritten.extend_from_slice(b"\r\n");
    }
    // Force close so a keep-alive client can't pipeline a request for a
    // different host onto our spliced socket.
    rewritten.extend_from_slice(b"Connection: close\r\n\r\n");

    let target_host = host.trim_start_matches('[').trim_end_matches(']');
    let connect_timeout = if looks_like_ip(target_host) {
        std::time::Duration::from_secs(4)
    } else {
        std::time::Duration::from_secs(10)
    };

    let upstream = if let Some(proxy) = rewrite_ctx.upstream_socks5.as_deref() {
        match socks5_connect_via(proxy, target_host, port).await {
            Ok(s) => s,
            Err(e) => {
                tracing::warn!(
                    "upstream-socks5 {} -> {}:{} failed: {} (falling back to direct)",
                    proxy,
                    host,
                    port,
                    e
                );
                match tokio::time::timeout(connect_timeout, TcpStream::connect((target_host, port)))
                    .await
                {
                    Ok(Ok(s)) => s,
                    _ => return Ok(()),
                }
            }
        }
    } else {
        match tokio::time::timeout(connect_timeout, TcpStream::connect((target_host, port))).await {
            Ok(Ok(s)) => s,
            Ok(Err(e)) => {
                tracing::debug!("plain-http connect {}:{} failed: {}", host, port, e);
                return Ok(());
            }
            Err(_) => {
                tracing::debug!("plain-http connect {}:{} timeout", host, port);
                return Ok(());
            }
        }
    };

    let _ = upstream.set_nodelay(true);

    let (mut ar, mut aw) = sock.split();
    let (mut br, mut bw) = upstream.into_split();

    bw.write_all(&rewritten).await?;
    if !leftover.is_empty() {
        bw.write_all(leftover).await?;
    }

    let t1 = tokio::io::copy(&mut ar, &mut bw);
    let t2 = tokio::io::copy(&mut br, &mut aw);
    tokio::select! {
        _ = t1 => {}
        _ = t2 => {}
    }
    Ok(())
}

/// Parse the target of a plain-HTTP proxy request line into
/// `(host, port, origin-form-path)`.
///
/// Browsers send absolute form (`http://host[:port]/path`). We also accept:
/// - `https://` defensively (some clients do odd things)
/// - origin-form (`/path`) with a `Host:` header fallback
fn resolve_plain_http_target(
    target: &str,
    headers: &[(String, String)],
) -> Option<(String, u16, String)> {
    let (rest, default_port) = if let Some(r) = target.strip_prefix("http://") {
        (r, 80u16)
    } else if let Some(r) = target.strip_prefix("https://") {
        (r, 443u16)
    } else if target.starts_with('/') {
        let host_header = headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case("host"))
            .map(|(_, v)| v.as_str())?;
        let (host, port) = split_authority(host_header, 80);
        return Some((host, port, target.to_string()));
    } else {
        return None;
    };

    let (authority, path) = match rest.find('/') {
        Some(i) => (&rest[..i], &rest[i..]),
        None => (rest, "/"),
    };
    if authority.is_empty() {
        return None;
    }
    let (host, port) = split_authority(authority, default_port);
    Some((host, port, path.to_string()))
}

/// Split an authority (`host[:port]`, with optional IPv6 brackets) into
/// `(host, port)`, using `default_port` when absent.
fn split_authority(authority: &str, default_port: u16) -> (String, u16) {
    // Bare IPv6 without brackets (multiple colons) — don't attempt to split.
    let colons = authority.bytes().filter(|&b| b == b':').count();
    if colons > 1 && !authority.starts_with('[') {
        return (authority.to_string(), default_port);
    }
    if let Some((h, p)) = authority.rsplit_once(':') {
        if let Ok(port) = p.parse::<u16>() {
            return (h.to_string(), port);
        }
    }
    (authority.to_string(), default_port)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use tokio::io::{duplex, AsyncReadExt, AsyncWriteExt};
    use tokio::net::{TcpListener, TcpStream};

    fn headers(pairs: &[(&str, &str)]) -> Vec<(String, String)> {
        pairs
            .iter()
            .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
            .collect()
    }

    #[test]
    fn lan_access_ipv4_filters_non_lan_addresses() {
        assert!(lan_access_ipv4(std::net::Ipv4Addr::new(192, 168, 1, 23)));
        assert!(lan_access_ipv4(std::net::Ipv4Addr::new(169, 254, 10, 20)));
        assert!(!lan_access_ipv4(std::net::Ipv4Addr::LOCALHOST));
        assert!(!lan_access_ipv4(std::net::Ipv4Addr::UNSPECIFIED));
        assert!(!lan_access_ipv4(std::net::Ipv4Addr::new(8, 8, 8, 8)));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn read_body_decodes_chunked_request() {
        let (mut client, mut server) = duplex(1024);
        let writer = tokio::spawn(async move {
            client
                .write_all(b"llo\r\n6\r\n world\r\n0\r\nFoo: bar\r\n\r\n")
                .await
                .unwrap();
        });

        let body = read_body(
            &mut server,
            b"5\r\nhe",
            &headers(&[("Transfer-Encoding", "chunked")]),
        )
        .await
        .unwrap();

        writer.await.unwrap();
        assert_eq!(body, b"hello world");
    }

    #[tokio::test(flavor = "current_thread")]
    async fn read_body_sends_100_continue_before_waiting_for_body() {
        let (mut client, mut server) = duplex(1024);
        let client_task = tokio::spawn(async move {
            let mut got = Vec::new();
            let mut tmp = [0u8; 64];
            loop {
                let n = client.read(&mut tmp).await.unwrap();
                assert!(n > 0, "proxy closed before sending 100 Continue");
                got.extend_from_slice(&tmp[..n]);
                if got.windows(4).any(|w| w == b"\r\n\r\n") {
                    break;
                }
            }
            assert_eq!(got, b"HTTP/1.1 100 Continue\r\n\r\n");
            client.write_all(b"hello").await.unwrap();
        });

        let body = read_body(
            &mut server,
            &[],
            &headers(&[("Content-Length", "5"), ("Expect", "100-continue")]),
        )
        .await
        .unwrap();

        client_task.await.unwrap();
        assert_eq!(body, b"hello");
    }

    #[tokio::test(flavor = "current_thread")]
    async fn read_body_rejects_oversized_content_length() {
        let (_client, mut server) = duplex(64);
        let too_large = (MAX_REQUEST_BODY_BYTES + 1).to_string();
        let err = read_body(
            &mut server,
            &[],
            &headers(&[("Content-Length", too_large.as_str())]),
        )
        .await
        .unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
        assert!(err.to_string().contains("request body too large"));
    }

    #[test]
    fn sni_rewrite_is_only_for_port_443() {
        let mut hosts = std::collections::HashMap::new();
        hosts.insert("example.com".to_string(), "1.2.3.4".to_string());

        assert!(should_use_sni_rewrite(&hosts, "google.com", 443, false));
        assert!(!should_use_sni_rewrite(&hosts, "google.com", 80, false));
        assert!(should_use_sni_rewrite(
            &hosts,
            "www.example.com",
            443,
            false
        ));
        assert!(!should_use_sni_rewrite(
            &hosts,
            "www.example.com",
            80,
            false
        ));
    }

    #[test]
    fn youtube_via_relay_routes_only_youtube_html_api_through_relay_path() {
        // When youtube_via_relay=true, YouTube HTML/API suffixes must NOT
        // match the SNI-rewrite path, so they fall through to Apps Script
        // relay. Asset CDNs stay on SNI rewrite to avoid quota burn.
        let hosts = std::collections::HashMap::new();

        // Default behaviour: everything in the pool rewrites.
        assert!(should_use_sni_rewrite(
            &hosts,
            "www.youtube.com",
            443,
            false
        ));
        assert!(should_use_sni_rewrite(&hosts, "i.ytimg.com", 443, false));
        assert!(should_use_sni_rewrite(&hosts, "youtu.be", 443, false));
        assert!(should_use_sni_rewrite(
            &hosts,
            "youtubei.googleapis.com",
            443,
            false
        ));
        assert!(should_use_sni_rewrite(&hosts, "www.google.com", 443, false));

        // With the toggle on: YouTube HTML/API opts out; Google and asset CDNs stay.
        assert!(!should_use_sni_rewrite(
            &hosts,
            "www.youtube.com",
            443,
            true
        ));
        assert!(!should_use_sni_rewrite(&hosts, "youtu.be", 443, true));
        assert!(!should_use_sni_rewrite(
            &hosts,
            "youtubei.googleapis.com",
            443,
            true
        ));
        assert!(should_use_sni_rewrite(&hosts, "i.ytimg.com", 443, true));
        assert!(should_use_sni_rewrite(&hosts, "www.google.com", 443, true));
        assert!(should_use_sni_rewrite(
            &hosts,
            "fonts.gstatic.com",
            443,
            true
        ));
    }

    #[test]
    fn hosts_override_beats_youtube_via_relay() {
        // If the user added an explicit hosts override for a YouTube
        // subdomain, it should win — the override is a deliberate
        // user choice, the toggle is a default policy.
        let mut hosts = std::collections::HashMap::new();
        hosts.insert("rr4.googlevideo.com".to_string(), "1.2.3.4".to_string());

        assert!(should_use_sni_rewrite(
            &hosts,
            "rr4.googlevideo.com",
            443,
            true
        ));
    }

    #[test]
    fn passthrough_hosts_exact_match() {
        let list = vec!["example.com".to_string(), "banking.local".to_string()];
        assert!(matches_passthrough("example.com", &list));
        assert!(matches_passthrough("banking.local", &list));
        assert!(matches_passthrough("EXAMPLE.COM", &list)); // case-insensitive
        assert!(!matches_passthrough("notexample.com", &list));
        assert!(!matches_passthrough("sub.example.com", &list)); // exact only, not suffix
    }

    #[test]
    fn passthrough_hosts_dot_prefix_is_suffix_match() {
        let list = vec![".internal.example".to_string()];
        assert!(matches_passthrough("internal.example", &list)); // bare parent matches
        assert!(matches_passthrough("a.internal.example", &list));
        assert!(matches_passthrough("a.b.c.internal.example", &list));
        assert!(!matches_passthrough("internal.exampleX", &list));
        assert!(!matches_passthrough("fakeinternal.example", &list));
    }

    #[test]
    fn passthrough_hosts_wildcard_prefix_is_suffix_match() {
        let list = vec!["*.metatrader.com".to_string()];
        assert!(matches_passthrough("metatrader.com", &list));
        assert!(matches_passthrough("trade.metatrader.com", &list));
        assert!(matches_passthrough("a.b.metatrader.com", &list));
        assert!(!matches_passthrough("notmetatrader.com", &list));
    }

    #[test]
    fn passthrough_hosts_empty_list_never_matches() {
        let list: Vec<String> = vec![];
        assert!(!matches_passthrough("anything.com", &list));
        assert!(!matches_passthrough("", &list));
    }

    #[test]
    fn passthrough_hosts_ignores_empty_and_whitespace_entries() {
        let list = vec!["".to_string(), "   ".to_string(), "real.com".to_string()];
        assert!(!matches_passthrough("", &list));
        assert!(matches_passthrough("real.com", &list));
    }

    #[test]
    fn resolve_plain_http_target_parses_absolute_form() {
        let h = headers(&[]);
        let (host, port, path) = resolve_plain_http_target("http://example.com/", &h).unwrap();
        assert_eq!(host, "example.com");
        assert_eq!(port, 80);
        assert_eq!(path, "/");

        let (host, port, path) =
            resolve_plain_http_target("http://example.com:8080/foo?x=1", &h).unwrap();
        assert_eq!(host, "example.com");
        assert_eq!(port, 8080);
        assert_eq!(path, "/foo?x=1");

        let (host, port, path) = resolve_plain_http_target("http://example.com", &h).unwrap();
        assert_eq!(host, "example.com");
        assert_eq!(port, 80);
        assert_eq!(path, "/");
    }

    #[test]
    fn resolve_plain_http_target_falls_back_to_host_header() {
        let h = headers(&[("Host", "example.com:8080")]);
        let (host, port, path) = resolve_plain_http_target("/foo", &h).unwrap();
        assert_eq!(host, "example.com");
        assert_eq!(port, 8080);
        assert_eq!(path, "/foo");
    }

    #[test]
    fn resolve_plain_http_target_rejects_bare_authority() {
        assert!(resolve_plain_http_target("example.com", &headers(&[])).is_none());
        assert!(resolve_plain_http_target("http://", &headers(&[])).is_none());
    }

    #[test]
    fn split_authority_handles_ports_and_ipv6() {
        assert_eq!(
            split_authority("example.com", 80),
            ("example.com".to_string(), 80)
        );
        assert_eq!(
            split_authority("example.com:8080", 80),
            ("example.com".to_string(), 8080)
        );
        assert_eq!(
            split_authority("[::1]:8080", 80),
            ("[::1]".to_string(), 8080)
        );
        assert_eq!(split_authority("::1", 80), ("::1".to_string(), 80));
    }

    #[test]
    fn socks5_udp_domain_packet_round_trips() {
        let raw = [
            0, 0, 0, 0x03, 11, b'e', b'x', b'a', b'm', b'p', b'l', b'e', b'.', b'c', b'o', b'm',
            0x12, 0x34, b'h', b'i',
        ];
        let (target, payload) = parse_socks5_udp_packet(&raw).unwrap();
        assert_eq!(target.host, "example.com");
        assert_eq!(target.port, 0x1234);
        assert_eq!(payload, b"hi");
        assert_eq!(build_socks5_udp_packet(&target, payload), raw);
    }

    #[test]
    fn socks5_udp_rejects_fragmented_packets() {
        let raw = [0, 0, 1, 0x01, 127, 0, 0, 1, 0, 53, b'x'];
        assert!(parse_socks5_udp_packet(&raw).is_none());
    }

    #[test]
    fn socks5_udp_rejects_non_utf8_domain() {
        let raw = [0, 0, 0, 0x03, 1, 0xff, 0, 53, b'x'];
        assert!(parse_socks5_udp_packet(&raw).is_none());
    }

    #[test]
    fn socks5_udp_rejects_truncated_inputs() {
        assert!(parse_socks5_udp_packet(&[0, 0, 0, 0x01]).is_none());
        assert!(parse_socks5_udp_packet(&[0, 0, 0, 0x01, 127, 0, 0]).is_none());
        assert!(parse_socks5_udp_packet(&[0, 0, 0, 0x01, 127, 0, 0, 1]).is_none());
        assert!(parse_socks5_udp_packet(&[0, 0, 0, 0x03, 0, 0, 80]).is_none());
        assert!(parse_socks5_udp_packet(&[0, 0, 0, 0x03, 5, b'a', b'b']).is_none());
        assert!(parse_socks5_udp_packet(&[0, 0, 0, 0x09, 1, 2, 3, 4]).is_none());
        let raw = [0, 0, 0, 0x04, 1, 2, 3, 4, 0, 53, b'x'];
        assert!(parse_socks5_udp_packet(&raw).is_none());
    }

    #[test]
    fn socks5_udp_ipv4_round_trips() {
        let raw = [0, 0, 0, 0x01, 127, 0, 0, 1, 0, 53, b'x'];
        let (target, payload) = parse_socks5_udp_packet(&raw).unwrap();
        assert_eq!(target.host, "127.0.0.1");
        assert_eq!(target.port, 53);
        assert_eq!(payload, b"x");
        assert_eq!(build_socks5_udp_packet(&target, payload), raw);
    }

    #[test]
    fn socks5_udp_ipv6_round_trips() {
        let mut raw = vec![0, 0, 0, 0x04];
        raw.extend_from_slice(&std::net::Ipv6Addr::LOCALHOST.octets());
        raw.extend_from_slice(&53u16.to_be_bytes());
        raw.extend_from_slice(b"dns");
        let (target, payload) = parse_socks5_udp_packet(&raw).unwrap();
        assert_eq!(target.host, "::1");
        assert_eq!(target.port, 53);
        assert_eq!(payload, b"dns");
        assert_eq!(build_socks5_udp_packet(&target, payload), raw);
    }

    #[test]
    fn passthrough_hosts_trailing_dot_normalized() {
        // FQDNs sometimes have a trailing dot; both entry-side and host-side
        // trailing dots should be treated as equivalent to the un-dotted form.
        let list = vec!["example.com.".to_string()];
        assert!(matches_passthrough("example.com", &list));
        assert!(matches_passthrough("example.com.", &list));
    }

    #[test]
    fn doh_default_list_exact_matches() {
        let extra: Vec<String> = vec![];
        assert!(matches_doh_host("chrome.cloudflare-dns.com", &extra));
        assert!(matches_doh_host("dns.google", &extra));
        assert!(matches_doh_host("dns.quad9.net", &extra));
        assert!(matches_doh_host("doh.opendns.com", &extra));
    }

    #[test]
    fn doh_default_list_case_insensitive_and_trailing_dot() {
        let extra: Vec<String> = vec![];
        assert!(matches_doh_host("DNS.GOOGLE", &extra));
        assert!(matches_doh_host("dns.google.", &extra));
    }

    #[test]
    fn doh_default_list_suffix_match_for_tenant_subdomains() {
        let extra: Vec<String> = vec![];
        assert!(matches_doh_host("tenant.cloudflare-dns.com", &extra));
        assert!(!matches_doh_host("xcloudflare-dns.com", &extra));
    }

    #[test]
    fn doh_extra_list_extends_default() {
        let extra = vec![
            ".internal-doh.example".to_string(),
            "doh.acme.test".to_string(),
        ];
        assert!(matches_doh_host("dns.google", &extra));
        assert!(matches_doh_host("doh.acme.test", &extra));
        assert!(matches_doh_host("a.b.internal-doh.example", &extra));
        assert!(!matches_doh_host("example.com", &extra));
    }

    #[test]
    fn doh_extra_entries_match_subdomains_without_leading_dot() {
        let extra = vec!["doh.acme.test".to_string()];
        assert!(matches_doh_host("doh.acme.test", &extra));
        assert!(matches_doh_host("tenant.doh.acme.test", &extra));
        assert!(!matches_doh_host("xdoh.acme.test", &extra));
    }

    fn tls_connector_test_only() -> TlsConnector {
        let tls_config = ClientConfig::builder()
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(NoVerify))
            .with_no_client_auth();
        TlsConnector::from(Arc::new(tls_config))
    }

    fn rewrite_ctx_lan(lan_token: Option<&str>, lan_allowlist: Option<Vec<&str>>) -> RewriteCtx {
        RewriteCtx {
            google_ip: "127.0.0.1".into(),
            front_domain: "example.com".into(),
            hosts: Default::default(),
            tls_connector: tls_connector_test_only(),
            upstream_socks5: None,
            mode: Mode::Direct,
            youtube_via_relay: false,
            passthrough_hosts: vec![],
            block_quic: false,
            bypass_doh: true,
            bypass_doh_hosts: vec![],
            fronting_groups: vec![],
            domain_overrides: vec![],
            lan_token: lan_token.map(str::to_string),
            lan_allowlist: lan_allowlist.map(|v| v.into_iter().map(|s| s.to_string()).collect()),
            listen_host: "0.0.0.0".into(),
        }
    }

    async fn tcp_connected_pair() -> (TcpStream, TcpStream) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let server = tokio::spawn(async move { listener.accept().await.unwrap().0 });
        let client = TcpStream::connect(addr).await.unwrap();
        let inbound = server.await.unwrap();
        (inbound, client)
    }

    #[tokio::test(flavor = "current_thread")]
    async fn lan_socks_allowed_no_lan_auth() {
        let (inbound, _keepalive) = tcp_connected_pair().await;
        let ctx = rewrite_ctx_lan(None, None);
        assert!(lan_socks_allowed(&inbound, &ctx));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn lan_socks_rejects_token_without_allowlist() {
        let (inbound, _keep) = tcp_connected_pair().await;
        let ctx = rewrite_ctx_lan(Some("secret"), None);
        assert!(!lan_socks_allowed(&inbound, &ctx));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn lan_socks_rejects_token_with_empty_allowlist_vec() {
        let (inbound, _keep) = tcp_connected_pair().await;
        let ctx = rewrite_ctx_lan(Some("secret"), Some(vec![]));
        assert!(!lan_socks_allowed(&inbound, &ctx));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn lan_socks_allowlist_only_enforces_peer_ip() {
        let (inbound, _keep) = tcp_connected_pair().await;
        let ctx = rewrite_ctx_lan(None, Some(vec!["127.0.0.1"]));
        assert!(lan_socks_allowed(&inbound, &ctx));

        let (inbound2, _keep2) = tcp_connected_pair().await;
        let ctx2 = rewrite_ctx_lan(None, Some(vec!["10.0.0.1"]));
        assert!(!lan_socks_allowed(&inbound2, &ctx2));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn lan_socks_token_plus_allowlist_allows_listed_peer() {
        let (inbound, _keep) = tcp_connected_pair().await;
        let ctx = rewrite_ctx_lan(Some("secret"), Some(vec!["127.0.0.1"]));
        assert!(lan_socks_allowed(&inbound, &ctx));
    }

    #[test]
    fn lan_allowlist_accepts_ipv4_and_ipv6_cidr() {
        assert!(lan_allowlist_entry_matches(
            "127.0.0.0/8",
            "127.0.0.1".parse().unwrap()
        ));
        assert!(!lan_allowlist_entry_matches(
            "10.0.0.0/8",
            "127.0.0.1".parse().unwrap()
        ));
        assert!(lan_allowlist_entry_matches(
            "2001:db8::/32",
            "2001:db8::42".parse().unwrap()
        ));
        assert!(!lan_allowlist_entry_matches(
            "2001:db8::/32",
            "2001:db9::42".parse().unwrap()
        ));
    }
}
