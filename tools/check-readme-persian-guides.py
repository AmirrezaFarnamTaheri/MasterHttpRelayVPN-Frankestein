#!/usr/bin/env python3
"""Static guard for the README Persian guide links.

The upstream v1.9.7 README iteration added Persian video/text setup resources,
then collapsed them into a compact two-item list. This fork intentionally keeps
that compact form near the language switcher: useful for onboarding, but not a
large YouTube thumbnail that dominates the first screen.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
README = ROOT / "README.md"


def die(msg: str) -> None:
    print(f"README Persian guide check failed: {msg}", file=sys.stderr)
    raise SystemExit(1)


def require(text: str, needle: str, label: str) -> None:
    if needle not in text:
        die(f"missing {label}: {needle!r}")


def main() -> int:
    if not README.is_file():
        die(f"missing {README}")
    text = README.read_text(encoding="utf-8")

    require(text, "**[English Guide](#setup-guide)**", "English guide language switcher")
    require(text, "**[راهنمای فارسی](#راهنمای-فارسی)**", "Persian guide language switcher")
    require(text, '<p align="center" dir="rtl">', "centered RTL guide block")
    require(text, "https://www.youtube.com/watch?v=voCwxgvWR5U", "Persian setup video link")
    require(text, "راهنمای تصویری راه‌اندازی به زبان فارسی", "Persian video label")
    require(text, "https://kian-irani.github.io/mhrv-setup-full-tunell/", "Kian Irani text guide link")
    require(text, "راهنمای جامع متنی راه‌اندازی به زبان فارسی", "Persian text guide label")
    require(text, "https://github.com/KIAN-IRANi", "Kian Irani credit link")
    require(text, 'target="_blank" rel="noopener noreferrer"', "external-link safety attrs")

    # Keep it compact: no YouTube thumbnail embed in the first viewport.
    if "img.youtube.com/vi/" in text:
        die("README reintroduced a YouTube thumbnail embed; keep compact text links")
    if re.search(r"<img\b[^>]*youtube", text, flags=re.I):
        die("README reintroduced an embedded YouTube image")

    first_120 = "\n".join(text.splitlines()[:120])
    if "https://www.youtube.com/watch?v=voCwxgvWR5U" not in first_120:
        die("Persian setup video link drifted out of the README first screen")
    if "https://kian-irani.github.io/mhrv-setup-full-tunell/" not in first_120:
        die("Persian text guide link drifted out of the README first screen")

    print("README Persian guide check: ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
