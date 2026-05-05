#!/usr/bin/env python3
"""Generate config registry docs from a canonical JSON source.

Batch 1 goal: keep Desktop/Android/docs/examples from drifting by maintaining
one source of truth for config field metadata.

Source of truth: docs/config-registry.json (root keys only must match serialized `Config`;
optional `nested_fields` documents inner JSON keys for composite roots; optional
`value_semantics` documents map-valued roots such as `hosts`).
Generated outputs:
  - docs/config-registry.md
  - docs/config-parity-matrix.md
"""

from __future__ import annotations

import argparse
import hashlib
import json
from dataclasses import dataclass
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[1]

# Human-readable Rust struct names for nested schema headings (optional).
# Keep in sync with `tools/check-config-registry-nested-fields.py` (`NESTED_RUST_TYPES`).
NESTED_RUST_TYPES: dict[str, str] = {
    "account_groups": "AccountGroup",
    "domain_overrides": "DomainOverride",
    "fronting_groups": "FrontingGroup",
    "vercel": "VercelConfig",
}
REGISTRY_JSON = ROOT / "docs" / "config-registry.json"
OUT_REGISTRY_MD = ROOT / "docs" / "config-registry.md"
OUT_PARITY_MD = ROOT / "docs" / "config-parity-matrix.md"


def sha256_text(text: str) -> str:
    return hashlib.sha256(text.encode("utf-8")).hexdigest()


def load_json(path: Path) -> Any:
    return json.loads(path.read_text(encoding="utf-8"))


def dump_md(path: Path, text: str) -> None:
    if not text.endswith("\n"):
        text += "\n"
    path.write_text(text, encoding="utf-8")


def norm_list(value: Any) -> list[str]:
    if value is None:
        return []
    if isinstance(value, list):
        return [str(v) for v in value]
    return [str(value)]


@dataclass(frozen=True)
class NestedEntry:
    key: str
    type: str
    docs: str
    validation: str


@dataclass(frozen=True)
class Field:
    name: str
    type: str
    default: str
    modes: list[str]
    desktop: str
    android: str
    backend: str
    docs: str
    validation: str
    examples: list[str]
    nested: tuple[NestedEntry, ...]


def parse_nested(field_name: str, meta: dict[str, Any]) -> tuple[NestedEntry, ...]:
    raw = meta.get("nested_fields")
    if raw is None:
        return ()
    if not isinstance(raw, dict):
        raise ValueError(f"field {field_name}: nested_fields must be an object")
    entries: list[NestedEntry] = []
    for key in sorted(raw.keys()):
        nm = raw[key]
        if not isinstance(nm, dict):
            raise ValueError(f"field {field_name}: nested_fields[{key!r}] must be an object")
        entries.append(
            NestedEntry(
                key=key,
                type=str(nm.get("type", "")),
                docs=str(nm.get("docs", "")),
                validation=str(nm.get("validation", "")),
            )
        )
    return tuple(entries)


def parse_fields(raw: dict[str, Any]) -> list[Field]:
    fields: list[Field] = []
    for name, meta in raw.items():
        if not isinstance(meta, dict):
            raise ValueError(f"field {name} must be an object")
        fields.append(
            Field(
                name=name,
                type=str(meta.get("type", "")),
                default=str(meta.get("default", "")),
                modes=norm_list(meta.get("modes")),
                desktop=str(meta.get("desktop", "")),
                android=str(meta.get("android", "")),
                backend=str(meta.get("backend", "")),
                docs=str(meta.get("docs", "")),
                validation=str(meta.get("validation", "")),
                examples=norm_list(meta.get("examples")),
                nested=parse_nested(name, meta),
            )
        )
    fields.sort(key=lambda f: f.name)
    return fields


def render_value_semantics_md(raw: dict[str, Any]) -> str:
    def cell(text: str) -> str:
        return (text or "-").replace("|", "\\|").replace("\n", " ")

    rows: list[tuple[str, str]] = []
    for name in sorted(raw.keys()):
        meta = raw[name]
        if not isinstance(meta, dict):
            continue
        vs = meta.get("value_semantics")
        if isinstance(vs, str) and vs.strip():
            rows.append((name, vs.strip()))

    if not rows:
        return ""

    lines = ["## Value semantics (map / special composites)", ""]
    lines.append(
        "Canonical prose for fields whose JSON shape is not a fixed Rust struct "
        "listed under **Nested object schemas** (see `value_semantics` in "
        "`docs/config-registry.json`)."
    )
    lines.append("")
    lines.append("| Field | Semantics |")
    lines.append("|---|---|")
    for name, text in rows:
        lines.append("| " + " | ".join([cell(f"`{name}`"), cell(text)]) + " |")
    lines.append("")
    return "\n".join(lines) + "\n"


