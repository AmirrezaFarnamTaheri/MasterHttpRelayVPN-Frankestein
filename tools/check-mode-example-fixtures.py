#!/usr/bin/env python3
"""Static guard for mode example/fixture coverage.

Each product mode needs at least one bundled config example. Those examples
must be validated by Rust tests and referenced by the parity matrix so Desktop,
Android, docs, and CI do not drift into separate fixture sets.
"""

from __future__ import annotations

import json
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
CONFIG_RS = ROOT / "src" / "config.rs"
PARITY_JSON = ROOT / "docs" / "parity-matrix.json"
ANDROID_CONFIG = ROOT / "android/app/src/main/java/com/farnam/mhrvf/ConfigStore.kt"

EXPECTED_EXAMPLES = {
    "config.example.json": ("apps_script", "Mode::AppsScript"),
    "config.direct.example.json": ("direct", "Mode::Direct"),
    "config.fronting-groups.example.json": ("direct", "Mode::Direct"),
    "config.full.example.json": ("full", "Mode::Full"),
    "config.google-only.example.json": ("direct", "Mode::Direct"),
    "config.vercel-edge.example.json": ("vercel_edge", "Mode::VercelEdge"),
}


def die(msg: str) -> None:
    print(f"mode example fixture check failed: {msg}", file=sys.stderr)
    raise SystemExit(1)


def read(path: Path) -> str:
    if not path.is_file():
        die(f"missing file: {path.relative_to(ROOT)}")
    return path.read_text(encoding="utf-8")


def main() -> int:
    config_rs = read(CONFIG_RS)
    parity = json.loads(read(PARITY_JSON))
    android_config = read(ANDROID_CONFIG)

    seen_modes: set[str] = set()
    for filename, (mode, rust_mode) in EXPECTED_EXAMPLES.items():
        path = ROOT / filename
        if not path.is_file():
            die(f"missing bundled example: {filename}")
        data = json.loads(read(path))
        if data.get("mode") != mode:
            die(f"{filename}: expected mode {mode!r}, got {data.get('mode')!r}")
        seen_modes.add(mode)
        if f'"{filename}"' not in config_rs:
            die(f"src/config.rs test does not name {filename}")
        if f'include_str!("../{filename}")' not in config_rs:
            die(f"src/config.rs test does not include_str {filename}")
        if rust_mode not in config_rs:
            die(f"src/config.rs test does not assert {rust_mode} for {filename}")

    for required_mode in ("apps_script", "vercel_edge", "direct", "full"):
        if required_mode not in seen_modes:
            die(f"no bundled example covers mode {required_mode}")

    if "fn bundled_example_configs_load_and_validate()" not in config_rs:
        die("missing Rust bundled example validation test")
    if "assert_example_config_loads(name, json, mode);" not in config_rs:
        die("Rust bundled example validation test must call assert_example_config_loads")

    modes = parity.get("modes")
    if not isinstance(modes, dict):
        die("docs/parity-matrix.json missing modes object")
    for mode, meta in modes.items():
        examples = meta.get("examples")
        if not isinstance(examples, list):
            die(f"parity matrix mode {mode}: examples must be a list")
        if not examples:
            die(f"parity matrix mode {mode}: must list at least one example")
        for example in examples:
            if example not in EXPECTED_EXAMPLES:
                die(f"parity matrix mode {mode}: unknown example {example}")
            example_mode = EXPECTED_EXAMPLES[example][0]
            if example_mode != mode:
                die(
                    f"parity matrix mode {mode}: example {example} has mode "
                    f"{example_mode}, not {mode}"
                )

    for marker in (
        '"apps_script"',
        '"vercel_edge" -> Mode.SERVERLESS_JSON',
        '"direct", "google_only" -> Mode.DIRECT',
        '"full" -> Mode.FULL',
        "preservedUnknownRootJson",
        "val preserved = JSONObject(obj.toString())",
        "val ownedKeys = listOf(",
        "ownedKeys.forEach { preserved.remove(it) }",
        "if (preserved.length() > 0) preserved.toString() else \"\"",
        "JSONObject(preservedUnknownRootJson)",
        "preservedAccountGroupsJson",
    ):
        if marker not in android_config:
            die(f"Android config importer/exporter missing marker: {marker}")

    print("mode example fixture check: ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
