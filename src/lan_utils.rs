//! LAN sharing helpers for desktop and future mobile parity surfaces.
//!
//! `detect_lan_ip()` uses the common UDP connect trick: bind a UDP socket,
//! connect it to a public address, and ask the OS which local source address
//! it selected. UDP connect does not send a packet here; it only commits route
//! selection in the kernel.

use std::net::{IpAddr, UdpSocket};

/// Return the host IPv4/IPv6 address the OS would use for normal outbound
/// traffic. `None` means there is no usable route or local address.
pub fn detect_lan_ip() -> Option<IpAddr> {
    let sock = UdpSocket::bind(("0.0.0.0", 0)).ok()?;
    sock.connect(("1.1.1.1", 80)).ok()?;
    let local = sock.local_addr().ok()?.ip();
    if local.is_unspecified() {
        None
    } else {
        Some(local)
    }
}

/// True when a listener bind string exposes all interfaces.
pub fn is_share_on_lan(listen_host: &str) -> bool {
    matches!(listen_host.trim(), "0.0.0.0" | "::" | "[::]")
}

/// True when a listener bind string is loopback-only.
pub fn is_loopback_only(listen_host: &str) -> bool {
    let trimmed = listen_host.trim().to_ascii_lowercase();
    matches!(
        trimmed.as_str(),
        "127.0.0.1" | "localhost" | "::1" | "[::1]"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn share_on_lan_recognizes_wildcards() {
        assert!(is_share_on_lan("0.0.0.0"));
        assert!(is_share_on_lan(" 0.0.0.0 "));
        assert!(is_share_on_lan("::"));
        assert!(is_share_on_lan("[::]"));
        assert!(!is_share_on_lan("127.0.0.1"));
        assert!(!is_share_on_lan("localhost"));
        assert!(!is_share_on_lan("192.168.1.42"));
        assert!(!is_share_on_lan(""));
    }

    #[test]
    fn loopback_only_recognizes_local_names() {
        assert!(is_loopback_only("127.0.0.1"));
        assert!(is_loopback_only("localhost"));
        assert!(is_loopback_only("LocalHost"));
        assert!(is_loopback_only("::1"));
        assert!(is_loopback_only("[::1]"));
        assert!(!is_loopback_only("0.0.0.0"));
        assert!(!is_loopback_only("192.168.1.42"));
        assert!(!is_loopback_only(""));
    }

    #[test]
    fn detect_lan_ip_never_returns_unspecified() {
        if let Some(ip) = detect_lan_ip() {
            assert!(!ip.is_unspecified(), "got unspecified address: {ip}");
        }
    }
}
