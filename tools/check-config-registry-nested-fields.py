#!/usr/bin/env python3
"""Ensure docs/config-registry.json nested_fields keys match Rust struct fields.

Keep `NESTED_RUST_TYPES` in sync with `tools/generate-config-registry.py`.

Uses serde JSON field names (Rust `pub` identifiers). Does not handle `serde(rename)`
on individual fields — extend parsing if those appear on documented structs.
"""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
REGISTRY = ROOT / "docs" / "config-registry.json"
RUST_CONFIG = ROOT / "src" / "config.rs"

# Registry root field name -> Rust struct name (same mapping as generate-config-registry.py).
NESTED_RUST_TYPES: dict[str, str] = {
    "account_groups": "AccountGroup",
    "domain_overrides": "DomainOverride",
    "fronting_groups": "FrontingGroup",
    "vercel": "VercelConfig",
}


def rust_struct_fields(src: str, struct_name: str) -> list[str]:
    anchor = f"pub struct {struct_name} {{"
    idx = src.find(anchor)
    if idx == -1:
        raise SystemExit(f"{RUST_CONFIG}: missing `{anchor}`")

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
        raise SystemExit(f"{RUST_CONFIG}: unterminated struct `{struct_name}`")

    body = src[brace_open + 1 : end]
    fields = re.findall(r"^\s*pub\s+(\w+)\s*:", body, flags=re.MULTILINE)
    if not fields:
        raise SystemExit(f"{RUST_CONFIG}: struct `{struct_name}` has no pub fields parsed")
    return fields


def main() -> int:
    registry = json.loads(REGISTRY.read_text(encoding="utf-8"))
    if not isinstance(registry, dict):
        raise SystemExit(f"{REGISTRY} must be a JSON object")

    rust_src = RUST_CONFIG.read_text(encoding="utf-8")

    nested_roots = sorted(k for k, v in registry.items() if isinstance(v, dict) and v.get("nested_fields"))

    errors: list[str] = []
    for root in nested_roots:
        meta = registry[root]
        nested_raw = meta.get("nested_fields")
        assert isinstance(nested_raw, dict)
        json_keys = sorted(nested_raw.keys())

        rust_struct = NESTED_RUST_TYPES.get(root)
        if rust_struct is None:
            errors.append(
                f"{REGISTRY}: root field `{root}` has nested_fields but no Rust struct mapping "
                f"(add `{root}` to NESTED_RUST_TYPES in check-config-registry-nested-fields.py "
                f"and generate-config-registry.py)"
            )
            continue

        rust_keys = sorted(set(rust_struct_fields(rust_src, rust_struct)))
        if rust_keys != json_keys:
            missing = sorted(set(rust_keys) - set(json_keys))
            extra = sorted(set(json_keys) - set(rust_keys))
            detail = []
            if missing:
                detail.append(f"missing_in_registry={missing}")
            if extra:
                detail.append(f"extra_in_registry={extra}")
            errors.append(
                f"{REGISTRY}: `{root}` nested_fields mismatch `{rust_struct}`: "
                + "; ".join(detail)
            )

    # Mapped structs without nested_fields (maintainer hygiene — registry lag).
    for root, rust_struct in sorted(NESTED_RUST_TYPES.items()):
        meta = registry.get(root)
        if not isinstance(meta, dict):
            errors.append(f"{REGISTRY}: missing root field `{root}` expected for nested drift gate")
            continue
        if not meta.get("nested_fields"):
            errors.append(
                f"{REGISTRY}: root `{root}` documents `{rust_struct}` in NESTED_RUST_TYPES "
                f"but has no nested_fields block"
            )

    if errors:
        print("\n".join(errors), file=sys.stderr)
        return 1

    print(f"ok nested_fields parity roots={len(NESTED_RUST_TYPES)} structs={sorted(NESTED_RUST_TYPES.values())}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
