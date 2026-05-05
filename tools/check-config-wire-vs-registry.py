#!/usr/bin/env python3
"""Ensure Desktop `ConfigWire` declares every root field from docs/config-registry.json.

Rust UI tests prove behavior when fields are present; this gate catches missing struct
members when `Config` grows but `src/bin/ui.rs` `ConfigWire` forgets to mirror it.

Serialized JSON keys default to Rust field names — `serde(rename)` on `ConfigWire`
fields is not supported here (extend parsing if introduced).
"""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
REGISTRY = ROOT / "docs" / "config-registry.json"
UI_RS = ROOT / "src" / "bin" / "ui.rs"


def extract_config_wire_fields(src: str) -> list[str]:
    anchor = "struct ConfigWire<'a> {"
    idx = src.find(anchor)
    if idx == -1:
        raise SystemExit(f"{UI_RS}: missing `{anchor}`")

    brace_open = src.find("{", idx)
    depth = 0
    end = -1
    for i in range(brace_open, len(src)):
        c = src[i]
        if c == "{":
            depth += 1
        elif c == "}":
            depth -= 1
            if depth == 0:
                end = i
                break
    if end == -1:
        raise SystemExit(f"{UI_RS}: unterminated ConfigWire struct")

    body = src[brace_open + 1 : end]
    fields: list[str] = []
    for raw_line in body.splitlines():
        line = raw_line.strip()
        if not line or line.startswith("//"):
            continue
        if line.startswith("#["):
            continue
        m = re.match(r"^(\w+)\s*:", line)
        if m:
            fields.append(m.group(1))
    if not fields:
        raise SystemExit(f"{UI_RS}: ConfigWire field parse produced empty list")
    return fields


def main() -> int:
    registry = json.loads(REGISTRY.read_text(encoding="utf-8"))
    if not isinstance(registry, dict):
        raise SystemExit(f"{REGISTRY} must be a JSON object")
    reg_keys = set(registry.keys())

    ui_src = UI_RS.read_text(encoding="utf-8")
    wire_fields = extract_config_wire_fields(ui_src)
    wire_set = set(wire_fields)
    if len(wire_fields) != len(wire_set):
        from collections import Counter

        dupes = [k for k, n in Counter(wire_fields).items() if n > 1]
        print(f"{UI_RS}: duplicate ConfigWire fields: {dupes}", file=sys.stderr)
        return 1

    missing = sorted(reg_keys - wire_set)
    extra = sorted(wire_set - reg_keys)
    if missing or extra:
        parts: list[str] = []
        if missing:
            parts.append(f"registry keys missing from ConfigWire: {missing}")
        if extra:
            parts.append(f"ConfigWire fields not in registry: {extra}")
        print("\n".join(parts), file=sys.stderr)
        return 1

    print(f"ok ConfigWire fields match registry roots n={len(reg_keys)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
