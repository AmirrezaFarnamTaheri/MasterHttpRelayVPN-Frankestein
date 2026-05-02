//! MasterHttpRelayVPN-Frankestein — `mhrv-f doctor` (first-run diagnostics).
//!
//! Goal: maximize first-run success by detecting the common failure modes and
//! printing actionable next steps.

use crate::cert_installer::{install_ca, is_ca_trusted};
use crate::config::{Config, Mode};
use crate::mitm::{MitmCertManager, CA_CERT_FILE};
use crate::test_cmd;
use rustls::pki_types::ServerName;
use serde::Deserialize;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;
use url::Url;

#[derive(Clone, Debug)]
pub enum DoctorLevel {
    Ok,
    Warn,
    Fail,
}

#[derive(Clone, Debug)]
pub struct DoctorItem {
    pub id: &'static str,
    pub level: DoctorLevel,
    pub title: String,
    pub detail: String,
    pub fix: Option<String>,
}

#[derive(Clone, Debug)]
pub struct DoctorReport {
    pub items: Vec<DoctorItem>,
}

#[derive(Clone, Debug, Default)]
pub struct DoctorOptions {
    /// Optional tunnel-node origin or URL to probe in full mode. When set,
    /// Doctor normalizes it to `/health/details` and performs a live capability
    /// check instead of only printing the manual smoke-test checklist.
    pub tunnel_node_url: Option<String>,
}

impl DoctorReport {
    pub fn ok(&self) -> bool {
        !self
            .items
            .iter()
            .any(|i| matches!(i.level, DoctorLevel::Fail))
    }
}

pub async fn run(config: &Config) -> DoctorReport {
    run_with_options(config, &DoctorOptions::default()).await
}

