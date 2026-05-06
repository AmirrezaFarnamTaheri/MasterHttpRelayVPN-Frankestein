#!/usr/bin/env python3
"""Guard the Android copied support-snapshot schema documentation.

The redaction owner guard checks the Kotlin implementation. This companion
guard keeps the human-facing schema document, docs index, JVM-test source, and
repo-sanity wiring in lockstep with the active copied-text marker.
"""

from __future__ import annotations

import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
SUPPORT_REDACTION = ROOT / "android/app/src/main/java/com/farnam/mhrvf/SupportRedaction.kt"
SUPPORT_TEST = ROOT / "android/app/src/test/java/com/farnam/mhrvf/SupportRedactionTest.kt"
SCHEMA_DOC = ROOT / "docs/android-support-snapshot.md"
DOC_INDEX = ROOT / "docs/index.md"
ANDROID_DOC = ROOT / "docs/android.md"
ANDROID_FA_DOC = ROOT / "docs/android.fa.md"
TRUST_DOC = ROOT / "docs/trust-center.md"
TOOLS_README = ROOT / "tools/README.md"
LOCAL_SANITY = ROOT / "tools/run-repo-sanity.py"

SCHEMA = "android-support-snapshot/v2"


def die(message: str) -> None:
    print(f"android support snapshot schema check failed: {message}", file=sys.stderr)
    raise SystemExit(1)


def read(path: Path) -> str:
    if not path.is_file():
        die(f"missing required file: {path.relative_to(ROOT)}")
    return path.read_text(encoding="utf-8")


def require_all(text: str, needles: list[str], *, label: str) -> None:
    missing = [needle for needle in needles if needle not in text]
    if missing:
        formatted = ", ".join(repr(needle) for needle in missing)
        die(f"{label} is missing required marker(s): {formatted}")


def main() -> int:
    support = read(SUPPORT_REDACTION)
    tests = read(SUPPORT_TEST)
    schema_doc = read(SCHEMA_DOC)
    doc_index = read(DOC_INDEX)
    android_doc = read(ANDROID_DOC)
    android_fa_doc = read(ANDROID_FA_DOC)
    trust_doc = read(TRUST_DOC)
    tools_readme = read(TOOLS_README)
    local_sanity = read(LOCAL_SANITY)

    require_all(
        support,
        [
            f"schema: {SCHEMA}",
            "doctor_available",
            "doctor_problem_ids",
        ],
        label="SupportRedaction.kt",
    )
    require_all(
        tests,
        [
            f"schema: {SCHEMA}",
            "androidSupportSnapshotIncludesDoctorSummaryWithoutDetails",
            "doctor_problem_ids: apps_script_urls, auth_key",
        ],
        label="SupportRedactionTest.kt",
    )
    require_all(
        schema_doc,
        [
            f"schema: {SCHEMA}",
            "doctor_available",
            "doctor_ok",
            "doctor_items_total",
            "doctor_items_ok",
            "doctor_items_warn",
            "doctor_items_fail",
            "doctor_problem_ids",
            "raw Doctor JSON",
            "Doctor titles, details, fixes, endpoint URLs",
            "SupportRedaction.kt",
            "tools/check-android-support-redaction.py",
        ],
        label="docs/android-support-snapshot.md",
    )
    require_all(
        doc_index,
        [
            "Android redacted support snapshot schema",
            "docs/android-support-snapshot.md",
        ],
        label="docs/index.md",
    )
    for label, text in [
        ("docs/android.md", android_doc),
        ("docs/android.fa.md", android_fa_doc),
        ("docs/trust-center.md", trust_doc),
        ("tools/README.md", tools_readme),
    ]:
        require_all(
            text,
            [
                "android-support-snapshot.md",
                "Doctor",
            ],
            label=label,
        )

    require_all(
        local_sanity,
        [
            "tools/check-android-support-snapshot-schema.py",
        ],
        label="tools/run-repo-sanity.py",
    )

    print("Android support snapshot schema check: ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
