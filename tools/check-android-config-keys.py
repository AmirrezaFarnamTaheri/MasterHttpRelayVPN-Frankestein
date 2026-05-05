#!/usr/bin/env python3
"""Static drift check: Android config JSON keys must match canonical registry.

Goal (Batch 1): prevent split-brain between:
  - Rust `Config` schema (`src/config.rs`)
  - Android config read/write (`android/.../ConfigStore.kt`)
  - Canonical registry (`docs/config-registry.json`)

This script extracts string-literal JSON keys used by Android when reading or
writing config and ensures:
  - Every key that maps to a Rust config field exists in the registry.
  - Android-only wrapper keys are explicitly allowlisted.
  - Legacy import-only keys are explicitly allowlisted.
"""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path

_TOOLS_DIR = Path(__file__).resolve().parent
if str(_TOOLS_DIR) not in sys.path:
    sys.path.insert(0, str(_TOOLS_DIR))

from android_config_allowlists import (  # noqa: E402
    ANDROID_ONLY_KEYS,
    LEGACY_KEYS,
    NESTED_KEYS,
)

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

KEY_RE = re.compile(
    r"""
    (?:
        put|optString|optInt|optBoolean|optJSONArray|optJSONObject|has|isNull|remove
    )\s*\(\s*
    "([^"]+)"
    """,
    re.VERBOSE,
)


def main() -> int:
    if not REGISTRY.exists():
        print(f"missing registry: {REGISTRY}", file=sys.stderr)
        return 1
    if not ANDROID_CONFIG.exists():
        print(f"missing android config source: {ANDROID_CONFIG}", file=sys.stderr)
        return 1

    registry_obj = json.loads(REGISTRY.read_text(encoding="utf-8"))
    if not isinstance(registry_obj, dict):
        print("docs/config-registry.json must be an object", file=sys.stderr)
        return 1
    registry_keys = set(registry_obj.keys())

    src = ANDROID_CONFIG.read_text(encoding="utf-8", errors="ignore")
    keys = set(KEY_RE.findall(src))

    allowed_extra = ANDROID_ONLY_KEYS | LEGACY_KEYS | NESTED_KEYS

    missing_from_registry = sorted(
        k for k in keys if (k not in registry_keys and k not in allowed_extra)
    )
    if missing_from_registry:
        print("Android ConfigStore.kt uses unknown config keys:", file=sys.stderr)
        for k in missing_from_registry:
            print(f"- {k}", file=sys.stderr)
        print(
            "Add the key to docs/config-registry.json (if it is a Rust Config field), "
            "or allowlist it in tools/android_config_allowlists.py (if Android-only).",
            file=sys.stderr,
        )
        return 1

    # Helpful summary for CI logs.
    rust_keys_used = sorted(k for k in keys if k in registry_keys)
    extra_keys_used = sorted(k for k in keys if k in allowed_extra)
    print(
        f"android config keys ok rust_keys_used={len(rust_keys_used)} "
        f"extra_keys_used={extra_keys_used}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

