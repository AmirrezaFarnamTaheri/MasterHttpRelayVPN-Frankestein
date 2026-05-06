#!/usr/bin/env python3
"""Guard the Android support-snapshot redaction owner.

This is a static CI/local drift gate. It deliberately does not run Gradle:
the Android JVM tests are the deeper executable contract, while this script
keeps repo-sanity fast and available on machines without Android tooling.
"""

from __future__ import annotations

import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
SUPPORT_REDACTION = ROOT / "android/app/src/main/java/com/farnam/mhrvf/SupportRedaction.kt"
HOME_SCREEN = ROOT / "android/app/src/main/java/com/farnam/mhrvf/ui/HomeScreen.kt"
SUPPORT_TEST = ROOT / "android/app/src/test/java/com/farnam/mhrvf/SupportRedactionTest.kt"


def die(message: str) -> None:
    print(f"android support redaction check failed: {message}", file=sys.stderr)
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


def forbid_all(text: str, needles: list[str], *, label: str) -> None:
    found = [needle for needle in needles if needle in text]
    if found:
        formatted = ", ".join(repr(needle) for needle in found)
        die(f"{label} contains forbidden stale owner marker(s): {formatted}")


def main() -> int:
    support = read(SUPPORT_REDACTION)
    home = read(HOME_SCREEN)
    tests = read(SUPPORT_TEST)

    require_all(
        support,
        [
            "fun maskDeploymentId",
            "fun androidSupportSnapshot",
            "doctorJson: String? = null",
            "schema: android-support-snapshot/v2",
            "auth_key, serverless AUTH_KEY, LAN token, upstream SOCKS5, and raw unknown JSON are not included.",
            "deployment IDs are masked.",
            "Doctor details/fixes are summarized by item id only",
            "apps_script_deployments_masked",
            "apps_script_auth_key_configured",
            "serverless_auth_key_configured",
            "lan_token_configured",
            "upstream_socks5_configured",
            "unknown_root_fields_preserved",
            "doctor_available",
            "doctor_items_fail",
            "doctor_problem_ids",
        ],
        label="SupportRedaction.kt",
    )

    require_all(
        home,
        [
            "import com.farnam.mhrvf.androidSupportSnapshot",
            "doctorJsonForSupport",
            "doctorJsonForSupport = null",
            "doctorJsonForSupport = json.takeIf { it.isNotBlank() }",
            "androidSupportSnapshot(cfg, caInstalled, doctorJsonForSupport)",
        ],
        label="HomeScreen.kt",
    )
    forbid_all(
        home,
        [
            "private fun maskedDeploymentId",
            "fun maskedDeploymentId",
            "private fun androidSupportSnapshot",
            "private fun yesNo",
        ],
        label="HomeScreen.kt",
    )

    require_all(
        tests,
        [
            "fun maskDeploymentIdNormalizesUrlsAndKeepsOnlyPrefixAndSuffix",
            "fun androidSupportSnapshotOmitsSecretsAndMasksDeploymentIds",
            "fun androidSupportSnapshotIncludesDoctorSummaryWithoutDetails",
            "doctor_available: yes",
            "doctor_problem_ids: apps_script_urls, auth_key",
            'assertFalse(snapshot.contains("AKfycb1234567890abcdef"))',
            'assertFalse(snapshot.contains("android-secret"))',
            'assertFalse(snapshot.contains("serverless-secret"))',
            'assertFalse(snapshot.contains("lan-secret-token"))',
            'assertFalse(snapshot.contains("user:pass@example.com"))',
            'assertFalse(snapshot.contains("https://secret.example/path"))',
            'assertFalse(snapshot.contains("""{"raw":"secret"}"""))',
        ],
        label="SupportRedactionTest.kt",
    )

    print("android support redaction check: ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
