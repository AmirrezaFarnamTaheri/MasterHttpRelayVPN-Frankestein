#!/usr/bin/env python3
"""Static guard for full-mode adaptive coalescing defaults.

The v1.9.8/v1.9.9 low-latency profile deliberately uses a short 10 ms client
coalesce step, a 1000 ms max, and matching tunnel-node straggler settle timing.
Rust config serialization still stores zero when omitted, meaning "compiled
runtime default"; Android saves concrete 10/1000 values. This guard keeps those
surfaces aligned.
"""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
TUNNEL_CLIENT = ROOT / "src" / "tunnel_client.rs"
PROXY_SERVER = ROOT / "src" / "proxy_server.rs"
TUNNEL_NODE = ROOT / "tunnel-node" / "src" / "main.rs"
ANDROID = ROOT / "android" / "app" / "src" / "main" / "java" / "com" / "farnam" / "mhrvf" / "ConfigStore.kt"
PLATFORM_JSON = ROOT / "docs" / "platform-defaults.json"
ADVANCED_DOCS = ROOT / "docs" / "advanced-options.md"
PLATFORM_DOCS = ROOT / "docs" / "platform-defaults.md"
CHANGELOG = ROOT / "docs" / "changelog" / "batch-6-upstream-v199-stability-tuning-2026-05-04.md"
EXAMPLES = [
    ROOT / "config.example.json",
    ROOT / "config.full.example.json",
]


def die(msg: str) -> None:
    print(f"coalesce tuning check failed: {msg}", file=sys.stderr)
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
    tunnel_client = read(TUNNEL_CLIENT)
    proxy_server = read(PROXY_SERVER)
    tunnel_node = read(TUNNEL_NODE)
    android = read(ANDROID)
    advanced = read(ADVANCED_DOCS)
    platform_docs = read(PLATFORM_DOCS)
    changelog = read(CHANGELOG)
    platform = json.loads(read(PLATFORM_JSON))
    examples = {path.name: json.loads(read(path)) for path in EXAMPLES}

    # Rust full-mode client runtime defaults.
    require(tunnel_client, "const DEFAULT_COALESCE_STEP_MS: u64 = 10;", "Rust client 10ms coalesce step")
    require(tunnel_client, "const DEFAULT_COALESCE_MAX_MS: u64 = 1000;", "Rust client 1000ms coalesce max")
    require(tunnel_client, "10 ms catches ops", "Rust client tuning rationale")
    require(tunnel_client, "coalesce_max_ms.max(coalesce_step_ms.max(1))", "max >= step runtime clamp")

    # ProxyServer translates config zero into compiled runtime defaults.
    require_regex(
        proxy_server,
        r"coalesce_step_ms:\s*if\s+config\.coalesce_step_ms\s*>\s*0\s*\{.*?config\.coalesce_step_ms\s+as\s+u64.*?\}\s*else\s*\{\s*10\s*\}",
        "ProxyServer coalesce_step_ms zero fallback",
    )
    require_regex(
        proxy_server,
        r"coalesce_max_ms:\s*if\s+config\.coalesce_max_ms\s*>\s*0\s*\{.*?config\.coalesce_max_ms\s+as\s+u64.*?\}\s*else\s*\{\s*1000\s*\}",
        "ProxyServer coalesce_max_ms zero fallback",
    )

    # Android stores concrete defaults so mobile exports match the compiled
    # low-latency profile rather than writing Rust's serde-zero sentinel.
    require(android, "val coalesceStepMs: Int = 10,", "Android coalesce step default")
    require(android, "val coalesceMaxMs: Int = 1000,", "Android coalesce max default")
    require(android, 'obj.optInt("coalesce_step_ms", 10)', "Android coalesce step import fallback")
    require(android, 'obj.optInt("coalesce_max_ms", 1000)', "Android coalesce max import fallback")
    require(android, 'if (coalesceStepMs != 10) put("coalesce_step_ms", coalesceStepMs)', "Android omits default coalesce step")
    require(android, 'if (coalesceMaxMs != 1000) put("coalesce_max_ms", coalesceMaxMs)', "Android omits default coalesce max")

    # User-facing examples must not keep the old conservative 40 ms profile as
    # the default. The old value remains documented only as an explicit opt-in.
    for name, example in examples.items():
        if example.get("coalesce_step_ms") != 10:
            die(f"{name}: coalesce_step_ms must be 10")
        if example.get("coalesce_max_ms") != 1000:
            die(f"{name}: coalesce_max_ms must be 1000")

    # tunnel-node must match client step/cap for the return-leg settle profile.
    require(tunnel_node, "const STRAGGLER_SETTLE_STEP: Duration = Duration::from_millis(10);", "tunnel-node 10ms settle step")
    require(tunnel_node, "const STRAGGLER_SETTLE_MAX: Duration = Duration::from_millis(1000);", "tunnel-node 1000ms settle max")
    require(tunnel_node, "short 10 ms settle window", "tunnel-node tuning rationale")
    require(tunnel_node, "wider cap still packs staggered upstream replies", "tunnel-node quota/latency rationale")

    # Canonical docs/defaults contract.
    rust_cli = platform.get("rust_desktop_cli", {})
    android_defaults = platform.get("android", {})
    if rust_cli.get("coalesce_step_ms_when_field_absent") != 0:
        die("Rust serde coalesce_step_ms absent-field value must stay 0 sentinel")
    if rust_cli.get("coalesce_max_ms_when_field_absent") != 0:
        die("Rust serde coalesce_max_ms absent-field value must stay 0 sentinel")
    if android_defaults.get("coalesce_step_ms_default") != 10:
        die("Android platform-defaults coalesce_step_ms_default must stay 10")
    if android_defaults.get("coalesce_max_ms_default") != 1000:
        die("Android platform-defaults coalesce_max_ms_default must stay 1000")
    require(rust_cli.get("coalesce_note", ""), "10/1000 ms", "platform JSON coalesce rationale")
    require(platform.get("android", {}).get("rationale_coalesce_ms_vs_rust", ""), "`10`/`1000`", "platform JSON Android coalesce rationale")

    require(advanced, "Defaults are `10` and `1000`", "advanced docs current defaults")
    require(advanced, "coalesce_step_ms` to `40`", "advanced docs old-behavior override")
    require(advanced, "download-heavy batches", "advanced docs latency rationale")
    require(platform_docs, "`coalesce_step_ms_default` | 10", "platform docs Android coalesce step")
    require(platform_docs, "`coalesce_max_ms_default` | 1000", "platform docs Android coalesce max")
    require(platform_docs, "`10`/`1000`", "platform docs Android/Rust rationale")
    require(changelog, "`coalesce_step_ms = 10`", "upstream tuning changelog step")
    require(changelog, "`coalesce_max_ms = 1000`", "upstream tuning changelog max")
    require(changelog, "tunnel-node straggler settle constants now agree", "upstream tuning changelog parity note")

    print("coalesce tuning check: ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
