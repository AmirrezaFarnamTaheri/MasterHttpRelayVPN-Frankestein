//! Central routing policy (single source of truth).
//!
//! The proxy has multiple transport paths (passthrough, full-tunnel, SNI-rewrite,
//! MITM+relay, plain relay). Keeping the decision logic in one place prevents
//! drift across HTTP CONNECT vs SOCKS5 vs UI helpers.

use crate::config::Mode;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RouteDecision {
    /// Raw TCP passthrough (optionally via upstream SOCKS5)
    Passthrough,
    /// Full tunnel mux (Apps Script + tunnel node). No MITM.
    FullTunnel,
    /// TLS SNI-rewrite tunnel to Google edge (direct).
    SniRewrite,
    /// Apps Script relay with MITM (TLS detected).
    MitmRelayTls,
    /// Apps Script relay of plain HTTP bytes (non-TLS but HTTP-looking).
    RelayPlainHttp { scheme: &'static str },
}

/// Decide which path to use for a CONNECT/SOCKS flow.
///
/// `peek_first` is the first byte of client payload if available (0x16 for TLS),
/// and `looks_like_http` indicates an HTTP method prefix.
#[allow(clippy::too_many_arguments)]
pub fn decide_route(
    mode: Mode,
    host: &str,
    port: u16,
    youtube_via_relay: bool,
    matches_passthrough: bool,
    matches_sni_rewrite: bool,
    forced: Option<RouteDecision>,
    peek_first: Option<u8>,
    looks_like_http: bool,
) -> RouteDecision {
    if let Some(f) = forced {
        let _ = (host, port, youtube_via_relay);
        return f;
    }
    if matches_passthrough {
        return RouteDecision::Passthrough;
    }
    if mode == Mode::Full {
        return RouteDecision::FullTunnel;
    }
    if matches_sni_rewrite {
        return RouteDecision::SniRewrite;
    }
    if mode == Mode::GoogleOnly {
        // No relay available; everything else is passthrough.
        let _ = (host, port, youtube_via_relay);
        return RouteDecision::Passthrough;
    }
    if peek_first == Some(0x16) {
        return RouteDecision::MitmRelayTls;
    }
    if looks_like_http {
        let scheme = if port == 443 { "https" } else { "http" };
        return RouteDecision::RelayPlainHttp { scheme };
    }
    RouteDecision::Passthrough
}
