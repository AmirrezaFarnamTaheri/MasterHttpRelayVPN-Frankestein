#!/usr/bin/env python3
"""Static guard for Apps Script relay hardening.

Protects the upstream v1.9.6/v1.9.7 helper fixes:

- `doGet` must be single-definition and ContentService-based, not HtmlService
  wrapped;
- relay JSON responses must use ContentService JSON;
- identity/IP-leak headers must be stripped in every helper;
- batch fallback may replay only safe methods after `fetchAll` fails as a
  whole;
- the Rust client must keep the `goog.script.init` / `userHtml` unwrap tests.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
HELPERS = [
    ROOT / "assets" / "apps_script" / "Code.gs",
    ROOT / "assets" / "apps_script" / "CodeFull.gs",
    ROOT / "assets" / "apps_script" / "CodeCloudflareWorker.gs",
]
RUST = ROOT / "src" / "domain_fronter.rs"

IDENTITY_HEADERS = [
    "forwarded",
    "via",
    "x-forwarded-for",
    "x-forwarded-host",
    "x-forwarded-proto",
    "x-forwarded-port",
    "x-forwarded-server",
    "x-forwarded-ssl",
    "x-real-ip",
    "x-client-ip",
    "x-originating-ip",
    "true-client-ip",
    "cf-connecting-ip",
    "fastly-client-ip",
    "x-cluster-client-ip",
    "x-proxyuser-ip",
    "x-original-forwarded-for",
    "client-ip",
]


def die(msg: str) -> None:
    print(f"Apps Script hardening check failed: {msg}", file=sys.stderr)
    raise SystemExit(1)


def require(text: str, needle: str, label: str) -> None:
    if needle not in text:
        die(f"missing {label}: {needle!r}")


def find_function(text: str, name: str) -> str:
    match = re.search(rf"function\s+{re.escape(name)}\s*\([^)]*\)\s*\{{", text)
    if not match:
        die(f"missing function {name}")
    brace = text.find("{", match.end() - 1)
    depth = 0
    for idx in range(brace, len(text)):
        ch = text[idx]
        if ch == "{":
            depth += 1
        elif ch == "}":
            depth -= 1
            if depth == 0:
                return text[match.start() : idx + 1]
    die(f"unterminated function {name}")


def check_helper(path: Path) -> None:
    if not path.is_file():
        die(f"missing helper {path}")
    text = path.read_text(encoding="utf-8")
    rel = path.relative_to(ROOT).as_posix()

    if len(re.findall(r"\bfunction\s+doGet\s*\(", text)) != 1:
        die(f"{rel}: expected exactly one doGet")
    if "HtmlService.createHtmlOutput" in text:
        die(f"{rel}: HtmlService.createHtmlOutput must not be used")

    do_get = find_function(text, "doGet")
    require(do_get, "ContentService", f"{rel} doGet ContentService")
    require(do_get, "ContentService.MimeType.HTML", f"{rel} doGet HTML MIME")
    require(do_get, "_compatInfo()", f"{rel} doGet compat endpoint")

    json_fn = find_function(text, "_json")
    require(json_fn, "ContentService", f"{rel} _json ContentService")
    require(json_fn, "ContentService.MimeType.JSON", f"{rel} _json JSON MIME")

    require(text, "const SAFE_REPLAY_METHODS = { GET: 1, HEAD: 1, OPTIONS: 1 };", f"{rel} safe replay set")
    require(text, "unsafe method not replayed", f"{rel} unsafe replay refusal")
    require(text, "UrlFetchApp.fetchAll", f"{rel} fetchAll path")
    require(text, "UrlFetchApp.fetch(fallbackUrl, fallbackOpts)", f"{rel} per-item fallback")
    require(text, '"bad item"', f"{rel} bad batch-item validation")
    require(text, '"bad url"', f"{rel} bad URL validation")
    require(text, "var responseMap = {}", f"{rel} original-index response map")

    lower = text.lower()
    for header in IDENTITY_HEADERS:
        if f'"{header}"' not in lower and f"{header}: 1" not in lower:
            die(f"{rel}: missing identity header strip for {header}")


def check_rust_unwrap() -> None:
    if not RUST.is_file():
        die(f"missing Rust source {RUST}")
    text = RUST.read_text(encoding="utf-8")
    for needle in [
        "fn extract_apps_script_user_html",
        'let marker = "goog.script.init(\\""',
        "fn decode_js_string_escapes",
        "decode_js_string_escapes_supports_apps_script_forms",
        "extract_apps_script_user_html_unwraps_goog_init",
        "parse_relay_json_unwraps_goog_script_init",
    ]:
        require(text, needle, f"Rust Apps Script unwrap contract {needle}")


def main() -> int:
    for helper in HELPERS:
        check_helper(helper)
    check_rust_unwrap()
    print("Apps Script hardening check: ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
