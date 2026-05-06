#!/usr/bin/env python3
"""Static guard for the friendly LAN-share UI contract.

The desktop UI intentionally exposes LAN sharing as a safe, copyable workflow:
a checkbox owns the normal loopback/all-interface toggle, a custom bind address
is preserved instead of clobbered, and docs explain the route-table IP detection
without pretending that LAN exposure is harmless.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
LAN_UTILS = ROOT / "src" / "lan_utils.rs"
LIB = ROOT / "src" / "lib.rs"
UI = ROOT / "src" / "bin" / "ui.rs"
DOC = ROOT / "docs" / "sharing-and-per-app-routing.md"


def die(msg: str) -> None:
    print(f"LAN sharing UI check failed: {msg}", file=sys.stderr)
    raise SystemExit(1)


def read(path: Path) -> str:
    if not path.is_file():
        die(f"missing file: {path.relative_to(ROOT)}")
    return path.read_text(encoding="utf-8")


def require(text: str, needle: str, label: str) -> None:
    if needle not in text:
        die(f"missing {label}: {needle!r}")


def require_regex(text: str, pattern: str, label: str) -> None:
    if not re.search(pattern, text, flags=re.S):
        die(f"missing {label}: /{pattern}/")


def main() -> int:
    lan = read(LAN_UTILS)
    lib = read(LIB)
    ui = read(UI)
    doc = read(DOC)

    # Shared helper surface.
    require(lib, "pub mod lan_utils;", "lan_utils module export")
    for fn_name in ("detect_lan_ip", "is_share_on_lan", "is_loopback_only"):
        require(lan, f"pub fn {fn_name}", f"lan utility function {fn_name}")
        require(ui, fn_name, f"desktop UI use of {fn_name}")
    require(lan, 'UdpSocket::bind(("0.0.0.0", 0))', "UDP route-table bind trick")
    require(lan, 'sock.connect(("1.1.1.1", 80))', "UDP route-table connect target")
    require(lan, "local.is_unspecified()", "unspecified address rejection")
    for test_name in (
        "share_on_lan_recognizes_wildcards",
        "loopback_only_recognizes_local_names",
        "detect_lan_ip_never_returns_unspecified",
    ):
        require(lan, test_name, f"lan_utils regression test {test_name}")

    # Desktop UX contract.
    require(ui, "Sharing and per-app routing", "sharing section")
    require(ui, "listen_host_snapshot", "listen_host snapshot before UI mutation")
    require(ui, "custom_bind", "custom bind detection")
    require(ui, "Custom bind:", "custom bind badge")
    require(ui, "Save cannot replace that custom interface", "custom bind overwrite protection text")
    require(ui, "Share with other devices on my Wi-Fi / network", "friendly LAN-share checkbox")
    require(ui, "macOS may show a Firewall prompt", "macOS firewall tooltip")
    require_regex(
        ui,
        r"self\.form\.listen_host\s*=\s*if\s+share\s*\{\s*\"0\.0\.0\.0\"\.into\(\)\s*\}\s*else\s*\{\s*\"127\.0\.0\.1\"\.into\(\)",
        "checkbox-owned listen_host transition",
    )
    require(ui, "LAN exposed", "LAN exposed state badge")
    require(ui, "local-only", "local-only state badge")
    require(ui, "this-device-LAN-IP", "LAN IP fallback placeholder")
    require(ui, 'format!("http://{}:{}"', "HTTP endpoint formatting")
    require(ui, 'format!("socks5://{}:{}"', "SOCKS5 endpoint formatting")
    require(ui, 'small_button("copy HTTP")', "copy HTTP action")
    require(ui, 'small_button("copy SOCKS")', "copy SOCKS action")
    require(ui, "LAN address was detected from the OS route table; no probe packet is sent.", "LAN detection reassurance")
    require(ui, "active Wi-Fi/Ethernet IPv4", "LAN detection fallback guidance")

    # Docs must describe the same behavior and security boundary.
    require(doc, "Share with other devices on my Wi-Fi / network", "docs LAN-share checkbox label")
    require(doc, 'listen_host =\n   "0.0.0.0"', "docs automatic listen_host change")
    require(doc, "copyable HTTP/SOCKS endpoints", "docs copyable endpoint behavior")
    require(doc, "UDP socket route lookup", "docs UDP route lookup explanation")
    require(doc, "does not send a network packet", "docs no-packet reassurance")
    require(doc, "this-device-LAN-IP", "docs fallback placeholder")
    require(doc, "Custom bind", "docs custom bind preservation")
    require(doc, "does not let the LAN checkbox overwrite it", "docs custom bind overwrite protection")
    require(doc, "lan_allowlist", "docs LAN allowlist")
    require_regex(doc, r"SOCKS5\s+(?:has no header preface, so it cannot use this|cannot use\s+this)\s+token", "docs SOCKS/token limitation")

    print("LAN sharing UI check: ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
