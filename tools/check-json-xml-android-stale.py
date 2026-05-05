#!/usr/bin/env python3
"""Validate repo JSON/XML, Android string refs, and block stale markers.

Mirrors the logic previously inlined in `.github/workflows/ci.yml` (repo-sanity).
"""

from __future__ import annotations

import base64
import json
import pathlib
import re
import sys
import xml.etree.ElementTree as ET


def main() -> int:
    root = pathlib.Path(".")
    skip_parts = {".git", "target", "dist", "build", ".gradle"}
    allowed: set[str] = set()
    errors: list[str] = []

    def keep(path: pathlib.Path) -> bool:
        return not any(part in skip_parts for part in path.parts)

    for path in root.rglob("*.json"):
        if keep(path):
            try:
                json.loads(path.read_text(encoding="utf-8-sig"))
            except Exception as exc:
                errors.append(f"JSON parse failed: {path}: {exc}")

    for path in root.rglob("*.xml"):
        if keep(path):
            try:
                ET.parse(path)
            except Exception as exc:
                errors.append(f"XML parse failed: {path}: {exc}")

    strings_path = pathlib.Path("android/app/src/main/res/values/strings.xml")
    if strings_path.exists():
        tree = ET.parse(strings_path)
        names = {
            node.attrib["name"]
            for node in tree.findall(".//string")
            if "name" in node.attrib
        }
        refs: set[str] = set()
        for kt in pathlib.Path("android/app/src/main/java").rglob("*.kt"):
            refs.update(re.findall(r"R\.string\.([A-Za-z0-9_]+)", kt.read_text(encoding="utf-8")))
        missing = sorted(refs - names)
        if missing:
            errors.append("Missing Android strings: " + ", ".join(missing))

    stale_patterns_b64 = [
        "SW52ZXN0aWdhdGUgdG8gUG9ydA==",
        "R29vc2VSZWxheVZQTi1tYWlu",
        "Q29tbWl0cyB0byAxODMgVXBzdHJlYW0=",
        "VXBzdHJlYW0gMTg1IGNvbW1pdHM=",
        "Y29tbWl0IGFuZCBmb3J1bQ==",
        "Zm9ydW0gZXhwb3J0",
        "QHhzZmlsdGVycm5ldA==",
        "QFRoZVZQTk1ldGhvZA==",
        "QEFyY2hpdmVUZWxs",
        "QHBpbmdwbGFzX2NoYW5uZWw=",
    ]
    stale_patterns = [
        base64.b64decode(pattern).decode("utf-8") for pattern in stale_patterns_b64
    ]
    for path in root.rglob("*"):
        if not path.is_file() or not keep(path):
            continue
        if str(path).replace("\\", "/") in allowed:
            continue
        try:
            text = path.read_text(encoding="utf-8", errors="ignore")
        except Exception:
            continue
        for pattern in stale_patterns:
            if pattern in text:
                errors.append(f"Stale investigation/forum marker in {path}: {pattern}")

    if errors:
        print("\n".join(errors), file=sys.stderr)
        return 1
    print("json/xml/android-string/stale-source checks ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
