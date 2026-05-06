#!/usr/bin/env python3
"""Check Android English/Persian string resource parity.

Local static guard only: no Gradle, no Android SDK. It catches missing or extra
translation keys before Compose code can reference strings that exist in one
locale but not the other.
"""

from __future__ import annotations

import sys
import xml.etree.ElementTree as ET
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
EN = ROOT / "android/app/src/main/res/values/strings.xml"
FA = ROOT / "android/app/src/main/res/values-fa/strings.xml"

REQUIRED_KEYS = {
    "app_name",
    "field_mode",
    "sec_apps_script_relay",
    "sec_serverless_json_relay",
    "help_section_mode",
    "help_section_serverless_json",
    "btn_connect",
    "btn_save_and_connect",
    "btn_disconnect",
    "btn_install_mitm",
    "repair_account_groups_label",
    "repair_serverless_base_label",
    "repair_direct_ip_label",
    "repair_ca_trust_label",
    "repair_lan_exposure_label",
    "repair_full_health_label",
}


def die(msg: str) -> None:
    print(f"Android string resource parity check failed: {msg}", file=sys.stderr)
    raise SystemExit(1)


def read_strings(path: Path) -> dict[str, str]:
    if not path.is_file():
        die(f"missing file: {path.relative_to(ROOT)}")
    try:
        tree = ET.parse(path)
    except Exception as exc:
        die(f"XML parse failed for {path.relative_to(ROOT)}: {exc}")

    out: dict[str, str] = {}
    for node in tree.findall(".//string"):
        name = node.attrib.get("name")
        if not name:
            continue
        if name in out:
            die(f"duplicate string name {name!r} in {path.relative_to(ROOT)}")
        out[name] = "".join(node.itertext()).strip()
    return out


def main() -> int:
    en = read_strings(EN)
    fa = read_strings(FA)

    missing_fa = sorted(set(en) - set(fa))
    extra_fa = sorted(set(fa) - set(en))
    if missing_fa:
        die("missing values-fa keys: " + ", ".join(missing_fa))
    if extra_fa:
        die("extra values-fa keys: " + ", ".join(extra_fa))

    missing_required = sorted(REQUIRED_KEYS - set(en))
    if missing_required:
        die("missing required English keys: " + ", ".join(missing_required))

    blank_en = sorted(name for name, value in en.items() if not value)
    blank_fa = sorted(name for name, value in fa.items() if not value)
    if blank_en:
        die("blank English strings: " + ", ".join(blank_en))
    if blank_fa:
        die("blank Persian strings: " + ", ".join(blank_fa))

    print(f"Android string resource parity check: ok keys={len(en)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
