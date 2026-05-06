#!/usr/bin/env python3
"""Static guard for the bundled fronting-groups starter example.

`fronting_groups` are examples, not guaranteed routes. Still, the example file
is a user-facing migration/onboarding surface, and upstream donor value has
landed here as a curated Vercel/Fastly/Netlify starter set. This guard keeps the
JSON shape, important domain families, and docs aligned.
"""

from __future__ import annotations

import json
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
EXAMPLE = ROOT / "config.fronting-groups.example.json"
DOC = ROOT / "docs" / "fronting-groups.md"
RELAY_MODES = ROOT / "docs" / "relay-modes.md"
PARITY = ROOT / "docs" / "parity-matrix.json"


def die(msg: str) -> None:
    print(f"fronting-groups example check failed: {msg}", file=sys.stderr)
    raise SystemExit(1)


def read_text(path: Path) -> str:
    if not path.is_file():
        die(f"missing file: {path.relative_to(ROOT)}")
    return path.read_text(encoding="utf-8")


def require(text: str, needle: str, label: str) -> None:
    if needle not in text:
        die(f"missing {label}: {needle!r}")


def require_json_path(path: Path) -> object:
    try:
        return json.loads(read_text(path))
    except json.JSONDecodeError as e:
        die(f"invalid JSON in {path.relative_to(ROOT)}: {e}")


def main() -> int:
    cfg = require_json_path(EXAMPLE)
    if not isinstance(cfg, dict):
        die("example root must be a JSON object")
    if cfg.get("mode") != "direct":
        die("fronting-groups example must stay in direct mode")
    if cfg.get("listen_host") != "127.0.0.1":
        die("example should default to loopback-only listen_host")

    groups = cfg.get("fronting_groups")
    if not isinstance(groups, list) or len(groups) < 3:
        die("fronting_groups must contain Vercel, Fastly, and Netlify starter groups")

    by_name = {}
    for group in groups:
        if not isinstance(group, dict):
            die("each fronting group must be an object")
        for key in ("name", "ip", "sni", "domains"):
            if key not in group:
                die(f"group missing required key {key!r}: {group}")
        domains = group["domains"]
        if not isinstance(domains, list) or not domains or not all(isinstance(d, str) and d for d in domains):
            die(f"group {group.get('name')!r} must have non-empty string domains")
        by_name[group["name"]] = group

    for name in ("vercel", "fastly", "netlify-cloudfront"):
        if name not in by_name:
            die(f"missing starter group {name!r}")

    vercel_domains = set(by_name["vercel"]["domains"])
    fastly_domains = set(by_name["fastly"]["domains"])
    netlify_domains = set(by_name["netlify-cloudfront"]["domains"])

    for domain in ("vercel.com", "vercel.app", "nextjs.org", "cursor.com", "ai-sdk.dev"):
        if domain not in vercel_domains:
            die(f"Vercel starter missing {domain}")
    if by_name["fastly"].get("ip") != "151.101.1.140":
        die("Fastly starter should keep the known 151.101.x.x anycast example IP")
    if by_name["fastly"].get("sni") != "www.python.org":
        die("Fastly starter should keep www.python.org as the example SNI")
    for domain in (
        "reddit.com",
        "redditstatic.com",
        "redditmedia.com",
        "pinterest.com",
        "pinimg.com",
        "cnn.com",
        "cnn.io",
        "buzzfeed.com",
        "buzzfeednews.com",
        "githubassets.com",
        "githubusercontent.com",
        "pypi.org",
        "fastly.com",
    ):
        if domain not in fastly_domains:
            die(f"Fastly starter missing {domain}")
    for domain in ("netlify.com", "netlify.app"):
        if domain not in netlify_domains:
            die(f"Netlify/CloudFront starter missing {domain}")

    doc = read_text(DOC)
    relay = read_text(RELAY_MODES)
    parity = require_json_path(PARITY)

    for text, label in ((doc, "fronting-groups docs"), (relay, "relay-modes docs")):
        require(text, "config.fronting-groups.example.json", label)
        require(text, "Fastly", label)
        require(text, "Netlify", label)
    for domain in ("reddit.com", "Pinterest", "CNN", "BuzzFeed", "PyPI"):
        require(doc, domain, f"fronting-groups docs mention {domain}")
    require(doc, "keep only the domains", "docs warns examples must be verified")
    require(doc, "www.python.org", "docs Fastly SNI example")

    direct = parity.get("modes", {}).get("direct", {})
    examples = direct.get("examples", [])
    if "config.fronting-groups.example.json" not in examples:
        die("parity matrix direct mode must reference config.fronting-groups.example.json")

    print("fronting-groups example check: ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
