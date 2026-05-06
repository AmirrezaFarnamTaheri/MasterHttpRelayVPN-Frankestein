#!/usr/bin/env python3
"""Guard the shared Doctor JSON renderer contract."""

from __future__ import annotations

import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
DOCTOR = ROOT / "src/doctor.rs"
SUPPORT_BUNDLE = ROOT / "src/support_bundle.rs"
DOC = ROOT / "docs/doctor-json-contract.md"
DOC_INDEX = ROOT / "docs/index.md"
TOOLS_README = ROOT / "tools/README.md"
RUN_SANITY = ROOT / "tools/run-repo-sanity.py"

REQUIRED_KEYS = ["ok", "items", "id", "level", "title", "detail", "fix"]
REQUIRED_LEVELS = ["ok", "warn", "fail"]


def die(msg: str) -> None:
    print(f"doctor JSON contract check failed: {msg}", file=sys.stderr)
    raise SystemExit(1)


def main() -> int:
    doctor = DOCTOR.read_text(encoding="utf-8")
    support = SUPPORT_BUNDLE.read_text(encoding="utf-8")
    doc = DOC.read_text(encoding="utf-8")
    doc_index = DOC_INDEX.read_text(encoding="utf-8")
    tools_readme = TOOLS_README.read_text(encoding="utf-8")
    run_sanity = RUN_SANITY.read_text(encoding="utf-8")

    for symbol in [
        "pub fn doctor_level_str",
        "pub fn doctor_item_json_value",
        "pub fn doctor_report_json_value",
    ]:
        if symbol not in doctor:
            die(f"missing shared renderer symbol {symbol!r}")

    for key in REQUIRED_KEYS:
        if f'"{key}"' not in doctor:
            die(f"shared renderer missing JSON key {key!r}")
        if f"`{key}`" not in doc:
            die(f"contract doc missing JSON key {key!r}")

    for level in REQUIRED_LEVELS:
        if f'"{level}"' not in doctor:
            die(f"shared renderer missing Doctor level {level!r}")
        if f"`{level}`" not in doc:
            die(f"contract doc missing Doctor level {level!r}")

    if "doctor::doctor_report_json_value(&report)" not in support:
        die("support bundle doctor.json must use doctor_report_json_value")

    forbidden = [
        '"id": it.id',
        '"level": match it.level',
        'serde_json::json!({ "ok": report.ok()',
    ]
    for needle in forbidden:
        if needle in support:
            die(f"support bundle appears to hand-build doctor JSON: {needle}")

    for path_text, owner in [
        ("doctor-json-contract.md", "docs/index.md"),
        ("doctor-json-contract.md", "tools/README.md"),
        ("tools/check-doctor-json-contract.py", "tools/run-repo-sanity.py"),
    ]:
        haystack = {
            "docs/index.md": doc_index,
            "tools/README.md": tools_readme,
            "tools/run-repo-sanity.py": run_sanity,
        }[owner]
        if path_text not in haystack:
            die(f"{owner} must mention {path_text}")

    print("doctor JSON contract check: ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
