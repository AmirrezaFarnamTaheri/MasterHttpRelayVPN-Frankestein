#!/usr/bin/env python3
"""Guard the docs/tooling source map."""

from __future__ import annotations

import json
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
CONTRACT = ROOT / "docs" / "tooling-source-map.json"
DOC = ROOT / "docs" / "tooling-source-map.md"
DOCS_INDEX = ROOT / "docs" / "index.md"
CONTRIBUTING = ROOT / "CONTRIBUTING.md"
TOOLS_README = ROOT / "tools" / "README.md"
SANITY = ROOT / "tools" / "run-repo-sanity.py"
PARITY = ROOT / "tools" / "check-ci-local-sanity-parity.py"

REQUIRED_DOCS = [
    "docs/config-registry.md",
    "docs/config-parity-matrix.md",
    "docs/parity-matrix.md",
    "docs/platform-defaults.md",
    "docs/readiness-matrix.md",
    "docs/android-hardcoded-copy-inventory.md",
    "docs/changelog/index.md",
    "docs/status-stats-json-contract.md",
    "docs/doctor-json-contract.md",
    "docs/android-support-snapshot.md",
    "docs/verification-profiles.md",
    "docs/change-impact-checklist.md",
    "docs/adr/README.md",
    "docs/release-checklist.md",
    "docs/ownership.md",
    "src/bin/ui.rs",
    "src/bin/ui.rs",
    "src/bin/ui.rs",
    "src/bin/ui.rs",
    "src/bin/ui.rs",
    "src/bin/ui.rs",
    "src/bin/ui.rs",
    "src/bin/ui.rs",
    "src/bin/ui.rs",
]


def die(message: str) -> None:
    print(f"tooling source map check failed: {message}", file=sys.stderr)
    raise SystemExit(1)


def read(path: Path) -> str:
    if not path.is_file():
        die(f"missing required file: {path.relative_to(ROOT)}")
    return path.read_text(encoding="utf-8")


def require(text: str, needle: str, label: str) -> None:
    if needle not in text:
        die(f"{label} missing {needle!r}")


def exists_or_glob(pattern: str) -> bool:
    if any(ch in pattern for ch in "*?[]"):
        return bool(list(ROOT.glob(pattern)))
    return (ROOT / pattern).exists()


def main() -> int:
    if not CONTRACT.is_file():
        die("missing docs/tooling-source-map.json")
    try:
        data = json.loads(CONTRACT.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        die(f"invalid docs/tooling-source-map.json: {exc}")

    if data.get("schema") != "mhrv-f-tooling-source-map/v1":
        die("unexpected tooling source map schema")

    documents = data.get("documents")
    if not isinstance(documents, list):
        die("documents must be a list")

    paths = []
    for item in documents:
        if not isinstance(item, dict):
            die("each document row must be an object")
        for key in ["path", "source", "generated_by", "guarded_by", "purpose"]:
            if key not in item:
                die(f"document row missing {key!r}: {item!r}")
        path = item["path"]
        paths.append(path)
        if not exists_or_glob(path):
            die(f"mapped document does not exist: {path}")
        if not exists_or_glob(item["source"]):
            die(f"mapped source does not exist: {item['source']}")
        generated_by = item["generated_by"]
        if generated_by is not None and not exists_or_glob(generated_by):
            die(f"mapped generator does not exist: {generated_by}")
        guards = item["guarded_by"]
        if not isinstance(guards, list) or not guards:
            die(f"{path} must have at least one guard")
        for guard in guards:
            if not exists_or_glob(guard):
                die(f"mapped guard does not exist for {path}: {guard}")
        if not str(item["purpose"]).strip():
            die(f"{path} purpose must be non-empty")

    if paths != REQUIRED_DOCS:
        die(f"document paths/order mismatch: got {paths!r}")

    doc = read(DOC)
    require(doc, "docs/tooling-source-map.json", "docs/tooling-source-map.md")
    require(doc, "tools/check-tooling-source-map.py", "docs/tooling-source-map.md")
    for path in REQUIRED_DOCS:
        require(doc, path, "docs/tooling-source-map.md")

    docs_index = read(DOCS_INDEX)
    require(docs_index, "tooling-source-map.md", "docs/index.md")

    contributing = read(CONTRIBUTING)
    require(contributing, "docs/tooling-source-map.md", "CONTRIBUTING.md")
    require(contributing, "python tools\\check-tooling-source-map.py", "CONTRIBUTING.md")

    tools_readme = read(TOOLS_README)
    require(tools_readme, "Tooling source map guard", "tools/README.md")
    require(tools_readme, "check-tooling-source-map.py", "tools/README.md")

    sanity = read(SANITY)
    require(sanity, "tools/check-tooling-source-map.py", "tools/run-repo-sanity.py")
    require(sanity, "tooling source map", "tools/run-repo-sanity.py")

    parity = read(PARITY)
    require(parity, "tools/check-tooling-source-map.py", "tools/check-ci-local-sanity-parity.py")

    print("tooling source map check: ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
