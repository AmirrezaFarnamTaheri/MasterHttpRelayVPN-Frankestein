#!/usr/bin/env python3
"""Static guard for canonical relay-mode vocabulary.

The config still uses stable wire names (`apps_script`, `vercel_edge`,
`direct`, `full`), but user-facing docs and UI should present the product modes
as Apps Script, Serverless JSON, Direct fronting, and Full tunnel.
"""

from __future__ import annotations

import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
RELAY_MODES = ROOT / "docs" / "relay-modes.md"
DOC_INDEX = ROOT / "docs" / "index.md"
README = ROOT / "README.md"
DESKTOP_UI = ROOT / "src" / "bin" / "ui.rs"
DESKTOP_MODE = ROOT / "src" / "bin" / "ui_mode.rs"
ANDROID_HOME = ROOT / "android/app/src/main/java/com/farnam/mhrvf/ui/HomeScreen.kt"
ANDROID_CONFIG = ROOT / "android/app/src/main/java/com/farnam/mhrvf/ConfigStore.kt"


def die(msg: str) -> None:
    print(f"mode vocabulary check failed: {msg}", file=sys.stderr)
    raise SystemExit(1)


def read(path: Path) -> str:
    if not path.is_file():
        die(f"missing file: {path.relative_to(ROOT)}")
    return path.read_text(encoding="utf-8")


def require(text: str, needle: str, label: str) -> None:
    if needle not in text:
        die(f"missing {label}: {needle!r}")


def main() -> int:
    relay_modes = read(RELAY_MODES)
    index = read(DOC_INDEX)
    readme = read(README)
    desktop = read(DESKTOP_UI)
    desktop_mode = read(DESKTOP_MODE)
    android_home = read(ANDROID_HOME)
    android_config = read(ANDROID_CONFIG)

    for mode in ("`apps_script`", "`vercel_edge`", "`direct`", "`full`"):
        require(relay_modes, mode, f"relay modes raw config value {mode}")
    for label in (
        "## Apps Script Mode",
        "## Serverless JSON Mode",
        "## Direct Mode",
        "## Full Mode",
        "Full tunnel",
        "Direct fronting",
    ):
        require(relay_modes, label, f"relay modes label {label}")
    require(relay_modes, "The config mode is `vercel_edge` for compatibility", "Serverless JSON compatibility wording")
    require(relay_modes, "The desktop **Test Relay** action is an Apps Script / serverless JSON probe.", "mode-specific Test Relay warning")

    require(index, "[`docs/relay-modes.md`](relay-modes.md)", "docs index relay-modes link")
    require(index, "Full tunnel without local MITM", "docs index full tunnel row")
    require(index, "Direct fronting groups", "docs index direct fronting link")
    require(readme, "**Apps Script (`apps_script`)**", "README Apps Script mode label")
    require(readme, "**Serverless JSON (`vercel_edge`)**", "README Serverless JSON mode label")
    require(readme, "**Direct (`direct`)**", "README Direct mode label")
    require(readme, "**Full tunnel (`full`)**", "README Full tunnel mode label")

    for marker in (
        '"Apps Script (MITM)"',
        '"Serverless JSON (no VPS)"',
        '"Direct fronting (no relay)"',
        '"Full tunnel (no cert)"',
    ):
        require(desktop, marker, f"desktop mode selector vocabulary {marker}")
    for marker in (
        '"Serverless JSON"',
        '"Direct fronting"',
        '"Full tunnel"',
        "SNI-rewrite path only",
        "No Apps Script credentials",
        "Deploy tools/vercel-json-relay or tools/netlify-json-relay",
    ):
        require(desktop_mode, marker, f"desktop mode dashboard vocabulary {marker}")

    for marker in (
        'val labelApps = "Apps Script (MITM)"',
        'val labelServerless = "Serverless JSON (no VPS)"',
        'val labelDirect = "Direct fronting (no relay)"',
        'val labelFull = "Full tunnel (no cert)"',
        '"No-VPS JSON fetch relay hosted on Vercel or Netlify',
        '"SNI-rewrite only, no relay.',
        '"All traffic tunneled end-to-end through Apps Script + remote tunnel node.',
    ):
        require(android_home, marker, f"Android mode vocabulary {marker}")
    for marker in (
        "enum class Mode { APPS_SCRIPT, SERVERLESS_JSON, DIRECT, FULL }",
        'Mode.APPS_SCRIPT -> "apps_script"',
        'Mode.SERVERLESS_JSON -> "vercel_edge"',
        'Mode.DIRECT -> "direct"',
        'Mode.FULL -> "full"',
        '"vercel_edge" -> Mode.SERVERLESS_JSON',
        '"direct", "google_only" -> Mode.DIRECT',
        '"full" -> Mode.FULL',
    ):
        require(android_config, marker, f"Android mode wire value {marker}")

    print("mode vocabulary check: ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
