#!/usr/bin/env python3
"""Guard the shared status/stats JSON renderer contract.

The local `/status` endpoint, support bundle status snapshot, and Android JNI
stats bridge should not hand-maintain separate `StatsSnapshot` JSON field
lists. Android may return the raw stats object for its lightweight polling UI,
but it must get that object from `status_api::stats_snapshot_json_value`.
"""

from __future__ import annotations

import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
STATUS_API = ROOT / "src/status_api.rs"
ANDROID_JNI = ROOT / "src/android_jni.rs"
ANDROID_HOME = ROOT / "android/app/src/main/java/com/farnam/mhrvf/ui/HomeScreen.kt"
DOC = ROOT / "docs/status-stats-json-contract.md"
DOC_INDEX = ROOT / "docs/index.md"
TOOLS_README = ROOT / "tools/README.md"

REQUIRED_KEYS = [
    "relay_calls",
    "relay_failures",
    "cache_hits",
    "cache_misses",
    "cache_bytes",
    "bytes_relayed",
    "coalesced",
    "scripts_total",
    "scripts_blacklisted",
    "total_scripts",
    "blacklisted_scripts",
    "today_calls",
    "today_bytes",
    "today_reset_secs",
    "degrade_level",
    "degrade_reason",
]


def die(msg: str) -> None:
    print(f"status/stats JSON contract check failed: {msg}", file=sys.stderr)
    raise SystemExit(1)


def main() -> int:
    status = STATUS_API.read_text(encoding="utf-8")
    android = ANDROID_JNI.read_text(encoding="utf-8")
    home = ANDROID_HOME.read_text(encoding="utf-8")
    doc = DOC.read_text(encoding="utf-8")
    doc_index = DOC_INDEX.read_text(encoding="utf-8")
    tools_readme = TOOLS_README.read_text(encoding="utf-8")

    if "pub fn stats_snapshot_json_value" not in status:
        die("missing shared stats_snapshot_json_value renderer")
    for key in REQUIRED_KEYS:
        if f'"{key}"' not in status:
            die(f"shared renderer missing key {key!r}")

    if "stats_snapshot_json_value(fronter.snapshot_stats())" not in android:
        die("Android JNI statsJson must call the shared status_api renderer")

    marker = "pub extern \"system\" fn Java_com_farnam_mhrvf_Native_statsJson"
    start = android.find(marker)
    if start == -1:
        die("missing Android statsJson JNI function")
    body = android[start:]
    forbidden = [
        '"relay_calls"',
        '"today_calls"',
        '"scripts_total"',
        '"total_scripts"',
        '"degrade_level"',
    ]
    for needle in forbidden:
        if needle in body:
            die(f"Android statsJson hand-serializes {needle}; use shared renderer")

    if "private fun statsPayloadFromJson" not in home:
        die("Android Usage Today card must parse stats through statsPayloadFromJson")
    if 'root.optJSONObject("stats") ?: root' not in home:
        die("Android stats parser must accept both /status envelope and raw stats JSON")
    if "statsPayloadFromJson(statsJson)" not in home:
        die("Usage Today card must use the envelope-tolerant stats parser")

    if "status-stats-json-contract.md" not in doc_index:
        die("docs/index.md must link the status/stats JSON contract")
    if "status-stats-json-contract.md" not in tools_readme:
        die("tools/README.md must link the status/stats JSON contract")
    for key in REQUIRED_KEYS:
        if f"`{key}`" not in doc:
            die(f"contract doc missing required key {key!r}")
    required_doc_phrases = [
        "status_api::stats_snapshot_json_value",
        "`/status`",
        "`status.json`",
        "`Native.statsJson(handle)`",
        'root.optJSONObject("stats") ?: root',
    ]
    for phrase in required_doc_phrases:
        if phrase not in doc:
            die(f"contract doc missing phrase {phrase!r}")

    print("status/stats JSON contract check: ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
