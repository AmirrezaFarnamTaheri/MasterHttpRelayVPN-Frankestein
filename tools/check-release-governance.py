#!/usr/bin/env python3
"""Guard release/changelog governance surfaces."""

from __future__ import annotations

import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent


FILES = {
    "CHANGELOG.md": ROOT / "CHANGELOG.md",
    "docs/RELEASE_NOTES.md": ROOT / "docs/RELEASE_NOTES.md",
    "docs/changelog/README.md": ROOT / "docs/changelog/README.md",
    "docs/changelog/TEMPLATE.md": ROOT / "docs/changelog/TEMPLATE.md",
    "docs/changelog/index.md": ROOT / "docs/changelog/index.md",
    "docs/release-checklist.md": ROOT / "docs/release-checklist.md",
    "docs/versioning-policy.md": ROOT / "docs/versioning-policy.md",
    "docs/rollback-policy.md": ROOT / "docs/rollback-policy.md",
    "docs/index.md": ROOT / "docs/index.md",
    "tools/README.md": ROOT / "tools/README.md",
    "tools/run-repo-sanity.py": ROOT / "tools/run-repo-sanity.py",
}


def die(message: str) -> None:
    print(f"release governance check failed: {message}", file=sys.stderr)
    raise SystemExit(1)


def read(label: str) -> str:
    path = FILES[label]
    if not path.is_file():
        die(f"missing required file: {label}")
    return path.read_text(encoding="utf-8")


def require_all(text: str, needles: list[str], *, label: str) -> None:
    missing = [needle for needle in needles if needle not in text]
    if missing:
        formatted = ", ".join(repr(needle) for needle in missing)
        die(f"{label} is missing required marker(s): {formatted}")


def main() -> int:
    changelog = read("CHANGELOG.md")
    template = read("docs/changelog/TEMPLATE.md")
    release_checklist = read("docs/release-checklist.md")
    versioning = read("docs/versioning-policy.md")
    rollback = read("docs/rollback-policy.md")
    index_doc = read("docs/index.md")
    tools_readme = read("tools/README.md")
    sanity = read("tools/run-repo-sanity.py")

    require_all(
        changelog,
        [
            "Canonical public release",
            "docs/changelog/v<version>.md",
            "docs/RELEASE_NOTES.md",
            "docs/changelog/index.md",
            "docs/changelog/TEMPLATE.md",
            "python tools\\generate-changelog-index.py",
            "python tools\\check-release-governance.py",
        ],
        label="CHANGELOG.md",
    )
    require_all(
        template,
        [
            "## UI / UX",
            "## Config / Schema",
            "## Backend Helpers",
            "## Security / Trust",
            "## Breaking / Cleanup",
            "## Parity",
            "## Race / Split-Brain Review",
            "## Verification",
        ],
        label="docs/changelog/TEMPLATE.md",
    )
    require_all(
        release_checklist,
        [
            "CHANGELOG.md",
            "docs/changelog/TEMPLATE.md",
            "docs/versioning-policy.md",
            "docs/rollback-policy.md",
            "python tools/check-release-governance.py",
        ],
        label="docs/release-checklist.md",
    )
    require_all(
        versioning,
        [
            "Patch-style release",
            "Minor-style release",
            "Major-style release",
            "Config And Helper Compatibility",
        ],
        label="docs/versioning-policy.md",
    )
    require_all(
        rollback,
        [
            "Bad Desktop / CLI Release",
            "Bad Android Release",
            "Bad Config Migration",
            "Bad Backend Helper",
            "Bad tunnel-node Release",
            "GitHub Release is canonical",
        ],
        label="docs/rollback-policy.md",
    )
    require_all(
        index_doc,
        [
            "CHANGELOG.md",
            "versioning-policy.md",
            "rollback-policy.md",
        ],
        label="docs/index.md",
    )
    require_all(
        tools_readme,
        [
            "check-release-governance.py",
            "generate-changelog-index.py",
        ],
        label="tools/README.md",
    )
    require_all(
        sanity,
        [
            "tools/check-release-governance.py",
        ],
        label="tools/run-repo-sanity.py",
    )

    print("release governance check: ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
