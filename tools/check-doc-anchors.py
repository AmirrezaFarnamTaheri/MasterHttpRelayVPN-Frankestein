#!/usr/bin/env python3
"""Validate local Markdown heading anchors for intra-repo links.

`tools/check-doc-links.py` verifies that local link *targets* exist.
This script verifies that `file.md#anchor` fragments match actual headings
in the target Markdown file using GitHub-style slug rules (roughly).

Scope: maintained doc roots (same as check-doc-links.py).
"""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
import re
import sys
import unicodedata
from urllib.parse import unquote, urlparse


ROOT = Path.cwd()

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

HEADING_RE = re.compile(r"^(#{1,6})\s+(.+?)\s*$")


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
        target = target.split(" ", 1)[0]
    if target.startswith("<") and target.endswith(">"):
        target = target[1:-1]
    return target


def should_skip(target: str) -> bool:
    if not target:
        return True
    parsed = urlparse(target)
    if parsed.scheme in SKIP_SCHEMES:
        return True
    if parsed.scheme and parsed.scheme not in {"", "file"}:
        return True
    return False


def split_link(target: str) -> tuple[str, str | None]:
    # returns (path_part, fragment_without_hash or None)
    if "#" not in target:
        return target, None
    path_part, frag = target.split("#", 1)
    if frag == "":
        return path_part, None
    return path_part, frag


def resolve_path(source: Path, raw_path: str) -> Path | None:
    if not raw_path:
        return None
    decoded = unquote(raw_path.split("?", 1)[0])
    if decoded.startswith("file:"):
        return None
    return (source.parent / decoded).resolve()


def github_slug(text: str) -> str:
    # Approximate GitHub heading slug algorithm.
    # - strip markdown inline code/backticks
    # - casefold (unicode-aware)
    # - keep unicode letters/digits, spaces, and hyphens; drop punctuation
    # - replace spaces with hyphens
    # - keep repeated hyphens (GitHub does; e.g. spaces around an em dash)
    t = text.strip()
    t = re.sub(r"`([^`]+)`", r"\1", t)
    t = t.casefold()
    kept: list[str] = []
    for ch in t:
        cat = unicodedata.category(ch)
        if ch.isalnum() or cat.startswith("M"):
            kept.append(ch)
        elif ch in {" ", "-", "\u200c", "\u200d"}:
            kept.append(ch)
        else:
            # Drop punctuation and symbols (including zero-width joiners).
            continue
    t = "".join(kept)
    t = t.replace(" ", "-").strip("-")
    return t


def heading_anchors(md_text: str) -> set[str]:
    counts: dict[str, int] = {}
    anchors: set[str] = set()
    for line in md_text.splitlines():
        m = HEADING_RE.match(line)
        if not m:
            continue
        title = m.group(2)
        base = github_slug(title)
        if not base:
            continue
        n = counts.get(base, 0)
        counts[base] = n + 1
        if n == 0:
            anchors.add(base)
        else:
            anchors.add(f"{base}-{n}")
    return anchors


@dataclass(frozen=True)
class AnchorRef:
    source: Path
    target: Path
    fragment: str


def collect_anchor_refs(root: Path) -> list[AnchorRef]:
    refs: list[AnchorRef] = []
    for md in iter_markdown_files(root):
        text = md.read_text(encoding="utf-8", errors="ignore")
        for regex in (INLINE_LINK_RE, IMAGE_LINK_RE):
            for match in regex.finditer(text):
                raw = clean_target(match.group(1))
                if should_skip(raw):
                    continue
                path_part, frag = split_link(raw)
                if frag is None:
                    continue
                # Pure in-page anchors (#x) are handled by this checker too.
                if raw.startswith("#"):
                    refs.append(AnchorRef(source=md, target=md, fragment=frag))
                    continue
                target_path = resolve_path(md, path_part)
                if target_path is None:
                    continue
                try:
                    target_path.relative_to(root.resolve())
                except ValueError:
                    continue
                if target_path.suffix.lower() != ".md":
                    continue
                refs.append(AnchorRef(source=md, target=target_path, fragment=frag))
    return refs


def main() -> int:
    refs = collect_anchor_refs(ROOT)
    if not refs:
        print("markdown anchor checks ok (no anchors found)")
        return 0

    cache: dict[Path, set[str]] = {}
    errors: list[str] = []

    for ref in refs:
        if not ref.target.exists():
            # file existence is handled by check-doc-links
            continue
        anchors = cache.get(ref.target)
        if anchors is None:
            anchors = heading_anchors(ref.target.read_text(encoding="utf-8", errors="ignore"))
            cache[ref.target] = anchors
        if ref.fragment not in anchors:
            errors.append(
                f"{ref.source.relative_to(ROOT).as_posix()}: missing anchor "
                f"'{ref.fragment}' in {ref.target.relative_to(ROOT).as_posix()}"
            )

    if errors:
        print("Markdown anchor check failed:", file=sys.stderr)
        for e in errors:
            print(f"- {e}", file=sys.stderr)
        return 1

    print(f"markdown anchor checks ok ({len(refs)} anchors validated)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

