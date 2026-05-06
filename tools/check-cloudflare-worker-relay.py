#!/usr/bin/env python3
"""Static guard for the Apps Script + Cloudflare Worker relay bridge."""

from __future__ import annotations

import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
WORKER = ROOT / "tools" / "cloudflare-worker-json-relay" / "worker.js"
WORKER_README = ROOT / "tools" / "cloudflare-worker-json-relay" / "README.md"
APP_SCRIPT = ROOT / "assets" / "apps_script" / "CodeCloudflareWorker.gs"
DOCS = [
    ROOT / "docs" / "cloudflare-worker-json-relay.md",
    ROOT / "docs" / "cfw-reference-audit.md",
    ROOT / "docs" / "relay-modes.md",
    ROOT / "docs" / "index.md",
    ROOT / "docs" / "release-checklist.md",
    ROOT / "docs" / "ui-desktop.md",
    ROOT / "README.md",
]
UI = ROOT / "src" / "bin" / "ui.rs"
UI_HELP = ROOT / "src" / "bin" / "ui_help.rs"


def die(msg: str) -> None:
    print(f"Cloudflare Worker relay check failed: {msg}", file=sys.stderr)
    raise SystemExit(1)


def read(path: Path) -> str:
    if not path.is_file():
        die(f"missing file: {path.relative_to(ROOT)}")
    return path.read_text(encoding="utf-8")


def require(text: str, needle: str, label: str) -> None:
    if needle not in text:
        die(f"missing {label}: {needle!r}")


def main() -> int:
    worker = read(WORKER)
    worker_readme = read(WORKER_README)
    script = read(APP_SCRIPT)
    ui = read(UI)
    ui_help = read(UI_HELP)
    docs = {path.name: read(path) for path in DOCS}

    # Worker must not be an open public fetch proxy.
    require(worker, "WORKER_AUTH_KEY", "Worker auth env var")
    require(worker, "req.wk !== workerAuthKey", "Worker request auth check")
    require(worker, "x-relay-hop", "Worker loop guard header")
    require(worker, "loop detected", "Worker loop guard response")
    require(worker, 'targetUrl.hostname.endsWith(".workers.dev")', "Worker self-fetch block")
    require(worker, 'targetUrl.protocol !== "http:" && targetUrl.protocol !== "https:"', "Worker URL scheme validation")
    for header in (
        "forwarded",
        "x-forwarded-for",
        "x-forwarded-host",
        "x-forwarded-proto",
        "x-forwarded-port",
        "x-real-ip",
        "cf-connecting-ip",
        "cf-ipcountry",
        "cf-ray",
        "cf-visitor",
    ):
        require(worker, f'"{header}"', f"Worker strips {header}")
    require(worker, "base64ToBytes", "Worker base64 body decoder")
    require(worker, "bytesToBase64", "Worker base64 response encoder")
    require(worker, 'headers: { "content-type": "application/json" }', "Worker JSON response content-type")

    # Apps Script bridge must preserve the normal client protocol while adding
    # a separate Worker-hop secret and compatibility marker.
    require(script, 'const HELPER_KIND = "apps_script_cloudflare_worker";', "Apps Script helper kind")
    require(script, "cloudflare_worker_exit", "Apps Script helper feature marker")
    require(script, "WORKER_URL", "Apps Script Worker URL placeholder")
    require(script, "WORKER_AUTH_KEY", "Apps Script Worker secret")
    require(script, "AUTH_KEY", "Apps Script client secret")
    require(script, "wk: WORKER_AUTH_KEY", "Apps Script passes Worker-hop key")
    require(script, "UrlFetchApp.fetch(WORKER_URL", "Apps Script single Worker fetch")
    require(script, "UrlFetchApp.fetchAll", "Apps Script batch Worker fetch")
    require(script, "SAFE_REPLAY_METHODS", "Apps Script safe replay fallback")
    require(script, "apps_script_cloudflare_worker", "Apps Script compat probe kind")
    require(script, "e.parameter.compat === \"1\"", "Apps Script compat probe branch")
    require(script, "ContentService", "Apps Script ContentService output")

    # UI and docs must describe this as optional apps_script egress, not a
    # separate native mode or full tunnel replacement.
    for label, text in docs.items():
        require(text, "Cloudflare", f"{label} Cloudflare mention")
    require(docs["cloudflare-worker-json-relay.md"], "mhrv-f client -> Apps Script -> Cloudflare Worker -> target website", "Worker flow docs")
    require(docs["cloudflare-worker-json-relay.md"], "not a replacement for full tunnel", "Worker limit docs")
    require(docs["cloudflare-worker-json-relay.md"], "does not add raw TCP/UDP support", "Worker TCP/UDP limit docs")
    require(docs["cloudflare-worker-json-relay.md"], "Keep the two secrets separate", "two-secret docs")
    require(docs["cloudflare-worker-json-relay.md"], "apps_script_cloudflare_worker", "compat kind docs")
    require(docs["cfw-reference-audit.md"], "not an older donor copy", "donor-copy warning")
    require(docs["relay-modes.md"], "Keep desktop mode as `apps_script`", "relay modes same-mode docs")
    require(docs["relay-modes.md"], "another quota surface and another secret", "relay modes quota/secret warning")
    require(docs["release-checklist.md"], "CodeCloudflareWorker.gs", "release checklist helper")
    require(docs["release-checklist.md"], "HELPER_KIND", "release checklist compat markers")
    require(docs["ui-desktop.md"], "Cloudflare Worker exit", "desktop docs backend tool")
    require(docs["README.md"], "optional Cloudflare Worker JSON relay", "README optional Worker pointer")
    require(worker_readme, "WORKER_AUTH_KEY", "Worker tool README auth docs")
    require(worker_readme, "CodeCloudflareWorker.gs", "Worker tool README Apps Script bridge")
    require(ui, "backend_tool_entries", "desktop UI backend tool catalog use")
    require(ui_help, "Cloudflare Worker exit", "desktop UI backend tool label")
    require(ui_help, "tools/cloudflare-worker-json-relay", "desktop UI Worker path")
    require(ui_help, "CodeCloudflareWorker.gs", "desktop UI Apps Script bridge")

    print("Cloudflare Worker relay check: ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