pub async fn run_with_options(config: &Config, options: &DoctorOptions) -> DoctorReport {
    let mut items: Vec<DoctorItem> = Vec::new();

    // 1) Config safety warnings (non-fatal).
    let warns = config.unsafe_warnings();
    if warns.is_empty() {
        items.push(DoctorItem {
            id: "config_warnings",
            level: DoctorLevel::Ok,
            title: "Config warnings".into(),
            detail: "No unsafe-setting warnings detected.".into(),
            fix: None,
        });
    } else {
        items.push(DoctorItem {
            id: "config_warnings",
            level: DoctorLevel::Warn,
            title: "Config warnings".into(),
            detail: warns.join("\n- "),
            fix: Some("Review the warnings above and adjust config.json if needed.".into()),
        });
    }

    // 2) Mode sanity.
    let mode = match config.mode_kind() {
        Ok(m) => m,
        Err(e) => {
            items.push(DoctorItem {
                id: "mode",
                level: DoctorLevel::Fail,
                title: "Mode".into(),
                detail: format!("Invalid mode: {e}"),
                fix: Some("Set `mode` to one of: apps_script, vercel_edge, direct, full.".into()),
            });
            return DoctorReport { items };
        }
    };
    items.push(DoctorItem {
        id: "mode",
        level: DoctorLevel::Ok,
        title: "Mode".into(),
        detail: format!("mode = {}", mode.as_str()),
        fix: None,
    });

    // 3) Relay config presence.
    if matches!(mode, Mode::AppsScript | Mode::Full) {
        let groups = config.account_groups_resolved();
        if groups.is_empty() {
            items.push(DoctorItem {
                id: "account_groups",
                level: DoctorLevel::Fail,
                title: "Apps Script accounts".into(),
                detail: "No enabled account_groups found.".into(),
                fix: Some(
                    "Add at least one enabled account group with auth_key + script_ids (deployment IDs).".into(),
                ),
            });
        } else {
            let total_ids: usize = groups
                .iter()
                .map(|g| g.script_ids.clone().into_vec().len())
                .sum();
            items.push(DoctorItem {
                id: "account_groups",
                level: DoctorLevel::Ok,
                title: "Apps Script accounts".into(),
                detail: format!(
                    "Enabled accounts: {}  Deployments: {}",
                    groups.len(),
                    total_ids
                ),
                fix: None,
            });
        }
    } else if mode == Mode::VercelEdge {
        if config.vercel.base_url.trim().is_empty() || config.vercel.auth_key.trim().is_empty() {
            items.push(DoctorItem {
                id: "vercel_edge",
                level: DoctorLevel::Fail,
                title: "Serverless JSON relay".into(),
                detail: "vercel.base_url and vercel.auth_key are required.".into(),
                fix: Some(
                    "Deploy tools/vercel-json-relay or tools/netlify-json-relay, set AUTH_KEY, then copy the deployment URL and key into config.json."
                        .into(),
                ),
            });
        } else {
            items.push(DoctorItem {
                id: "vercel_edge",
                level: DoctorLevel::Ok,
                title: "Serverless JSON relay".into(),
                detail: format!(
                    "Endpoint: {}{}",
                    config.vercel.base_url.trim_end_matches('/'),
                    config.vercel.relay_path
                ),
                fix: None,
            });
        }
    }

    if mode == Mode::Full {
        add_full_mode_external_checks(config, options, &mut items).await;
    }

    // 4) MITM CA readiness / trust. Not needed for full mode.
    if mode == Mode::Full {
        items.push(DoctorItem {
            id: "mitm_ca",
            level: DoctorLevel::Ok,
            title: "MITM certificate".into(),
            detail: "Full mode: MITM CA not required.".into(),
            fix: None,
        });
    } else {
        let base = crate::data_dir::data_dir();
        match MitmCertManager::new_in(&base) {
            Ok(_) => {
                let ca_path = base.join(CA_CERT_FILE);
                let trusted = is_ca_trusted(&ca_path);
                if trusted {
                    items.push(DoctorItem {
                        id: "mitm_ca",
                        level: DoctorLevel::Ok,
                        title: "MITM certificate".into(),
                        detail: format!(
                            "CA file exists and appears trusted: {}",
                            ca_path.display()
                        ),
                        fix: None,
                    });
                } else {
                    items.push(DoctorItem {
                        id: "mitm_ca",
                        level: DoctorLevel::Warn,
                        title: "MITM certificate".into(),
                        detail: format!("CA file exists but does not appear trusted: {}", ca_path.display()),
                        fix: Some("Run `mhrv-f --install-cert` (may require admin) or import ca/ca.crt into your OS trust store.".into()),
                    });
                }
            }
            Err(e) => items.push(DoctorItem {
                id: "mitm_ca",
                level: DoctorLevel::Fail,
                title: "MITM certificate".into(),
                detail: format!("Failed to initialize local CA files: {e}"),
                fix: Some("Ensure the user-data directory is writable and try again.".into()),
            }),
        }
    }

    // 5) End-to-end relay probe (same as `test`), only when relay exists.
    if matches!(mode, Mode::AppsScript | Mode::VercelEdge) {
        let ok = test_cmd::run(config).await;
        items.push(DoctorItem {
            id: "relay_probe",
            level: if ok { DoctorLevel::Ok } else { DoctorLevel::Fail },
            title: "Relay probe".into(),
            detail: if ok {
                "PASS: end-to-end relay verified.".into()
            } else {
                "FAIL: relay probe failed. See logs above for the detailed reason.".into()
            },
            fix: if ok {
                None
            } else {
                Some("Common fixes: verify AUTH_KEY matches; for Apps Script, replace dead deployment IDs/re-deploy as 'Anyone'/scan a different google_ip; for serverless JSON, remove protection/routing pages and redeploy after env var changes.".into())
            },
        });
    } else if mode == Mode::Full {
        items.push(DoctorItem {
            id: "relay_probe",
            level: DoctorLevel::Warn,
            title: "Relay probe".into(),
            detail: "`mhrv-f test` is intentionally skipped in full mode; verify full mode by browsing through the tunnel and checking the tunnel-node public IP.".into(),
            fix: None,
        });
    }

    DoctorReport { items }
}

