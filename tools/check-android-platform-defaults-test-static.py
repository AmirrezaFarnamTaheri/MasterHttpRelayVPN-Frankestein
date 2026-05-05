#!/usr/bin/env python3
"""Static drift gate: PlatformDefaultsContractTest.kt must exercise every contract field.

Avoids local Gradle — complements GitHub-hosted `android-unit-tests` by catching
missing assertions before CI runs JVM tests.
"""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
JSON_SRC = ROOT / "docs" / "platform-defaults.json"
KT_TEST = ROOT / "android" / "app" / "src" / "test" / "java" / "com" / "farnam" / "mhrvf" / "PlatformDefaultsContractTest.kt"


def kotlin_accessor(val: object) -> str:
    if isinstance(val, bool):
        return "getBoolean"
    if isinstance(val, int):
        return "getInt"
    return "getString"


def main() -> int:
    spec = json.loads(JSON_SRC.read_text(encoding="utf-8"))
    kt = KT_TEST.read_text(encoding="utf-8")

    shared = spec["shared"]
    parity = spec["parity_shared_defaults"]
    android_raw = spec["android"]
    android = {k: v for k, v in android_raw.items() if not str(k).startswith("rationale_")}

    sections: list[tuple[str, dict[str, object]]] = [
        ("shared", shared),
        ("parity", parity),
        ("android", android),
    ]

    errs: list[str] = []
    min_occurrences = 2  # both JVM tests duplicate the same asserts

    for var, obj in sections:
        for key, val in obj.items():
            acc = kotlin_accessor(val)
            pat = rf'{re.escape(var)}\.{acc}\(\s*"{re.escape(key)}"'
            n = len(re.findall(pat, kt))
            if n == 0:
                errs.append(f"{KT_TEST.name}: missing {var}.{acc}({key!r})")
            elif n < min_occurrences:
                errs.append(
                    f"{KT_TEST.name}: expected ≥{min_occurrences} uses of contract key "
                    + f"{var}.{key!r}, found {n}"
                )

    if errs:
        print("\n".join(errs), file=sys.stderr)
        return 1

    checked = sum(len(o) for _, o in sections)
    print(f"ok platform-defaults JVM test static gate keys checked={checked}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
