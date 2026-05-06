#!/usr/bin/env python3
"""Guard architecture decision records and their governance links."""

from __future__ import annotations

import re
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
ADR_DIR = ROOT / "docs" / "adr"

ADR_FILES = [
    "0001-committed-android-signing-material.md",
    "0002-release-artifacts-and-authority.md",
    "0003-android-config-preservation-and-simple-editor.md",
    "0004-legacy-config-migration-boundary.md",
    "0005-platform-defaults-are-documented-and-test-governed.md",
    "0006-canonical-status-and-doctor-contracts.md",
    "0007-lightweight-governance-before-codeowners.md",
    "0008-no-stale-leftovers-cleanup-policy.md",
]

FILES = {
    "docs/adr/README.md": ADR_DIR / "README.md",
    "docs/adr/TEMPLATE.md": ADR_DIR / "TEMPLATE.md",
    "docs/index.md": ROOT / "docs" / "index.md",
    "CONTRIBUTING.md": ROOT / "CONTRIBUTING.md",
    "tools/README.md": ROOT / "tools" / "README.md",
    "tools/run-repo-sanity.py": ROOT / "tools" / "run-repo-sanity.py",
    "tools/check-ci-local-sanity-parity.py": ROOT / "tools" / "check-ci-local-sanity-parity.py",
}

REQUIRED_SECTIONS = [
    "## Status",
    "## Context",
    "## Decision",
    "## Consequences",
]


def die(message: str) -> None:
    print(f"ADR governance check failed: {message}", file=sys.stderr)
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


def check_adr_file(name: str, readme: str) -> None:
    path = ADR_DIR / name
    if not path.is_file():
        die(f"missing ADR file: docs/adr/{name}")
    text = path.read_text(encoding="utf-8")
    if not re.search(r"^# ADR-\d{4}: .+", text, flags=re.M):
        die(f"docs/adr/{name} must start with '# ADR-0000: Title'")
    require_all(text, REQUIRED_SECTIONS, label=f"docs/adr/{name}")
    require_all(readme, [name], label="docs/adr/README.md")


def main() -> int:
    readme = read("docs/adr/README.md")
    template = read("docs/adr/TEMPLATE.md")
    docs_index = read("docs/index.md")
    contributing = read("CONTRIBUTING.md")
    tools_readme = read("tools/README.md")
    sanity = read("tools/run-repo-sanity.py")
    parity = read("tools/check-ci-local-sanity-parity.py")

    require_all(
        readme,
        [
            "Architecture Decision Records",
            "When To Add An ADR",
            "tools/check-adr-governance.py",
            "TEMPLATE.md",
        ],
        label="docs/adr/README.md",
    )
    require_all(template, REQUIRED_SECTIONS, label="docs/adr/TEMPLATE.md")
    for name in ADR_FILES:
        check_adr_file(name, readme)

    require_all(
        docs_index,
        [
            "adr/README.md",
            "Architecture decision records",
        ],
        label="docs/index.md",
    )
    require_all(
        contributing,
        [
            "docs/adr/README.md",
            "Architecture Decisions",
            "python tools\\check-adr-governance.py",
        ],
        label="CONTRIBUTING.md",
    )
    require_all(
        tools_readme,
        [
            "ADR governance guard",
            "check-adr-governance.py",
        ],
        label="tools/README.md",
    )
    require_all(
        sanity,
        [
            "tools/check-adr-governance.py",
            "ADR governance",
        ],
        label="tools/run-repo-sanity.py",
    )
    require_all(
        parity,
        [
            "tools/check-adr-governance.py",
        ],
        label="tools/check-ci-local-sanity-parity.py",
    )

    print("ADR governance check: ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