async fn add_full_mode_external_checks(
    config: &Config,
    options: &DoctorOptions,
    items: &mut Vec<DoctorItem>,
) {
    items.push(DoctorItem {
        id: crate::readiness::FULL_CODEFULL_DEPLOYMENT,
        level: DoctorLevel::Warn,
        title: "Full tunnel: CodeFull deployment".into(),
        detail: "Doctor cannot inspect deployed Apps Script source. Confirm every configured deployment ID points to assets/apps_script/CodeFull.gs, not classic Code.gs.".into(),
        fix: Some("Open each Apps Script deployment, paste CodeFull.gs, redeploy as Web app, then update the deployment ID if Apps Script issued a new one.".into()),
    });
    items.push(DoctorItem {
        id: crate::readiness::FULL_TUNNEL_NODE_URL,
        level: DoctorLevel::Warn,
        title: "Full tunnel: tunnel-node URL".into(),
        detail: "Doctor cannot read TUNNEL_SERVER_URL from deployed CodeFull.gs. Confirm it points to the public tunnel-node origin, without /tunnel appended.".into(),
        fix: Some("Check CodeFull.gs TUNNEL_SERVER_URL and verify the node responds at /healthz and /health/details.".into()),
    });
    items.push(DoctorItem {
        id: crate::readiness::FULL_TUNNEL_AUTH,
        level: DoctorLevel::Warn,
        title: "Full tunnel: tunnel auth".into(),
        detail: "Doctor cannot read the tunnel-node environment. Confirm CodeFull.gs TUNNEL_AUTH_KEY exactly matches the tunnel-node TUNNEL_AUTH_KEY.".into(),
        fix: Some("Regenerate one long secret and set the same value in CodeFull.gs and the tunnel-node environment, then restart/redeploy both sides.".into()),
    });

    let socks_ready = config.socks5_port.is_some();
    items.push(DoctorItem {
        id: crate::readiness::FULL_UDP_SUPPORT,
        level: if socks_ready {
            DoctorLevel::Ok
        } else {
            DoctorLevel::Warn
        },
        title: "Full tunnel: UDP/SOCKS5 path".into(),
        detail: if let Some(port) = config.socks5_port {
            format!("SOCKS5 listener configured on port {port}; UDP-capable clients can use SOCKS5 UDP ASSOCIATE.")
        } else {
            "No socks5_port configured. TCP browsing can still work, but app-level UDP through full mode needs SOCKS5 UDP ASSOCIATE.".into()
        },
        fix: if socks_ready {
            None
        } else {
            Some("Set socks5_port to a local port such as 8086 when clients need UDP through full mode.".into())
        },
    });

    items.push(full_tunnel_health_item(options).await);
}

async fn full_tunnel_health_item(options: &DoctorOptions) -> DoctorItem {
    let Some(raw_url) = options.tunnel_node_url.as_deref().map(str::trim) else {
        return manual_full_tunnel_health_item();
    };
    if raw_url.is_empty() {
        return manual_full_tunnel_health_item();
    }

    match normalize_tunnel_node_health_url(raw_url) {
        Ok(url) => match probe_tunnel_node_health(&url).await {
            Ok(health) => health.to_doctor_item(&url),
            Err(e) => DoctorItem {
                id: crate::readiness::FULL_TUNNEL_HEALTH,
                level: DoctorLevel::Fail,
                title: "Full tunnel: tunnel-node health".into(),
                detail: format!(
                    "Could not verify {}: {e}",
                    redact_url_for_display(&url)
                ),
                fix: Some(
                    "Confirm the tunnel-node is reachable from this machine, TLS is valid, /health/details is served, and reverse-proxy routes do not require login."
                        .into(),
                ),
            },
        },
        Err(e) => DoctorItem {
            id: crate::readiness::FULL_TUNNEL_HEALTH,
            level: DoctorLevel::Fail,
            title: "Full tunnel: tunnel-node health".into(),
            detail: format!("Invalid --tunnel-node-url value: {e}"),
            fix: Some(
                "Pass the public tunnel-node origin, for example `--tunnel-node-url https://tunnel.example.com`."
                    .into(),
            ),
        },
    }
}

