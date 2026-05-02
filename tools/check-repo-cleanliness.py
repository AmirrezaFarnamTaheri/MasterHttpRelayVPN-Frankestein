#!/usr/bin/env python3
"""Repository hygiene checks for release and CI.

The check intentionally treats local archive/build folders as informational:
the maintainer chose to keep `dist/` and `releases/` as backup material while
CI remains the release source of truth. Source-tree hygiene issues still fail.
"""

from __future__ import annotations

import argparse
import os
from pathlib import Path
import re
import sys


DEFAULT_MAX_SOURCE_BYTES = 1_000_000

SKIP_DIRS = {
    ".git",
    ".gradle",
    ".gradle-user-home",
    ".idea",
    ".vscode",
    "__pycache__",
    "build",
    "target",
}

ARCHIVE_DIRS = {"dist", "releases"}

ALLOWED_LARGE_FILES = {
    "Cargo.lock",
    "elevation_audit_roadmap_source.md",
}

LOCAL_SECRET_NAMES = {
    ".env",
    "config.json",
    "local.properties",
}

SOURCE_IMAGE_EXTS = {".png", ".jpg", ".jpeg", ".gif", ".webp"}
ALLOW_IMAGE_REFERENCES_IN = {
    "elevation_audit_roadmap_source.md",
}

SCREENSHOT_LINK_RE = re.compile(
    r"(!\[[^\]]*\]\([^)]+\.(?:png|jpg|jpeg|gif|webp)(?:#[^)]+)?\)|"
    r"\b[\w./-]+\.(?:png|jpg|jpeg|gif|webp)\b)",
    re.IGNORECASE,
)


def rel(path: Path, root: Path) -> str:
    return path.relative_to(root).as_posix()


def should_skip(path: Path, root: Path) -> bool:
    parts = path.relative_to(root).parts
    return any(part in SKIP_DIRS for part in parts)


def under_archive(path: Path, root: Path) -> bool:
    parts = path.relative_to(root).parts
    return bool(parts) and parts[0] in ARCHIVE_DIRS


def iter_files(root: Path):
    for current_root, dirs, files in os.walk(root):
        current = Path(current_root)
        rel_parts = current.relative_to(root).parts
        if any(part in SKIP_DIRS for part in rel_parts):
            dirs[:] = []
            continue
        dirs[:] = [d for d in dirs if d not in SKIP_DIRS]
        for name in files:
            yield current / name


def check(root: Path, max_source_bytes: int) -> tuple[list[str], list[str]]:
    errors: list[str] = []
    notes: list[str] = []

    for dirname in ARCHIVE_DIRS:
        archive_dir = root / dirname
        if archive_dir.exists():
            notes.append(f"archive directory present (allowed backup): {dirname}/")
            if dirname == "releases" and not (archive_dir / "README.md").exists():
                errors.append("releases/ exists but has no README.md explaining artifact policy")

    for path in iter_files(root):
        relative = rel(path, root)
        if under_archive(path, root):
            continue
        if should_skip(path, root):
            continue

        lower_name = path.name.lower()
        if lower_name in LOCAL_SECRET_NAMES:
            errors.append(f"local-only secret/config file in source tree: {relative}")

        if path.suffix.lower() in {".apk", ".aab", ".exe", ".dll", ".dylib", ".so"}:
            errors.append(f"binary build artifact in source tree: {relative}")

        size = path.stat().st_size
        if size > max_source_bytes and path.name not in ALLOWED_LARGE_FILES:
            errors.append(
                f"large source-tree file: {relative} ({size} bytes > {max_source_bytes})"
            )

        if path.suffix.lower() in SOURCE_IMAGE_EXTS:
            errors.append(f"image asset in source docs/tree needs explicit policy: {relative}")

        if (
            path.suffix.lower() in {".md", ".txt", ".rst"}
            and relative not in ALLOW_IMAGE_REFERENCES_IN
        ):
            text = path.read_text(encoding="utf-8", errors="ignore")
            for match in SCREENSHOT_LINK_RE.finditer(text):
                snippet = match.group(0)
                if "screenshot" in snippet.lower() or snippet.lower().endswith(
                    (".png", ".jpg", ".jpeg", ".gif", ".webp")
                ):
                    errors.append(
                        f"stale-prone screenshot/image reference in {relative}: {snippet}"
                    )

    return errors, notes


def main() -> int:
    parser = argparse.ArgumentParser(description="Check repository cleanliness.")
    parser.add_argument(
        "--max-source-bytes",
        type=int,
        default=DEFAULT_MAX_SOURCE_BYTES,
        help="Maximum size for ordinary source files.",
    )
    args = parser.parse_args()

    root = Path.cwd()
    errors, notes = check(root, args.max_source_bytes)

    for note in notes:
        print(f"note: {note}")

    if errors:
        print("Repository cleanliness check failed:", file=sys.stderr)
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return 1

    print("repo cleanliness checks ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
