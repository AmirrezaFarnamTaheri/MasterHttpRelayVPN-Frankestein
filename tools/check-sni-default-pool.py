#!/usr/bin/env python3
"""Fail if Rust DEFAULT_GOOGLE_SNI_POOL and Android DEFAULT_SNI_POOL diverge."""

from __future__ import annotations

import re
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
RUST = ROOT / "src" / "domain_fronter.rs"
KOTLIN = ROOT / "android" / "app" / "src" / "main" / "java" / "com" / "farnam" / "mhrvf" / "ConfigStore.kt"


def _strip_full_line_cpp_comments(snippet: str) -> str:
    out: list[str] = []
    for line in snippet.splitlines():
        stripped = line.lstrip()
        if stripped.startswith("//"):
            continue
        if "//" in line:
            line = line.split("//", 1)[0].rstrip()
        out.append(line)
    return "\n".join(out)


def rust_default_google_sni_pool() -> list[str]:
    text = RUST.read_text(encoding="utf-8")
    m = re.search(
        r"pub const DEFAULT_GOOGLE_SNI_POOL:\s*&\[\s*&str\s*\]\s*=\s*&\[\s*((?:.|\n)*?)\n\];",
        text,
    )
    if not m:
        raise SystemExit(f"{RUST}: could not find DEFAULT_GOOGLE_SNI_POOL slice")
    block = _strip_full_line_cpp_comments(m.group(1))
    vals = re.findall(r'"([^"]+)"', block)
    return vals


def kotlin_default_sni_pool() -> list[str]:
    text = KOTLIN.read_text(encoding="utf-8")
    m = re.search(
        r"val\s+DEFAULT_SNI_POOL:\s*List<String>\s*=\s*listOf\(((?:.|\n)*?)\n\)",
        text,
    )
    if not m:
        raise SystemExit(f"{KOTLIN}: could not find DEFAULT_SNI_POOL listOf(...) body")
    block = _strip_full_line_cpp_comments(m.group(1))
    vals = re.findall(r'"([^"]+)"', block)
    return vals


def main() -> int:
    r = rust_default_google_sni_pool()
    k = kotlin_default_sni_pool()

    errs: list[str] = []
    if len(r) != len(k):
        errs.append(f"pool lengths differ: rust={len(r)} kotlin={len(k)}")

    rust_set = set(r)
    kotlin_set = set(k)
    if len(rust_set) != len(r):
        errs.append(f"Rust pool has duplicates: {[x for x in r if r.count(x) > 1]}")
    if len(kotlin_set) != len(k):
        errs.append(f"Kotlin pool has duplicates: {[x for x in k if k.count(x) > 1]}")

    missing_in_kotlin = [x for x in r if x not in kotlin_set]
    extra_in_kotlin = [x for x in k if x not in rust_set]
    if missing_in_kotlin or extra_in_kotlin:
        if missing_in_kotlin:
            errs.append("In Rust DEFAULT_GOOGLE_SNI_POOL but not Android DEFAULT_SNI_POOL: " + repr(missing_in_kotlin))
        if extra_in_kotlin:
            errs.append("In Android DEFAULT_SNI_POOL but not Rust: " + repr(extra_in_kotlin))

    order_mismatch_positions = [(i, a, b) for i, (a, b) in enumerate(zip(r, k)) if a != b]
    if rust_set == kotlin_set and order_mismatch_positions:
        errs.append(
            "Same host set but different order (prefer matching Rust):\n"
            + "\n".join(f"  index {i}: rust={ra!r} kotlin={kb!r}" for i, ra, kb in order_mismatch_positions)
        )

    if errs:
        print("\n".join(errs), file=sys.stderr)
        return 1

    print(f"ok DEFAULT_SNI_POOL parity n={len(r)} pools match")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
