#!/usr/bin/env python3
"""Static/rendering checks for the Telegram release notification script."""

from __future__ import annotations

import importlib.util
import sys
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
SCRIPT = ROOT / ".github" / "scripts" / "telegram_release_notify.py"


def die(msg: str) -> None:
    print(f"telegram release notify check failed: {msg}", file=sys.stderr)
    raise SystemExit(1)


def load_module():
    spec = importlib.util.spec_from_file_location("telegram_release_notify", SCRIPT)
    if spec is None or spec.loader is None:
        die(f"cannot load {SCRIPT}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def main() -> int:
    if not SCRIPT.is_file():
        die(f"missing script: {SCRIPT}")
    text = SCRIPT.read_text(encoding="utf-8")
    for needle in [
        "def md_to_tg_html",
        "def build_changelog_reply",
        "TG_MESSAGE_BUDGET",
        "html_escape",
        "See full notes on GitHub.",
    ]:
        if needle not in text:
            die(f"missing renderer contract marker: {needle}")

    tg = load_module()

    sample = (
        "<!-- editor note -->\n"
        "• **مهم**: مقدار `auth_key` و [راهنما](https://example.com?a=1&b=2)\n"
        "---\n"
        "• **Important**: keep `<secret>` inside `config.json`."
    )
    with tempfile.TemporaryDirectory() as tmp:
        path = Path(tmp) / "v9.9.9.md"
        path.write_text(sample, encoding="utf-8")
        fa, en = tg.parse_changelog(str(path))

    html = tg.md_to_tg_html(fa)
    if "<b>مهم</b>" not in html:
        die("bold markdown was not converted")
    if "<code>auth_key</code>" not in html:
        die("inline code markdown was not converted")
    if '<a href="https://example.com?a=1&b=2">راهنما</a>' not in html:
        die("markdown link was not converted")
    if "<!--" in html:
        die("leading HTML comment leaked into Telegram text")

    reply = tg.build_changelog_reply(fa, en)
    if len(reply) > tg.TG_MESSAGE_BUDGET + 16:
        die(f"reply too long: {len(reply)}")
    if "<secret>" in reply:
        die("raw HTML-like text was not escaped")
    if "&lt;secret&gt;" not in reply:
        die("escaped HTML-like text missing")
    if not reply.startswith("<blockquote>") or "</blockquote>" not in reply:
        die("reply blockquote structure missing")

    long_html = tg.md_to_tg_html("\n".join([f"• line {i}" for i in range(1000)]), max_len=300)
    if len(long_html) > 360:
        die("truncated Telegram HTML exceeded expected budget")
    if "See full notes on GitHub." not in long_html:
        die("truncation footer missing")

    print("telegram release notify check: ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