fn manual_full_tunnel_health_item() -> DoctorItem {
    DoctorItem {
        id: crate::readiness::FULL_TUNNEL_HEALTH,
        level: DoctorLevel::Warn,
        title: "Full tunnel: health smoke test".into(),
        detail: "Start full mode, open a public IP-check page, and compare the result with tunnel-node logs. The normal relay probe is not sufficient for full mode.".into(),
        fix: Some("Run `mhrv-f doctor --tunnel-node-url https://<tunnel-node>` to probe /health/details before starting; after starting, confirm the browser egress IP matches the VPS.".into()),
    }
}

fn normalize_tunnel_node_health_url(raw: &str) -> Result<Url, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err("URL is empty".into());
    }
    let candidate = if trimmed.contains("://") {
        trimmed.to_string()
    } else {
        format!("https://{trimmed}")
    };
    let mut url = Url::parse(&candidate).map_err(|e| e.to_string())?;
    match url.scheme() {
        "http" | "https" => {}
        other => return Err(format!("unsupported scheme `{other}`; use http or https")),
    }
    if url.host_str().is_none() {
        return Err("missing host".into());
    }
    url.set_path("/health/details");
    url.set_query(None);
    url.set_fragment(None);
    Ok(url)
}

fn redact_url_for_display(url: &Url) -> String {
    let mut display = url.clone();
    let _ = display.set_username("");
    let _ = display.set_password(None);
    display.to_string()
}

#[derive(Debug, Deserialize)]
struct TunnelNodeHealth {
    status: Option<String>,
    service: Option<String>,
    version: Option<String>,
    protocol: Option<String>,
    supports_batch: Option<bool>,
    supports_udp: Option<bool>,
    supports_udpgw: Option<bool>,
    auth: Option<String>,
}

impl TunnelNodeHealth {
    fn to_doctor_item(&self, url: &Url) -> DoctorItem {
        let mut missing = Vec::new();
        if self.status.as_deref() != Some("ok") {
            missing.push("status=ok");
        }
        if self.protocol.as_deref() != Some("mhrv-full-tunnel") {
            missing.push("protocol=mhrv-full-tunnel");
        }
        if self.supports_batch != Some(true) {
            missing.push("supports_batch=true");
        }
        if self.supports_udp != Some(true) {
            missing.push("supports_udp=true");
        }
        if self.supports_udpgw != Some(true) {
            missing.push("supports_udpgw=true");
        }

        if missing.is_empty() {
            let service = self.service.as_deref().unwrap_or("tunnel-node");
            let version = self.version.as_deref().unwrap_or("unknown");
            let auth_name = self.auth.as_deref().unwrap_or("unknown");
            DoctorItem {
                id: crate::readiness::FULL_TUNNEL_HEALTH,
                level: DoctorLevel::Ok,
                title: "Full tunnel: tunnel-node health".into(),
                detail: format!(
                    "{} is reachable and advertises full-tunnel capabilities. service={service}; version={version}; auth={auth_name}",
                    redact_url_for_display(url)
                ),
                fix: None,
            }
        } else {
            DoctorItem {
                id: crate::readiness::FULL_TUNNEL_HEALTH,
                level: DoctorLevel::Warn,
                title: "Full tunnel: tunnel-node health".into(),
                detail: format!(
                    "{} responded, but the capability document is incomplete or version-skewed: missing {}.",
                    redact_url_for_display(url),
                    missing.join(", ")
                ),
                fix: Some(
                    "Upgrade/redeploy tunnel-node and confirm /health/details matches the documented full-tunnel capability document."
                        .into(),
                ),
            }
        }
    }
}

