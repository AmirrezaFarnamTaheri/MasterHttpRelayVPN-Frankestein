use crate::domain_fronter::{DomainFronter, StatsSnapshot};
use std::sync::Arc;
use tokio::net::TcpListener;

#[derive(Debug, thiserror::Error)]
pub enum StatusApiError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
}

pub fn stats_snapshot_json_value(s: StatsSnapshot) -> serde_json::Value {
    let degrade_reason = std::str::from_utf8(&s.degrade_reason)
        .unwrap_or("")
        .trim_matches(char::from(0))
        .trim();
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
        // Android historically consumed these names. Keep them as aliases so
        // all status consumers can use one renderer without a compatibility
        // fork in JNI.
        "total_scripts": s.total_scripts,
        "blacklisted_scripts": s.blacklisted_scripts,
        "today_calls": s.today_calls,
        "today_bytes": s.today_bytes,
        "today_reset_secs": s.today_reset_secs,
        "degrade_level": s.degrade_level,
        "degrade_reason": degrade_reason,
    })
}

pub fn render_status_json(
    mode: &str,
    http_listen: (&str, u16),
    socks_listen: Option<(&str, u16)>,
    stats: Option<StatsSnapshot>,
) -> String {
    let (hh, hp) = http_listen;
    let socks5 = socks_listen.map(|(sh, sp)| format!("{sh}:{sp}"));
    let stats_json = stats.map(stats_snapshot_json_value);
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

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_stats() -> StatsSnapshot {
        let mut reason = [0u8; 32];
        reason[..7].copy_from_slice(b"warming");
        StatsSnapshot {
            relay_calls: 10,
            relay_failures: 2,
            coalesced: 3,
            bytes_relayed: 4096,
            cache_hits: 4,
            cache_misses: 6,
            cache_bytes: 1024,
            blacklisted_scripts: 1,
            total_scripts: 5,
            today_calls: 7,
            today_bytes: 2048,
            today_reset_secs: 3600,
            degrade_level: 1,
            degrade_reason: reason,
        }
    }

    #[test]
    fn stats_snapshot_json_keeps_canonical_and_android_alias_keys() {
        let v = stats_snapshot_json_value(sample_stats());
        assert_eq!(v["relay_calls"], 10);
        assert_eq!(v["scripts_total"], 5);
        assert_eq!(v["scripts_blacklisted"], 1);
        assert_eq!(v["total_scripts"], 5);
        assert_eq!(v["blacklisted_scripts"], 1);
        assert_eq!(v["today_calls"], 7);
        assert_eq!(v["degrade_reason"], "warming");
    }

    #[test]
    fn render_status_json_uses_shared_stats_renderer() {
        let text = render_status_json(
            "apps_script",
            ("127.0.0.1", 8085),
            Some(("127.0.0.1", 8086)),
            Some(sample_stats()),
        );
        let v: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(v["ok"], true);
        assert_eq!(v["mode"], "apps_script");
        assert_eq!(v["http"], "127.0.0.1:8085");
        assert_eq!(v["socks5"], "127.0.0.1:8086");
        assert_eq!(v["stats"]["scripts_total"], 5);
        assert_eq!(v["stats"]["total_scripts"], 5);
    }
}
