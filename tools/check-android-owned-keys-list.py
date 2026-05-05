#!/usr/bin/env python3
"""Validate ConfigStore.kt `ownedKeys` entries against docs/config-registry.json.

Android removes these keys from imported JSON before storing `preservedUnknownRootJson`.
Every entry must be a real Rust Config root key from the registry, or an explicit
Android-only / legacy allowlisted key from tools/android_config_allowlists.py.

Adding a typo or obsolete name to ownedKeys breaks preservation merges silently for
desktop-imported configs — this gate fails fast.
"""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path

_TOOLS_DIR = Path(__file__).resolve().parent
if str(_TOOLS_DIR) not in sys.path:
    sys.path.insert(0, str(_TOOLS_DIR))

from android_config_allowlists import ANDROID_ONLY_KEYS, LEGACY_KEYS  # noqa: E402

ROOT = Path(__file__).resolve().parents[1]
REGISTRY = ROOT / "docs" / "config-registry.json"
ANDROID_CONFIG = (
    ROOT
    / "android"
    / "app"
    / "src"
    / "main"
    / "java"
    / "com"
    / "farnam"
    / "mhrvf"
    / "ConfigStore.kt"
)


def extract_owned_keys(kt: str) -> list[str]:
    anchor = "val ownedKeys = listOf("
    idx = kt.find(anchor)
    if idx == -1:
        raise SystemExit(f"{ANDROID_CONFIG}: missing `{anchor}`")

    paren = kt.find("(", idx)
    depth = 0
    end = -1
    for j in range(paren, len(kt)):
        c = kt[j]
        if c == "(":
            depth += 1
        elif c == ")":
            depth -= 1
            if depth == 0:
                end = j
                break
    if end == -1:
        raise SystemExit(f"{ANDROID_CONFIG}: unterminated ownedKeys listOf(...)")

    block = kt[paren + 1 : end]
    keys = re.findall(r'"([^"]+)"', block)
    if not keys:
        raise SystemExit(f"{ANDROID_CONFIG}: ownedKeys parse produced empty list")
    return keys


def main() -> int:
    registry_obj = json.loads(REGISTRY.read_text(encoding="utf-8"))
    if not isinstance(registry_obj, dict):
        raise SystemExit("docs/config-registry.json must be an object")
    registry_keys = set(registry_obj.keys())

    kt = ANDROID_CONFIG.read_text(encoding="utf-8")
    owned = extract_owned_keys(kt)
    if len(owned) != len(set(owned)):
        from collections import Counter

        dupes = [k for k, n in Counter(owned).items() if n > 1]
        print(f"{ANDROID_CONFIG}: duplicate ownedKeys entries: {dupes}", file=sys.stderr)
        return 1

    allowed_extra = ANDROID_ONLY_KEYS | LEGACY_KEYS
    bad = sorted(k for k in owned if k not in registry_keys and k not in allowed_extra)
    if bad:
        print(
            f"{ANDROID_CONFIG}: ownedKeys entries not in registry and not allowlisted: {bad}",
            file=sys.stderr,
        )
        print(
            "Fix typos or add keys to docs/config-registry.json; Android-only roots stay "
            "in tools/android_config_allowlists.py.",
            file=sys.stderr,
        )
        return 1

    reg_refs = sorted(k for k in owned if k in registry_keys)
    print(f"ok android ownedKeys n={len(owned)} registry_refs={len(reg_refs)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