async fn probe_tunnel_node_health(url: &Url) -> Result<TunnelNodeHealth, String> {
    let host = url
        .host_str()
        .ok_or_else(|| "health URL is missing host".to_string())?;
    let port = url
        .port_or_known_default()
        .ok_or_else(|| "health URL is missing port".to_string())?;
    let path = match url.query() {
        Some(query) => format!("{}?{query}", url.path()),
        None => url.path().to_string(),
    };

    let response = match url.scheme() {
        "http" => {
            let mut stream =
                tokio::time::timeout(Duration::from_secs(6), TcpStream::connect((host, port)))
                    .await
                    .map_err(|_| "tcp connect timeout".to_string())?
                    .map_err(|e| format!("tcp connect: {e}"))?;
            let _ = stream.set_nodelay(true);
            write_get_and_read(&mut stream, host, port, &path).await?
        }
        "https" => {
            let mut roots = rustls::RootCertStore::empty();
            roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
            let tls_cfg = rustls::ClientConfig::builder()
                .with_root_certificates(roots)
                .with_no_client_auth();
            let connector = TlsConnector::from(Arc::new(tls_cfg));
            let stream =
                tokio::time::timeout(Duration::from_secs(6), TcpStream::connect((host, port)))
                    .await
                    .map_err(|_| "tcp connect timeout".to_string())?
                    .map_err(|e| format!("tcp connect: {e}"))?;
            let _ = stream.set_nodelay(true);
            let server_name =
                ServerName::try_from(host.to_string()).map_err(|e| format!("bad host: {e}"))?;
            let mut tls = tokio::time::timeout(
                Duration::from_secs(8),
                connector.connect(server_name, stream),
            )
            .await
            .map_err(|_| "tls handshake timeout".to_string())?
            .map_err(|e| format!("tls handshake: {e}"))?;
            write_get_and_read(&mut tls, host, port, &path).await?
        }
        other => return Err(format!("unsupported scheme `{other}`")),
    };

    let (status, body) = split_http_response(&response)?;
    if !(200..300).contains(&status) {
        let preview = String::from_utf8_lossy(&body)
            .chars()
            .take(220)
            .collect::<String>();
        return Err(format!("HTTP {status}: {preview}"));
    }
    serde_json::from_slice::<TunnelNodeHealth>(&body).map_err(|e| format!("invalid JSON: {e}"))
}

async fn write_get_and_read<S>(
    stream: &mut S,
    host: &str,
    port: u16,
    path: &str,
) -> Result<Vec<u8>, String>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    let request = format!(
        "GET {path} HTTP/1.1\r\n\
         Host: {host_header}\r\n\
         User-Agent: mhrv-f/{version} (doctor)\r\n\
         Accept: application/json\r\n\
         Connection: close\r\n\
         \r\n",
        host_header = host_header(host, port),
        version = env!("CARGO_PKG_VERSION"),
    );
    stream
        .write_all(request.as_bytes())
        .await
        .map_err(|e| format!("write request: {e}"))?;
    stream
        .flush()
        .await
        .map_err(|e| format!("flush request: {e}"))?;

    let mut response = Vec::with_capacity(16 * 1024);
    let read_fut = async {
        let mut chunk = [0u8; 4096];
        loop {
            let n = stream
                .read(&mut chunk)
                .await
                .map_err(|e| format!("read response: {e}"))?;
            if n == 0 {
                break;
            }
            response.extend_from_slice(&chunk[..n]);
            if response.len() > 64 * 1024 {
                return Err("response too large".to_string());
            }
        }
        Ok::<(), String>(())
    };
    tokio::time::timeout(Duration::from_secs(10), read_fut)
        .await
        .map_err(|_| "read response timeout".to_string())??;
    Ok(response)
}

fn host_header(host: &str, port: u16) -> String {
    let host_part = if host.contains(':') && !host.starts_with('[') {
        format!("[{host}]")
    } else {
        host.to_string()
    };
    match port {
        80 | 443 => host_part,
        _ => format!("{host_part}:{port}"),
    }
}