def render_registry_md(fields: list[Field], raw: dict[str, Any]) -> str:
    def cell(text: str) -> str:
        # Markdown table cells cannot contain raw pipes.
        return (text or "-").replace("|", "\\|").replace("\n", " ")

    lines: list[str] = []
    lines.append("# Config registry (canonical metadata)")
    lines.append("")
    lines.append("Source of truth: `docs/config-registry.json`.")
    lines.append("Generated by `tools/generate-config-registry.py`.")
    lines.append("")
    lines.append("## Fields")
    lines.append("")
    lines.append("| Field | Type | Default | Modes | Desktop | Android | Backend | Docs |")
    lines.append("|---|---|---|---|---|---|---|---|")
    for f in fields:
        modes = ", ".join(f.modes) if f.modes else "all"
        lines.append(
            "| "
            + " | ".join(
                [
                    cell(f"`{f.name}`"),
                    cell(f.type),
                    cell(f.default),
                    cell(modes),
                    cell(f.desktop),
                    cell(f.android),
                    cell(f.backend),
                    cell(f"`{f.docs}`" if f.docs else "-"),
                ]
            )
            + " |"
        )
    lines.append("")
    nested_md = render_nested_schemas_md(fields)
    if nested_md:
        lines.append(nested_md.rstrip("\n"))
        lines.append("")
    value_md = render_value_semantics_md(raw)
    if value_md:
        lines.append(value_md.rstrip("\n"))
        lines.append("")
    lines.append("## Notes")
    lines.append("")
    lines.append("- This registry intentionally describes **capabilities** and **support**.")
    lines.append("- Validation is enforced by Rust (`src/config.rs`) and mapped into readiness rules where applicable.")
    lines.append(
        "- **`nested_fields`** in `docs/config-registry.json` documents JSON keys inside "
        "composite roots (`vercel`, `account_groups[]`, `domain_overrides[]`, "
        "`fronting_groups[]`, …); they are not separate serialized `Config` keys."
    )
    lines.append(
        "- **`value_semantics`** documents map-valued roots such as **`hosts`** "
        "(hostname-to-target entries)."
    )
    return "\n".join(lines) + "\n"


def render_nested_schemas_md(fields: list[Field]) -> str:
    def cell(text: str) -> str:
        return (text or "-").replace("|", "\\|").replace("\n", " ")

    def nested_heading(field: Field) -> str:
        rust = NESTED_RUST_TYPES.get(field.name)
        if "list<" in field.type.lower():
            suffix = f" (`{rust}`)" if rust else ""
            return f"### Keys inside each `{field.name}[]` element{suffix}"
        title = f"`{field.name}` (`{rust}`)" if rust else f"`{field.name}`"
        return f"### Keys inside root field {title}"

    blocks: list[str] = []
    for f in fields:
        if not f.nested:
            continue
        blocks.append(nested_heading(f))
        blocks.append("")
        blocks.append("| Key | Type | Docs | Validation |")
        blocks.append("|---|---|---|---|")
        for n in f.nested:
            blocks.append(
                "| "
                + " | ".join(
                    [
                        cell(f"`{n.key}`"),
                        cell(n.type),
                        cell(f"`{n.docs}`" if n.docs else "-"),
                        cell(n.validation),
                    ]
                )
                + " |"
            )
        blocks.append("")
    if not blocks:
        return ""
    lines = ["## Nested object schemas", ""]
    lines.append(
        "Canonical descriptions for JSON keys nested under composite `Config` fields "
        "(see `nested_fields` in `docs/config-registry.json`). For **list** roots, "
        "the tables describe **each array element** (`account_groups[]`, "
        "`domain_overrides[]`, `fronting_groups[]`, …)."
    )
    lines.append("")
    lines.extend(blocks)
    return "\n".join(lines) + "\n"


def render_parity_md(fields: list[Field]) -> str:
    def cell(text: str) -> str:
        return (text or "-").replace("|", "\\|").replace("\n", " ")

    # Compact parity matrix: one row per field, surfaces and notes.
    lines: list[str] = []
    lines.append("# Config parity matrix (field × surface)")
    lines.append("")
    lines.append("Generated from `docs/config-registry.json`.")
    lines.append("")
    lines.append("| Field | Desktop | Android | Backend/runtime | Validation | Examples |")
    lines.append("|---|---|---|---|---|---|")
    for f in fields:
        ex = ", ".join(f"`{p}`" for p in f.examples) if f.examples else "-"
        lines.append(
            "| "
            + " | ".join(
                [
                    cell(f"`{f.name}`"),
                    cell(f.desktop),
                    cell(f.android),
                    cell(f.backend),
                    cell(f.validation),
                    cell(ex),
                ]
            )
            + " |"
        )
    lines.append("")
    return "\n".join(lines) + "\n"


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("-Check", action="store_true", help="Verify generated files are current.")
    args = parser.parse_args()

    raw = load_json(REGISTRY_JSON)
    if not isinstance(raw, dict):
        raise SystemExit("docs/config-registry.json must be an object of field->metadata")
    fields = parse_fields(raw)

    reg_md = render_registry_md(fields, raw)
    parity_md = render_parity_md(fields)

    if args.Check:
        errors: list[str] = []
        if not OUT_REGISTRY_MD.exists() or OUT_REGISTRY_MD.read_text(encoding="utf-8") != reg_md:
            errors.append(f"stale: {OUT_REGISTRY_MD.relative_to(ROOT)}")
        if not OUT_PARITY_MD.exists() or OUT_PARITY_MD.read_text(encoding="utf-8") != parity_md:
            errors.append(f"stale: {OUT_PARITY_MD.relative_to(ROOT)}")
        if errors:
            for e in errors:
                print(e)
            return 1
        print(
            f"ok registry={OUT_REGISTRY_MD} parity={OUT_PARITY_MD} fields={len(fields)} "
            f"sha256={sha256_text(reg_md)[:12]}"
        )
        return 0

    dump_md(OUT_REGISTRY_MD, reg_md)
    dump_md(OUT_PARITY_MD, parity_md)
    print(
        f"wrote registry={OUT_REGISTRY_MD} parity={OUT_PARITY_MD} fields={len(fields)} "
        f"sha256={sha256_text(reg_md)[:12]}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

