#!/usr/bin/env python3
"""Structural report for Nova-style / donor proxy JSON vs mhrv-f config registry.

Batch 2 uses this as a **read-only migration advisor**. It does **not** convert or
import rules into product defaults — compare ``docs/donor-absorption-matrix.md``.

Examples:

  python tools/report-nova-proxy-config.py --demo
  python tools/report-nova-proxy-config.py --path path/to/export.json

Exit code **0** on success or skipped demo path; **1** on invalid JSON or missing registry.
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
REGISTRY = ROOT / "docs" / "config-registry.json"

# Typical Nova donor ``settings.json`` / app-config shaped roots (not mhrv schema).
NOVA_SETTINGS_SIGNATURE = frozenset(
    {
        "listen_port",
        "tun",
        "cloudflare_config",
        "auto_routing",
    }
)


def load_registry_roots() -> frozenset[str]:
    obj = json.loads(REGISTRY.read_text(encoding="utf-8"))
    if not isinstance(obj, dict):
        raise SystemExit("docs/config-registry.json must be an object")
    return frozenset(obj.keys())


def print_nested_preview(obj: dict[str, object], *, max_children: int = 16) -> None:
    print("- Nested shapes at root (preview):")
    any_nested = False
    for k in sorted(obj.keys()):
        v = obj[k]
        if isinstance(v, dict):
            any_nested = True
            ck = sorted(v.keys())
            tail = " …" if len(ck) > max_children else ""
            shown = ck[:max_children]
            print(f"  - `{k}`: object ({len(ck)} keys): {shown}{tail}")
        elif isinstance(v, list):
            any_nested = True
            print(f"  - `{k}`: array (length {len(v)})")
    if not any_nested:
        print("  - (no object/array values at root)")


def analyze_top_level(keys: frozenset[str], registry: frozenset[str]) -> None:
    mhrv_like = bool(keys & {"mode", "config_version"}) or (
        "account_groups" in keys or "vercel" in keys
    )
    if mhrv_like:
        print(
            "- Shape: resembles **mhrv-f** config (has mode / account_groups / vercel-style keys)."
        )
    else:
        print(
            "- Shape: **not** a typical mhrv-f export - treat as external / donor JSON."
        )

    nova_sig_hits = sorted(keys & NOVA_SETTINGS_SIGNATURE)
    if nova_sig_hits:
        print(f"- Nova donor **settings** signature overlap: {nova_sig_hits}")

    name_collisions = sorted(k for k in keys if k in registry)
    if name_collisions:
        print(
            "- **Name collisions** with mhrv registry roots (same key name; "
            "types/semantics may differ): "
            f"{name_collisions}"
        )
        print(
            "  Do not assume values copy 1:1 - e.g. Nova `listen_port` may be a string; "
            "mhrv expects a JSON number."
        )

    external_only = sorted(k for k in keys if k not in registry)
    if external_only:
        print(f"- Keys **outside** mhrv registry roots ({len(external_only)}): {external_only}")
        print(
            "  These cannot be honored by Rust `Config` unless added to the schema; "
            "mhrv-f Android may still **preserve** unknown root blobs on import (see "
            "`docs/android-config-preservation.md`)."
        )


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--demo",
        action="store_true",
        help="Analyze Nova-Proxy-App-main/config/settings.json if present (CI/local smoke).",
    )
    parser.add_argument(
        "--path",
        type=Path,
        help="JSON file to analyze (user-exported donor or mhrv config).",
    )
    parser.add_argument(
        "--no-nested",
        action="store_true",
        help="Skip nested object/array preview under root keys.",
    )
    args = parser.parse_args()

    if args.path is None and not args.demo:
        parser.print_help()
        print(
            "\nTip: run with --demo after clone, or --path FILE for a downloaded export.",
            file=sys.stderr,
        )
        return 0

    if not REGISTRY.exists():
        print(f"missing registry: {REGISTRY}", file=sys.stderr)
        return 1
    registry = load_registry_roots()

    if args.path is not None:
        path = args.path if args.path.is_absolute() else (ROOT / args.path)
        try:
            label = str(path.relative_to(ROOT))
        except ValueError:
            label = str(path)
    else:
        path = ROOT / "Nova-Proxy-App-main" / "config" / "settings.json"
        label = "Nova-Proxy-App-main/config/settings.json (demo)"

    if not path.exists():
        print(f"report-nova-proxy-config: skip - file not found: {path}")
        return 0

    try:
        raw = path.read_text(encoding="utf-8", errors="replace")
        obj = json.loads(raw)
    except json.JSONDecodeError as e:
        print(f"{path}: invalid JSON ({e})", file=sys.stderr)
        return 1

    if not isinstance(obj, dict):
        print(f"{path}: expected JSON object at root", file=sys.stderr)
        return 1

    keys = frozenset(obj.keys())
    print(f"## External proxy config report: {label}")
    print(f"- Top-level keys: {len(keys)}")
    analyze_top_level(keys, registry)
    if not args.no_nested:
        print_nested_preview(obj)

    print(
        "\nRisk reminder: importing donor **rules** or MITM presets into mhrv-f defaults "
        "is **rejected** policy - use this report for manual triage only "
        "(`docs/donor-absorption-matrix.md`)."
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