fn split_http_response(response: &[u8]) -> Result<(u16, Vec<u8>), String> {
    let header_end = response
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .ok_or_else(|| "response missing HTTP header terminator".to_string())?;
    let header = &response[..header_end];
    let body = &response[header_end + 4..];
    let header_text = String::from_utf8_lossy(header);
    let mut lines = header_text.lines();
    let status_line = lines
        .next()
        .ok_or_else(|| "empty HTTP response".to_string())?;
    let status = status_line
        .split_whitespace()
        .nth(1)
        .ok_or_else(|| format!("bad HTTP status line: {status_line}"))?
        .parse::<u16>()
        .map_err(|_| format!("bad HTTP status line: {status_line}"))?;

    let chunked = lines.clone().any(|line| {
        let lower = line.to_ascii_lowercase();
        lower.starts_with("transfer-encoding:") && lower.contains("chunked")
    });
    if chunked {
        Ok((status, decode_chunked_body(body)?))
    } else {
        Ok((status, body.to_vec()))
    }
}

fn decode_chunked_body(mut body: &[u8]) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    loop {
        let line_end = body
            .windows(2)
            .position(|w| w == b"\r\n")
            .ok_or_else(|| "malformed chunked body".to_string())?;
        let size_line = std::str::from_utf8(&body[..line_end])
            .map_err(|_| "non-utf8 chunk size".to_string())?;
        let size_hex = size_line.split(';').next().unwrap_or("").trim();
        let size = usize::from_str_radix(size_hex, 16)
            .map_err(|_| format!("invalid chunk size `{size_hex}`"))?;
        body = &body[line_end + 2..];
        if size == 0 {
            return Ok(out);
        }
        if body.len() < size + 2 || &body[size..size + 2] != b"\r\n" {
            return Err("chunk body ended early".into());
        }
        out.extend_from_slice(&body[..size]);
        body = &body[size + 2..];
        if out.len() > 64 * 1024 {
            return Err("decoded response too large".into());
        }
    }
}

#[derive(Clone, Debug)]
pub struct FixOutcome {
    pub id: &'static str,
    pub ok: bool,
    pub detail: String,
}

/// Best-effort one-click fixes. Only performs actions that are:
/// - local-only
/// - safe to attempt automatically
/// - reversible by the user
///
/// This does NOT mutate config.json (except indirectly if OS CA installers do so).
pub fn apply_one_click_fixes(config: &Config) -> Vec<FixOutcome> {
    let mut out = Vec::new();
    let mode = config.mode_kind().ok();

    // Fix 1: ensure CA files exist + attempt to install into OS trust store
    // when needed (apps_script / direct; full mode doesn't use MITM).
    if mode != Some(Mode::Full) {
        let base = crate::data_dir::data_dir();
        match MitmCertManager::new_in(&base) {
            Ok(_) => {
                let ca_path = base.join(CA_CERT_FILE);
                if is_ca_trusted(&ca_path) {
                    out.push(FixOutcome {
                        id: "fix_install_ca",
                        ok: true,
                        detail: "CA already appears trusted; nothing to do.".into(),
                    });
                } else {
                    match install_ca(&ca_path) {
                        Ok(()) => out.push(FixOutcome {
                            id: "fix_install_ca",
                            ok: true,
                            detail: format!(
                                "Installed CA into OS trust store: {}",
                                ca_path.display()
                            ),
                        }),
                        Err(e) => out.push(FixOutcome {
                            id: "fix_install_ca",
                            ok: false,
                            detail: format!(
                                "Failed to install CA automatically (may require admin): {e}"
                            ),
                        }),
                    }
                }
            }
            Err(e) => out.push(FixOutcome {
                id: "fix_init_ca",
                ok: false,
                detail: format!("Failed to initialize CA files: {e}"),
            }),
        }
    }

    out
}

/// Run doctor, apply one-click fixes, then run doctor again.
pub async fn run_with_fixes(config: &Config) -> (DoctorReport, Vec<FixOutcome>, DoctorReport) {
    run_with_fixes_and_options(config, &DoctorOptions::default()).await
}

