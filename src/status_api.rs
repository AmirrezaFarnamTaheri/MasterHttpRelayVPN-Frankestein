use crate::domain_fronter::{DomainFronter, StatsSnapshot};
use std::sync::Arc;
use tokio::net::TcpListener;

#[derive(Debug, thiserror::Error)]
pub enum StatusApiError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
}

pub fn render_status_json(
    mode: &str,
    http_listen: (&str, u16),
    socks_listen: Option<(&str, u16)>,
    stats: Option<StatsSnapshot>,
) -> String {
    let (hh, hp) = http_listen;
    let socks5 = socks_listen.map(|(sh, sp)| format!("{sh}:{sp}"));
    let stats_json = stats.map(|s| {
        serde_json::json!({
            "relay_calls": s.relay_calls,
            "relay_failures": s.relay_failures,
            "cache_hits": s.cache_hits,
            "cache_misses": s.cache_misses,
            "cache_bytes": s.cache_bytes,
            "bytes_relayed": s.bytes_relayed,
            "coalesced": s.coalesced,
            "scripts_total": s.total_scripts,
            "scripts_blacklisted": s.blacklisted_scripts,
            "today_calls": s.today_calls,
            "today_bytes": s.today_bytes,
            "today_reset_secs": s.today_reset_secs,
            "degrade_level": s.degrade_level,
            "degrade_reason": std::str::from_utf8(&s.degrade_reason)
                .unwrap_or("")
                .trim_matches(char::from(0))
                .trim(),
        })
    });
    serde_json::json!({
        "ok": true,
        "mode": mode,
        "http": format!("{hh}:{hp}"),
        "socks5": socks5,
        "stats": stats_json,
    })
    .to_string()
}

/// Minimal local status endpoint.
///
/// - `GET /health` → `ok`
/// - `GET /status` → json snapshot
///
/// This is intentionally dependency-free (no HTTP framework) to keep the
/// binary small and avoid adding new transitive risks.
pub async fn serve_status_api(
    bind_host: &str,
    port: u16,
    mode: String,
    http_listen: (String, u16),
    socks_listen: Option<(String, u16)>,
    fronter: Option<Arc<DomainFronter>>,
) -> Result<(), StatusApiError> {
    let addr = format!("{}:{}", bind_host, port);
    let listener = TcpListener::bind(&addr).await?;
    tracing::info!("status api: http://{}/status (local)", addr);

    loop {
        let (mut sock, _peer) = listener.accept().await?;
        let mode = mode.clone();
        let http_listen = http_listen.clone();
        let socks_listen = socks_listen.clone();
        let fronter = fronter.clone();
        tokio::spawn(async move {
            use tokio::io::{AsyncReadExt, AsyncWriteExt};
            let mut buf = [0u8; 2048];
            let n = match sock.read(&mut buf).await {
                Ok(n) => n,
                Err(_) => return,
            };
            if n == 0 {
                return;
            }
            let req = String::from_utf8_lossy(&buf[..n]);
            let first = req.lines().next().unwrap_or("");
            let path = first.split_whitespace().nth(1).unwrap_or("/");

            let (status_line, body, ctype) = match path {
                "/health" => (
                    "HTTP/1.1 200 OK",
                    "ok\n".to_string(),
                    "text/plain; charset=utf-8",
                ),
                "/status" => {
                    let stats = fronter.as_ref().map(|f| f.snapshot_stats());
                    let json = render_status_json(
                        &mode,
                        (&http_listen.0, http_listen.1),
                        socks_listen
                            .as_ref()
                            .map(|(host, port)| (host.as_str(), *port)),
                        stats,
                    );
                    ("HTTP/1.1 200 OK", json, "application/json; charset=utf-8")
                }
                _ => (
                    "HTTP/1.1 404 Not Found",
                    "not found\n".to_string(),
                    "text/plain; charset=utf-8",
                ),
            };

            let resp = format!(
                "{status}\r\nContent-Type: {ctype}\r\nContent-Length: {len}\r\nConnection: close\r\n\r\n{body}",
                status = status_line,
                ctype = ctype,
                len = body.len(),
                body = body
            );
            let _ = sock.write_all(resp.as_bytes()).await;
            let _ = sock.flush().await;
        });
    }
}
