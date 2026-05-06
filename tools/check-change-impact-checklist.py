#!/usr/bin/env python3
"""Guard change-impact checklist docs and profile mappings."""

from __future__ import annotations

import json
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
CONTRACT = ROOT / "docs" / "change-impact-checklist.json"
DOC = ROOT / "docs" / "change-impact-checklist.md"
PROFILES = ROOT / "docs" / "verification-profiles.json"
DOCS_INDEX = ROOT / "docs" / "index.md"
CONTRIBUTING = ROOT / "CONTRIBUTING.md"
PR_TEMPLATE = ROOT / ".github" / "pull_request_template.md"
TOOLS_README = ROOT / "tools" / "README.md"
SANITY = ROOT / "tools" / "run-repo-sanity.py"
PARITY = ROOT / "tools" / "check-ci-local-sanity-parity.py"

REQUIRED_SURFACES = [
    "docs",
    "config",
    "desktop",
    "android",
    "backend_helpers",
    "full_tunnel",
    "release_security",
]


def die(message: str) -> None:
    print(f"change-impact checklist check failed: {message}", file=sys.stderr)
    raise SystemExit(1)


def read(path: Path) -> str:
    if not path.is_file():
        die(f"missing required file: {path.relative_to(ROOT)}")
    return path.read_text(encoding="utf-8")


def require(text: str, needle: str, label: str) -> None:
    if needle not in text:
        die(f"{label} missing {needle!r}")


def load_json(path: Path) -> dict:
    if not path.is_file():
        die(f"missing required file: {path.relative_to(ROOT)}")
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        die(f"invalid {path.relative_to(ROOT)}: {exc}")


def main() -> int:
    data = load_json(CONTRACT)
    if data.get("schema") != "mhrv-f-change-impact-checklist/v1":
        die("unexpected change-impact checklist schema")

    profile_data = load_json(PROFILES)
    profile_ids = {profile["id"] for profile in profile_data.get("profiles", [])}

    surfaces = data.get("surfaces")
    if not isinstance(surfaces, list):
        die("surfaces must be a list")

    by_id = {}
    for surface in surfaces:
        if not isinstance(surface, dict):
            die("each surface must be an object")
        surface_id = surface.get("id")
        if surface_id in by_id:
            die(f"duplicate surface id: {surface_id}")
        by_id[surface_id] = surface
        for key in ["id", "title", "paths", "profiles", "parity", "cleanup"]:
            if key not in surface:
                die(f"surface {surface_id!r} missing {key!r}")
        for list_key in ["paths", "profiles", "parity", "cleanup"]:
            if not isinstance(surface[list_key], list) or not surface[list_key]:
                die(f"surface {surface_id!r} must have non-empty {list_key}")
        for profile in surface["profiles"]:
            if profile not in profile_ids:
                die(f"surface {surface_id!r} references unknown profile {profile!r}")

    if list(by_id) != REQUIRED_SURFACES:
        die(f"surface ids/order mismatch: got {list(by_id)!r}")

    doc = read(DOC)
    require(doc, "docs/change-impact-checklist.json", "docs/change-impact-checklist.md")
    require(doc, "tools/check-change-impact-checklist.py", "docs/change-impact-checklist.md")
    for surface_id in REQUIRED_SURFACES:
        surface = by_id[surface_id]
        require(doc, surface["title"], "docs/change-impact-checklist.md")
        for profile in surface["profiles"]:
            require(doc, f"`{profile}`", "docs/change-impact-checklist.md")

    docs_index = read(DOCS_INDEX)
    require(docs_index, "change-impact-checklist.md", "docs/index.md")

    contributing = read(CONTRIBUTING)
    require(contributing, "docs/change-impact-checklist.md", "CONTRIBUTING.md")
    require(contributing, "python tools\\check-change-impact-checklist.py", "CONTRIBUTING.md")

    pr_template = read(PR_TEMPLATE)
    require(pr_template, "docs/change-impact-checklist.md", ".github/pull_request_template.md")
    require(pr_template, "Verification profile(s) selected", ".github/pull_request_template.md")

    tools_readme = read(TOOLS_README)
    require(tools_readme, "Change-impact checklist guard", "tools/README.md")
    require(tools_readme, "check-change-impact-checklist.py", "tools/README.md")

    sanity = read(SANITY)
    require(sanity, "tools/check-change-impact-checklist.py", "tools/run-repo-sanity.py")
    require(sanity, "change-impact checklist", "tools/run-repo-sanity.py")

    parity = read(PARITY)
    require(parity, "tools/check-change-impact-checklist.py", "tools/check-ci-local-sanity-parity.py")

    print("change-impact checklist check: ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
