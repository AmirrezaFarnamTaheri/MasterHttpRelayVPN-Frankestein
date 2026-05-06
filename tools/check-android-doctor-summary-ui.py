#!/usr/bin/env python3
"""Guard Android's Doctor summary card.

Android Doctor UI must consume `Native.doctorJson(configJson)`, parse the shared
Doctor JSON contract, avoid stale-result races when config changes mid-run, and
keep all visible copy in string resources.
"""

from __future__ import annotations

import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
HOME = ROOT / "android" / "app" / "src" / "main" / "java" / "com" / "farnam" / "mhrvf" / "ui" / "HomeScreen.kt"
STRINGS_EN = ROOT / "android" / "app" / "src" / "main" / "res" / "values" / "strings.xml"
STRINGS_FA = ROOT / "android" / "app" / "src" / "main" / "res" / "values-fa" / "strings.xml"

REQUIRED_STRINGS = [
    "sec_doctor_summary",
    "doctor_summary_hint",
    "doctor_status_not_run",
    "doctor_status_running",
    "doctor_status_ok",
    "doctor_status_attention",
    "doctor_counts",
    "doctor_fix_prefix",
    "doctor_more_items",
    "btn_run_doctor",
    "doctor_running_button",
    "doctor_stale_result_ignored",
]


def die(msg: str) -> None:
    print(f"Android Doctor summary UI guard failed: {msg}", file=sys.stderr)
    raise SystemExit(1)


def require(text: str, needle: str, label: str) -> None:
    if needle not in text:
        die(f"missing {label}: {needle!r}")


def main() -> int:
    for path in (HOME, STRINGS_EN, STRINGS_FA):
        if not path.is_file():
            die(f"missing file: {path}")

    home = HOME.read_text(encoding="utf-8")
    strings_en = STRINGS_EN.read_text(encoding="utf-8")
    strings_fa = STRINGS_FA.read_text(encoding="utf-8")

    require(home, "private data class AndroidDoctorItem", "Doctor item model")
    require(home, "private data class AndroidDoctorReport", "Doctor report model")
    require(home, "private fun parseAndroidDoctorReport", "Doctor JSON parser")
    require(home, 'obj.optJSONArray("items")', "items array parser")
    require(home, 'item.optString("level", "warn")', "level parser")
    require(home, "private fun DoctorSummaryCard(", "Doctor summary composable")
    require(home, "var doctorReport by remember", "Doctor report state")
    require(home, "var doctorRunning by remember", "Doctor running state")
    require(home, "val configSnapshot = cfg.toJson()", "config snapshot before Doctor run")
    require(home, "Native.doctorJson(configSnapshot)", "native Doctor bridge call")
    require(home, "if (configSnapshot == cfg.toJson())", "stale result guard")
    require(home, "DoctorSummaryCard(", "Home screen renders Doctor card")

    for key in REQUIRED_STRINGS:
        require(strings_en, f'name="{key}"', f"English string {key}")
        require(strings_fa, f'name="{key}"', f"Persian string {key}")
        require(home, f"R.string.{key}", f"HomeScreen string usage {key}")

    print("Android Doctor summary UI guard: ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