/// Run doctor with options, apply one-click fixes, then run doctor again.
pub async fn run_with_fixes_and_options(
    config: &Config,
    options: &DoctorOptions,
) -> (DoctorReport, Vec<FixOutcome>, DoctorReport) {
    let before = run_with_options(config, options).await;
    let fixes = apply_one_click_fixes(config);
    let after = run_with_options(config, options).await;
    (before, fixes, after)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    fn full_config(socks5_port: Option<u16>) -> Config {
        let socks = socks5_port
            .map(|port| format!(r#","socks5_port": {port}"#))
            .unwrap_or_default();
        Config::from_json_str(&format!(
            r#"{{
                "mode": "full",
                "account_groups": [{{
                    "auth_key": "test-auth-key-please-change-32chars",
                    "script_ids": ["AKfycb_full"],
                    "enabled": true
                }}]
                {socks}
            }}"#
        ))
        .expect("full config should load")
    }

    #[tokio::test]
    async fn doctor_reports_structured_full_mode_external_checks() {
        let report = run(&full_config(Some(8086))).await;
        let ids: Vec<_> = report.items.iter().map(|item| item.id).collect();

        for id in [
            crate::readiness::FULL_CODEFULL_DEPLOYMENT,
            crate::readiness::FULL_TUNNEL_NODE_URL,
            crate::readiness::FULL_TUNNEL_AUTH,
            crate::readiness::FULL_UDP_SUPPORT,
            crate::readiness::FULL_TUNNEL_HEALTH,
        ] {
            assert!(ids.contains(&id), "missing doctor item {id}");
        }
        assert!(report.items.iter().any(|item| {
            item.id == crate::readiness::FULL_UDP_SUPPORT && matches!(item.level, DoctorLevel::Ok)
        }));
    }

    #[tokio::test]
    async fn doctor_warns_when_full_mode_udp_has_no_socks5_listener() {
        let report = run(&full_config(None)).await;
        assert!(report.items.iter().any(|item| {
            item.id == crate::readiness::FULL_UDP_SUPPORT
                && matches!(item.level, DoctorLevel::Warn)
                && item.detail.contains("No socks5_port")
        }));
    }

    #[test]
    fn tunnel_node_health_url_normalization_targets_details_endpoint() {
        let url = normalize_tunnel_node_health_url("tunnel.example.com/base?old=1")
            .expect("bare host should default to https");
        assert_eq!(url.as_str(), "https://tunnel.example.com/health/details");

        let url = normalize_tunnel_node_health_url("http://127.0.0.1:8080/healthz")
            .expect("http URL should be supported");
        assert_eq!(url.as_str(), "http://127.0.0.1:8080/health/details");
    }

    #[tokio::test]
    async fn doctor_can_probe_tunnel_node_health_details() {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind test listener");
        let addr = listener.local_addr().expect("listener address");
        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.expect("accept health probe");
            let mut request = [0u8; 1024];
            let n = stream.read(&mut request).await.expect("read request");
            let text = String::from_utf8_lossy(&request[..n]);
            assert!(text.starts_with("GET /health/details HTTP/1.1"));
            let body = br#"{"status":"ok","service":"mhrv-f tunnel-node","version":"0.1.0","protocol":"mhrv-full-tunnel","supports_batch":true,"supports_udp":true,"supports_udpgw":true,"auth":"TUNNEL_AUTH_KEY"}"#;
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            );
            stream
                .write_all(response.as_bytes())
                .await
                .expect("write response header");
            stream.write_all(body).await.expect("write response body");
        });

        let options = DoctorOptions {
            tunnel_node_url: Some(format!("http://{addr}/old-path?ignored=true")),
        };
        let report = run_with_options(&full_config(Some(8086)), &options).await;
        server.await.expect("health probe server task");

        let health = report
            .items
            .iter()
            .find(|item| item.id == crate::readiness::FULL_TUNNEL_HEALTH)
            .expect("health item");
        assert!(matches!(health.level, DoctorLevel::Ok));
        assert!(health.detail.contains("full-tunnel capabilities"));
    }
}
