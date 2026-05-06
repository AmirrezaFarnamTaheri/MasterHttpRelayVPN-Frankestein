#!/usr/bin/env python3
"""Guard Android's structured Doctor JNI bridge.

Android should consume the same Doctor JSON contract as support bundles and
future UI cards. This guard keeps the JNI bridge from drifting into a separate
hand-maintained diagnostics shape.
"""

from __future__ import annotations

import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
JNI = ROOT / "src" / "android_jni.rs"
NATIVE = ROOT / "android" / "app" / "src" / "main" / "java" / "com" / "farnam" / "mhrvf" / "Native.kt"
DOC = ROOT / "docs" / "doctor-json-contract.md"


def die(msg: str) -> None:
    print(f"Android Doctor JNI bridge guard failed: {msg}", file=sys.stderr)
    raise SystemExit(1)


def require(text: str, needle: str, label: str) -> None:
    if needle not in text:
        die(f"missing {label}: {needle!r}")


def main() -> int:
    for path in (JNI, NATIVE, DOC):
        if not path.is_file():
            die(f"missing file: {path}")

    jni = JNI.read_text(encoding="utf-8")
    native = NATIVE.read_text(encoding="utf-8")
    doc = DOC.read_text(encoding="utf-8")

    require(
        jni,
        "Java_com_farnam_mhrvf_Native_doctorJson",
        "JNI export name",
    )
    require(jni, "Config::from_json_str(&json)", "config parsing through Rust Config")
    require(jni, "crate::doctor::run(&config)", "shared Doctor execution")
    require(
        jni,
        "crate::doctor::doctor_report_json_value(&report).to_string()",
        "shared Doctor JSON renderer",
    )
    require(jni, '"id": "config_json"', "contract-shaped invalid-config error")
    require(jni, '"id": "tokio_runtime"', "contract-shaped runtime-init error")

    require(native, "external fun doctorJson(configJson: String): String", "Kotlin Native declaration")
    require(native, "shared Doctor JSON contract", "Kotlin contract comment")

    require(doc, "Android `Native.doctorJson(configJson)`", "Doctor contract consumer row")
    require(
        doc,
        "tools/check-android-doctor-jni-bridge.py",
        "Doctor contract mentions Android guard",
    )

    print("Android Doctor JNI bridge guard: ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
