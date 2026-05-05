#!/usr/bin/env python3
"""Validate docs/parity-matrix.json against the repo (drift gate).

- Mode keys must match Rust `Mode::as_str` values in src/config.rs.
- `backend_taxonomy` must list exactly the keys under `backends` (canonical names).
- Every path listed under `docs` and `examples` must exist at repo root.
- Required fields and support-level strings are checked for consistency.
"""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[1]
SRC = ROOT / "docs" / "parity-matrix.json"

SUPPORT_LEVELS = frozenset({"supported", "partial", "planned", "not_applicable", "unsupported"})


def rust_mode_as_str_values() -> list[str]:
    text = (ROOT / "src" / "config.rs").read_text(encoding="utf-8")
    idx = text.find("impl Mode")
    if idx < 0:
        raise SystemExit("src/config.rs: could not find `impl Mode`")
    block = text[idx : idx + 4000]
    matches = re.findall(r"Mode::\w+\s*=>\s*\"([^\"]+)\"", block)
    if len(matches) < 2:
        raise SystemExit("src/config.rs: expected Mode::as_str string mappings near `impl Mode`")
    return matches


def check_object(
    label: str,
    key: str,
    obj: dict[str, Any],
    *,
    require_examples_key: bool,
) -> list[str]:
    errs: list[str] = []
    for field in ("desktop", "android", "runtime"):
        if field not in obj:
            errs.append(f"{label}[{key!r}]: missing {field!r}")
            continue
        v = obj[field]
        if v not in SUPPORT_LEVELS:
            errs.append(f"{label}[{key!r}]: {field!r} must be one of {sorted(SUPPORT_LEVELS)}, got {v!r}")
    for list_key in ("docs", "ci"):
        if list_key not in obj:
            errs.append(f"{label}[{key!r}]: missing {list_key!r} (use [] if none)")
            continue
        if not isinstance(obj[list_key], list):
            errs.append(f"{label}[{key!r}]: {list_key!r} must be a JSON array")
        else:
            for i, item in enumerate(obj[list_key]):
                if not isinstance(item, str) or not item.strip():
                    errs.append(f"{label}[{key!r}]: {list_key!r}[{i}] must be a non-empty string")
    if require_examples_key:
        if "examples" not in obj:
            errs.append(f"{label}[{key!r}]: missing 'examples' (use [] if none)")
        elif not isinstance(obj["examples"], list):
            errs.append(f"{label}[{key!r}]: 'examples' must be a JSON array")
        else:
            for i, item in enumerate(obj["examples"]):
                if not isinstance(item, str) or not item.strip():
                    errs.append(f"{label}[{key!r}]: examples[{i}] must be a non-empty string")
    if "notes" not in obj or not isinstance(obj["notes"], str):
        errs.append(f"{label}[{key!r}]: missing or invalid 'notes' (use \"\" if none)")
    return errs


def verify_paths(rel_paths: list[str], *, context: str) -> list[str]:
    errs: list[str] = []
    for p in rel_paths:
        full = (ROOT / p).resolve()
        try:
            full.relative_to(ROOT.resolve())
        except ValueError:
            errs.append(f"{context}: path escapes repo root: {p!r}")
            continue
        if not full.is_file():
            errs.append(f"{context}: missing file {p!r}")
    return errs


def main() -> int:
    data = json.loads(SRC.read_text(encoding="utf-8"))
    if not isinstance(data, dict):
        print("docs/parity-matrix.json must be a JSON object", file=sys.stderr)
        return 1

    modes = data.get("modes")
    backends = data.get("backends")
    if not isinstance(modes, dict) or not isinstance(backends, dict):
        print("docs/parity-matrix.json must contain 'modes' and 'backends' objects", file=sys.stderr)
        return 1

    taxonomy = data.get("backend_taxonomy")
    errors: list[str] = []
    if not isinstance(taxonomy, list) or not taxonomy:
        errors.append("docs/parity-matrix.json: missing non-empty array backend_taxonomy")
    elif not all(isinstance(x, str) and x.strip() for x in taxonomy):
        errors.append("backend_taxonomy must be a list of non-empty strings")
    elif len(taxonomy) != len(set(taxonomy)):
        errors.append(f"backend_taxonomy has duplicates: {taxonomy!r}")
    else:
        back_keys = list(backends.keys())
        if set(taxonomy) != set(back_keys):
            only_tax = sorted(set(taxonomy) - set(back_keys))
            only_back = sorted(set(back_keys) - set(taxonomy))
            if only_tax:
                errors.append(f"backend_taxonomy entries not present under backends: {only_tax}")
            if only_back:
                errors.append(f"backends keys missing from backend_taxonomy: {only_back}")
        elif back_keys != taxonomy:
            errors.append(
                "backends object key order must match backend_taxonomy exactly "
                f"(got {back_keys!r}, expected {taxonomy!r})"
            )

    rust_modes = rust_mode_as_str_values()
    json_mode_keys = sorted(modes.keys())
    rust_mode_set = set(rust_modes)
    json_mode_set = set(json_mode_keys)
    if rust_mode_set != json_mode_set:
        only_rust = sorted(rust_mode_set - json_mode_set)
        only_json = sorted(json_mode_set - rust_mode_set)
        if only_rust:
            errors.append(f"modes: keys in Rust Mode::as_str but not in parity-matrix.json: {only_rust}")
        if only_json:
            errors.append(f"modes: keys in parity-matrix.json but not in Rust Mode::as_str: {only_json}")

    for key, meta in modes.items():
        if not isinstance(meta, dict):
            errors.append(f"modes[{key!r}]: must be an object")
            continue
        errors.extend(check_object("modes", key, meta, require_examples_key=True))

    for key, meta in backends.items():
        if not isinstance(meta, dict):
            errors.append(f"backends[{key!r}]: must be an object")
            continue
        errors.extend(check_object("backends", key, meta, require_examples_key=False))
        ex = meta.get("examples")
        if ex is not None and not isinstance(ex, list):
            errors.append(f"backends[{key!r}]: 'examples' must be a JSON array or omitted")

    # File existence: docs + examples (modes and backends).
    for section_name, items, ex_key in (
        ("modes", modes, "examples"),
        ("backends", backends, "examples"),
    ):
        for key, meta in items.items():
            if not isinstance(meta, dict):
                continue
            docs = meta.get("docs")
            if isinstance(docs, list):
                errors.extend(
                    verify_paths([d for d in docs if isinstance(d, str)], context=f"{section_name}[{key}].docs")
                )
            ex = meta.get(ex_key)
            if isinstance(ex, list):
                errors.extend(
                    verify_paths([e for e in ex if isinstance(e, str)], context=f"{section_name}[{key}].examples")
                )

    if errors:
        print("\n".join(errors), file=sys.stderr)
        return 1

    print(
        "ok parity-matrix keys modes="
        + str(len(json_mode_keys))
        + " backends="
        + str(len(backends))
        + " rust_modes="
        + str(rust_modes)
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
