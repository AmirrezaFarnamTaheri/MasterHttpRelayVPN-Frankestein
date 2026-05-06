#!/usr/bin/env python3
"""Static guard for CI/local verification parity.

The repository deliberately keeps the broad drift checks in one local script:
`tools/run-repo-sanity.py`. CI should call that script instead of re-declaring a
second hand-maintained list of Python/Node checks. This guard fails if the CI
workflow stops using the local runner or drops the release-critical Rust,
tunnel-node, or Android JVM-test gates.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
CI = ROOT / ".github" / "workflows" / "ci.yml"
LOCAL = ROOT / "tools" / "run-repo-sanity.py"


def die(msg: str) -> None:
    print(f"CI/local sanity parity check failed: {msg}", file=sys.stderr)
    raise SystemExit(1)


def require(text: str, needle: str, label: str) -> None:
    if needle not in text:
        die(f"missing {label}: {needle!r}")


def require_regex(text: str, pattern: str, label: str) -> None:
    if not re.search(pattern, text, flags=re.S):
        die(f"missing {label}: /{pattern}/")


def main() -> int:
    if not CI.is_file():
        die(f"missing workflow: {CI}")
    if not LOCAL.is_file():
        die(f"missing local runner: {LOCAL}")

    ci = CI.read_text(encoding="utf-8")
    local = LOCAL.read_text(encoding="utf-8")

    require(ci, "repo-sanity:", "repo-sanity job")
    require(ci, "actions/setup-node@v4", "Node setup for JS/Apps Script checks")
    require(ci, "python3 tools/run-repo-sanity.py", "CI call to local repo-sanity runner")
    require(
        ci,
        "Keep checks in sync with `tools/run-repo-sanity.py`",
        "single-source-of-truth workflow comment",
    )

    # Release-blocking Rust/backend checks. Keep these outside repo-sanity
    # because they compile/test the Rust crates and need system packages.
    require(ci, "cargo fmt --check", "root rustfmt")
    require(ci, "cargo clippy --all-targets --all-features -- -D warnings", "root clippy all features")
    require(ci, "cargo test --all-targets --features ui", "root UI-feature tests")
    require_regex(
        ci,
        r"name:\s*clippy \(tunnel-node\).*?working-directory:\s*tunnel-node.*?cargo clippy --all-targets -- -D warnings",
        "tunnel-node clippy step",
    )
    require_regex(
        ci,
        r"name:\s*test \(tunnel-node\).*?working-directory:\s*tunnel-node.*?cargo test --all-targets",
        "tunnel-node test step",
    )

    # Android JVM parity tests are intentionally CI/pre-provisioned only. The
    # local script carries static gates so maintainers do not need Gradle.
    require(ci, "android-unit-tests:", "Android JVM test job")
    require(ci, "gradle/actions/setup-gradle@v4", "Gradle setup in CI")
    require(ci, "./gradlew :app:testDebugUnitTest", "Android JVM unit test command")
    require(ci, "--tests com.farnam.mhrvf.PlatformDefaultsContractTest", "platform defaults JVM test")
    require(ci, "-x cargoBuildDebug", "Android CI skips local Rust cargo-ndk debug task")
    require(ci, "-x cargoBuildRelease", "Android CI skips local Rust cargo-ndk release task")

    # The local runner must carry the recently added high-risk drift gates, so
    # CI inherits them through the single call above.
    local_required = [
        "tools/check-android-vpn-lifecycle.py",
        "tools/check-changelog-headings.py",
        "tools/generate-changelog-index.py",
        "tools/check-release-governance.py",
        "tools/check-repo-governance.py",
        "tools/check-adr-governance.py",
        "tools/check-verification-profiles.py",
        "tools/check-change-impact-checklist.py",
        "tools/check-tooling-source-map.py",
        "tools/check-android-string-resource-parity.py",
        "tools/generate-android-hardcoded-copy-inventory.py",
        "tools/check-apps-script-hardening.py",
        "tools/check-cloudflare-worker-relay.py",
        "tools/check-lan-sharing-ui.py",
        "tools/check-fronting-groups-example.py",
        "tools/check-coalesce-tuning.py",
        "tools/check-status-stats-json-contract.py",
        "tools/check-doctor-json-contract.py",
        "tools/check-android-doctor-jni-bridge.py",
        "tools/check-android-doctor-summary-ui.py",
        "tools/check-android-support-snapshot-schema.py",
        "tools/check-desktop-doctor-summary.py",
        "tools/check-desktop-test-relay-mode-guard.py",
        "tools/check-desktop-ui-modularization.py",
        "tools/check-mode-vocabulary.py",
        "tools/check-mode-example-fixtures.py",
        "tools/check-readiness-ui-contract.py",
        "tools/check-tunnel-node-drain-concurrency.py",
        "tools/check-telegram-release-notify.py",
        "tools/check-readme-persian-guides.py",
        "tools/check-platform-defaults.py",
        "tools/generate-platform-defaults-doc.py",
        "tools/generate-config-registry.py",
        "tools/generate-parity-matrix.py",
        "tools/check-json-xml-android-stale.py",
    ]
    for needle in local_required:
        require(local, needle, f"local repo-sanity step {needle}")

    print("CI/local sanity parity check: ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
