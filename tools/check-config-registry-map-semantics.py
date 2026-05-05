#!/usr/bin/env python3
"""Require registry fields whose type looks like a JSON map to define value_semantics."""

from __future__ import annotations

import json
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
REGISTRY = ROOT / "docs" / "config-registry.json"


def main() -> int:
    raw = json.loads(REGISTRY.read_text(encoding="utf-8"))
    if not isinstance(raw, dict):
        raise SystemExit(f"{REGISTRY} must be a JSON object")

    errors: list[str] = []
    for name in sorted(raw.keys()):
        meta = raw[name]
        if not isinstance(meta, dict):
            continue
        typ = str(meta.get("type", "")).lower()
        if "map<" not in typ:
            continue
        vs = meta.get("value_semantics")
        if not isinstance(vs, str) or not vs.strip():
            errors.append(
                f"{REGISTRY}: `{name}` has map-like type `{meta.get('type')}` "
                "but missing non-empty string `value_semantics`"
            )

    if errors:
        print("\n".join(errors), file=sys.stderr)
        return 1

    print("ok map-valued registry fields have value_semantics")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
