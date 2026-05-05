#!/usr/bin/env python3
"""Guard Android QR/deep-link config sharing contracts without Gradle."""

from __future__ import annotations

import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
CONFIG_STORE = ROOT / "android/app/src/main/java/com/farnam/mhrvf/ConfigStore.kt"
CONFIG_TEST = ROOT / "android/app/src/test/java/com/farnam/mhrvf/ConfigStoreTest.kt"


def die(message: str) -> None:
    print(f"android config sharing check failed: {message}", file=sys.stderr)
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
    store = read(CONFIG_STORE)
    tests = read(CONFIG_TEST)

    require_all(
        store,
        [
            'private const val HASH_PREFIX = "mhrvf://"',
            'private const val LEGACY_HASH_PREFIX = "mhrv-rs://"',
            "fun encode(cfg: MhrvConfig): String",
            "cfg.preservedUnknownRootJson",
            "JSONObject(cfg.preservedUnknownRootJson)",
            'return "$HASH_PREFIX$b64"',
            "fun decode(encoded: String): MhrvConfig?",
            "trimmed.startsWith(HASH_PREFIX)",
            "trimmed.startsWith(LEGACY_HASH_PREFIX)",
            "java.util.zip.DeflaterOutputStream",
            "java.util.zip.InflaterInputStream",
        ],
        label="ConfigStore.kt",
    )

    require_all(
        tests,
        [
            "fun deepLinkEncodeUsesCurrentSchemeAndPreservesAdvancedRoots",
            "fun deepLinkDecodeAcceptsLegacyScheme",
            "fun decodeRejectsInvalidQrPayload",
            'encoded.startsWith("mhrvf://")',
            'current.replaceFirst("mhrvf://", "mhrv-rs://")',
            'assertNull(ConfigStore.decode("mhrvf://not-valid-base64"))',
            'assertNull(ConfigStore.decode("mhrv-rs://not-valid-base64"))',
            'preservedUnknownRootJson = """{"future_backend":{"enabled":true},"desktop_only":"keep"}"""',
            'saved.has("future_backend")',
            'saved.getString("desktop_only")',
        ],
        label="ConfigStoreTest.kt",
    )

    print("android config sharing check: ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
