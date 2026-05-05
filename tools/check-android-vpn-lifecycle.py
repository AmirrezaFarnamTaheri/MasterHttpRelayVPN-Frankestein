#!/usr/bin/env python3
"""Static Android VPN lifecycle guard.

This no-Gradle check protects the disconnect fixes that avoid Android
ACTION_STOP/stopService lifecycle races and tun2proxy/Rust-runtime teardown
use-after-free races. It is intentionally textual: local repo-sanity can run it
without downloading Android/Gradle tooling, while CI/pre-provisioned JVM/device
tests remain the deeper executable contract.
"""

from __future__ import annotations

import re
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
MAIN_ACTIVITY = ROOT / "android/app/src/main/java/com/farnam/mhrvf/MainActivity.kt"
VPN_SERVICE = ROOT / "android/app/src/main/java/com/farnam/mhrvf/MhrvVpnService.kt"


def die(message: str) -> None:
    raise SystemExit(f"android vpn lifecycle check failed: {message}")


def require_contains(text: str, needle: str, path: Path) -> None:
    if needle not in text:
        die(f"{path.relative_to(ROOT)} is missing required marker: {needle!r}")


def require_order(text: str, markers: list[str], path: Path) -> None:
    cursor = -1
    for marker in markers:
        idx = text.find(marker, cursor + 1)
        if idx == -1:
            die(f"{path.relative_to(ROOT)} is missing ordered marker: {marker!r}")
        if idx <= cursor:
            die(f"{path.relative_to(ROOT)} marker out of order: {marker!r}")
        cursor = idx


def strip_kotlin_comments(text: str) -> str:
    text = re.sub(r"/\*.*?\*/", "", text, flags=re.S)
    return "\n".join(line.split("//", 1)[0] for line in text.splitlines())


def check_main_activity() -> None:
    text = MAIN_ACTIVITY.read_text(encoding="utf-8")
    code = strip_kotlin_comments(text)

    if "stopService(" in code:
        die(
            "MainActivity.kt must not call stopService() for Disconnect; "
            "ACTION_STOP + service stopSelf() is the single lifecycle owner"
        )

    require_order(
        code,
        [
            "onStop = {",
            "val stopAction = Intent(this, MhrvVpnService::class.java)",
            ".setAction(MhrvVpnService.ACTION_STOP)",
            "startService(stopAction)",
        ],
        MAIN_ACTIVITY,
    )
    require_contains(text, "ACTION_STOP alone", MAIN_ACTIVITY)
    require_contains(text, "stopSelf()", MAIN_ACTIVITY)


def check_vpn_service() -> None:
    text = VPN_SERVICE.read_text(encoding="utf-8")

    match = re.search(r"private fun teardown\(\) \{(?P<body>.*?)\n    \}\n    override fun onDestroy", text, re.S)
    if not match:
        die("could not locate MhrvVpnService.teardown() body")
    body = match.group("body")

    require_order(
        body,
        [
            "val handle = proxyHandle",
            "proxyHandle = 0L",
            "Native.stopProxy(handle)",
            "Tun2proxy.stop()",
            "tun?.close()",
            "tun2proxyThread?.join(4_000)",
            "VpnState.setProxyHandle(0L)",
            "VpnState.setRunning(false)",
        ],
        VPN_SERVICE,
    )
    require_contains(body, "With step 1 having closed its upstream socket", VPN_SERVICE)
    require_contains(body, "tornDown.compareAndSet(false, true)", VPN_SERVICE)

    stale_phrases = [
        "runtime shutdown below will knock",
        "stopProxy(handle)" + "\n" + "        // Flip UI state last",
    ]
    for phrase in stale_phrases:
        if phrase in body:
            die(f"MhrvVpnService.kt still contains stale teardown shape/claim: {phrase!r}")


def main() -> int:
    check_main_activity()
    check_vpn_service()
    print("android vpn lifecycle check: ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
