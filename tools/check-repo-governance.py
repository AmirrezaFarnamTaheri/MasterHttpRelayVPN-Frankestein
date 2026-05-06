#!/usr/bin/env python3
"""Guard contributor, issue, PR, ownership, and security governance files."""

from __future__ import annotations

import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent

FILES = {
    "CONTRIBUTING.md": ROOT / "CONTRIBUTING.md",
    "SECURITY.md": ROOT / "SECURITY.md",
    "docs/ownership.md": ROOT / "docs/ownership.md",
    ".github/pull_request_template.md": ROOT / ".github/pull_request_template.md",
    ".github/ISSUE_TEMPLATE/bug_report.yml": ROOT / ".github/ISSUE_TEMPLATE/bug_report.yml",
    ".github/ISSUE_TEMPLATE/android_problem.yml": ROOT / ".github/ISSUE_TEMPLATE/android_problem.yml",
    ".github/ISSUE_TEMPLATE/backend_helper_problem.yml": ROOT / ".github/ISSUE_TEMPLATE/backend_helper_problem.yml",
    ".github/ISSUE_TEMPLATE/feature_request.yml": ROOT / ".github/ISSUE_TEMPLATE/feature_request.yml",
    "docs/index.md": ROOT / "docs/index.md",
    "tools/README.md": ROOT / "tools/README.md",
    "tools/run-repo-sanity.py": ROOT / "tools/run-repo-sanity.py",
}


def die(message: str) -> None:
    print(f"repo governance check failed: {message}", file=sys.stderr)
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
    contributing = read("CONTRIBUTING.md")
    security = read("SECURITY.md")
    ownership = read("docs/ownership.md")
    pr_template = read(".github/pull_request_template.md")
    bug = read(".github/ISSUE_TEMPLATE/bug_report.yml")
    android = read(".github/ISSUE_TEMPLATE/android_problem.yml")
    backend = read(".github/ISSUE_TEMPLATE/backend_helper_problem.yml")
    feature = read(".github/ISSUE_TEMPLATE/feature_request.yml")
    docs_index = read("docs/index.md")
    tools_readme = read("tools/README.md")
    sanity = read("tools/run-repo-sanity.py")

    require_all(
        contributing,
        [
            "python tools\\run-repo-sanity.py",
            "cargo clippy --all-targets --all-features -- -D warnings",
            "Android JVM tests are CI/pre-provisioned",
            "Desktop UI",
            "Android UI",
            "backend helpers",
            "docs/changelog/TEMPLATE.md",
            "elevation_audit_roadmap_source.md",
        ],
        label="CONTRIBUTING.md",
    )
    require_all(
        security,
        [
            "Reporting Vulnerabilities",
            "Supported Versions",
            "Secret Handling",
            "Trust Model Caveats",
            "Security-Sensitive Changes",
            "GitHub Releases and `SHA256SUMS.txt` are the canonical release source",
        ],
        label="SECURITY.md",
    )
    require_all(
        ownership,
        [
            "Rust core",
            "Desktop UI",
            "Android",
            "Backend helpers",
            "tunnel-node",
            "Security/trust",
            "Future CODEOWNERS",
        ],
        label="docs/ownership.md",
    )
    require_all(
        pr_template,
        [
            "Parity Checklist",
            "Desktop impact reviewed",
            "Android impact reviewed",
            "Config/schema/readiness/status contracts reviewed",
            "Support-bundle/redaction impact reviewed",
            "Security / Trust",
            "Stale/deprecated code/docs removed",
        ],
        label=".github/pull_request_template.md",
    )
    for label, text, markers in [
        (
            ".github/ISSUE_TEMPLATE/bug_report.yml",
            bug,
            ["Redacted diagnostics", "I removed auth keys", "Platform"],
        ),
        (
            ".github/ISSUE_TEMPLATE/android_problem.yml",
            android,
            ["Android connection mode", "Copy redacted support snapshot", "deployment IDs"],
        ),
        (
            ".github/ISSUE_TEMPLATE/backend_helper_problem.yml",
            backend,
            ["Compatibility marker", "Apps Script CodeFull.gs", "tunnel-node"],
        ),
        (
            ".github/ISSUE_TEMPLATE/feature_request.yml",
            feature,
            ["Parity notes", "Main surface", "Release/governance"],
        ),
    ]:
        require_all(text, markers, label=label)

    require_all(
        docs_index,
        [
            "CONTRIBUTING.md",
            "SECURITY.md",
            "ownership.md",
        ],
        label="docs/index.md",
    )
    require_all(
        tools_readme,
        [
            "check-repo-governance.py",
        ],
        label="tools/README.md",
    )
    require_all(
        sanity,
        [
            "tools/check-repo-governance.py",
        ],
        label="tools/run-repo-sanity.py",
    )

    print("repo governance check: ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
