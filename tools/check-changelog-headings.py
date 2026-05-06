#!/usr/bin/env python3
"""Require every changelog Markdown file to have a first-level heading."""

from __future__ import annotations

import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
CHANGELOG_DIR = ROOT / "docs" / "changelog"
SKIP = {"README.md", "index.md"}


def main() -> int:
    missing: list[str] = []
    for path in sorted(CHANGELOG_DIR.glob("*.md")):
        if path.name in SKIP:
            continue
        text = path.read_text(encoding="utf-8")
        if not any(line.strip().startswith("# ") for line in text.splitlines()):
            missing.append(str(path.relative_to(ROOT)))
    if missing:
        print("changelog heading check failed: missing H1 headings:", file=sys.stderr)
        for item in missing:
            print(f"- {item}", file=sys.stderr)
        return 1
    print("changelog heading check: ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
