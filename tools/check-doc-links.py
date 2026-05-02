#!/usr/bin/env python3
"""Check local Markdown links in maintained docs.

The check is intentionally local-only: external HTTP(S), mailto, and pure
anchor links are skipped. Relative links must point to an existing file or
directory after URL decoding and fragment stripping.
"""

from __future__ import annotations

from pathlib import Path
import re
import sys
from urllib.parse import unquote, urlparse


DOC_ROOTS = [
    Path("README.md"),
    Path("SF_README.md"),
    Path("docs"),
    Path("tools/README.md"),
    Path("assets/apps_script/README.md"),
    Path("tunnel-node/README.md"),
    Path("releases/README.md"),
]

SKIP_SCHEMES = {"http", "https", "mailto", "tel", "data"}
SKIP_DIRS = {".git", "target", "dist", "build", ".gradle", ".gradle-user-home"}

INLINE_LINK_RE = re.compile(r"(?<!!)\[[^\]]+\]\(([^)\s]+(?:\s+\"[^\"]*\")?)\)")
IMAGE_LINK_RE = re.compile(r"!\[[^\]]*\]\(([^)\s]+(?:\s+\"[^\"]*\")?)\)")


def iter_markdown_files(root: Path) -> list[Path]:
    files: list[Path] = []
    for item in DOC_ROOTS:
        path = root / item
        if not path.exists():
            continue
        if path.is_file():
            files.append(path)
            continue
        for md in path.rglob("*.md"):
            if any(part in SKIP_DIRS for part in md.relative_to(root).parts):
                continue
            files.append(md)
    return sorted(set(files))


def clean_target(raw: str) -> str:
    target = raw.strip()
    if " " in target:
        # Drop an optional Markdown title: [x](target "title").
        target = target.split(" ", 1)[0]
    if target.startswith("<") and target.endswith(">"):
        target = target[1:-1]
    return target


def should_skip(target: str) -> bool:
    if not target or target.startswith("#"):
        return True
    parsed = urlparse(target)
    if parsed.scheme in SKIP_SCHEMES:
        return True
    if parsed.scheme and parsed.scheme not in {"", "file"}:
        return True
    return False


def target_path(source: Path, raw_target: str) -> Path | None:
    target = clean_target(raw_target)
    if should_skip(target):
        return None
    if target.startswith("file:"):
        return None
    without_fragment = target.split("#", 1)[0].split("?", 1)[0]
    if not without_fragment:
        return None
    decoded = unquote(without_fragment)
    return (source.parent / decoded).resolve()


def check(root: Path) -> list[str]:
    errors: list[str] = []
    root_resolved = root.resolve()

    for md in iter_markdown_files(root):
        text = md.read_text(encoding="utf-8", errors="ignore")
        for regex in (INLINE_LINK_RE, IMAGE_LINK_RE):
            for match in regex.finditer(text):
                raw = match.group(1)
                resolved = target_path(md, raw)
                if resolved is None:
                    continue
                try:
                    resolved.relative_to(root_resolved)
                except ValueError:
                    errors.append(
                        f"{md.relative_to(root).as_posix()}: link escapes repo: {raw}"
                    )
                    continue
                if not resolved.exists():
                    errors.append(
                        f"{md.relative_to(root).as_posix()}: missing local link target: {raw}"
                    )

    return errors


def main() -> int:
    root = Path.cwd()
    errors = check(root)
    if errors:
        print("Markdown local link check failed:", file=sys.stderr)
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return 1
    print("markdown local link checks ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
