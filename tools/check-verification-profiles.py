#!/usr/bin/env python3
"""Guard verification profile docs and machine-readable contract."""

from __future__ import annotations

import json
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
CONTRACT = ROOT / "docs" / "verification-profiles.json"
DOC = ROOT / "docs" / "verification-profiles.md"
DOCS_INDEX = ROOT / "docs" / "index.md"
CONTRIBUTING = ROOT / "CONTRIBUTING.md"
TOOLS_README = ROOT / "tools" / "README.md"
RELEASE_CHECKLIST = ROOT / "docs" / "release-checklist.md"
SANITY = ROOT / "tools" / "run-repo-sanity.py"
PARITY = ROOT / "tools" / "check-ci-local-sanity-parity.py"

REQUIRED_IDS = [
    "docs_governance",
    "config_schema",
    "android_ui",
    "backend_helpers",
    "desktop_runtime",
    "full_tunnel",
    "release_ready",
]

REQUIRED_COMMAND_MARKERS = {
    "docs_governance": [
        "python tools/check-doc-links.py",
        "python tools/check-adr-governance.py",
        "python tools/check-verification-profiles.py",
    ],
    "config_schema": [
        "python tools/generate-config-registry.py -Check",
        "python tools/check-android-config-keys.py",
        "cargo test bundled_example_configs_load_and_validate",
    ],
    "android_ui": [
        "python tools/check-android-string-resource-parity.py",
        "python tools/check-android-vpn-lifecycle.py",
        "python tools/check-android-support-snapshot-schema.py",
    ],
    "backend_helpers": [
        "python tools/check-apps-script-hardening.py",
        "python tools/check-cloudflare-worker-relay.py",
        "node assets/apps_script/tests/compat_marker_test.js",
    ],
    "desktop_runtime": [
        "cargo fmt --check",
        "cargo test --all-targets --features ui",
        "python tools/check-readiness-ui-contract.py",
    ],
    "full_tunnel": [
        "python tools/check-tunnel-node-drain-concurrency.py",
        "cd tunnel-node; cargo test --all-targets",
    ],
    "release_ready": [
        "python tools/run-repo-sanity.py",
        "cargo clippy --all-targets --all-features -- -D warnings",
        "python tools/check-ci-local-sanity-parity.py",
    ],
}


def die(message: str) -> None:
    print(f"verification profiles check failed: {message}", file=sys.stderr)
    raise SystemExit(1)


def read(path: Path) -> str:
    if not path.is_file():
        die(f"missing required file: {path.relative_to(ROOT)}")
    return path.read_text(encoding="utf-8")


def require(text: str, needle: str, label: str) -> None:
    if needle not in text:
        die(f"{label} missing {needle!r}")


def main() -> int:
    if not CONTRACT.is_file():
        die("missing docs/verification-profiles.json")
    try:
        data = json.loads(CONTRACT.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        die(f"invalid docs/verification-profiles.json: {exc}")

    if data.get("schema") != "mhrv-f-verification-profiles/v1":
        die("unexpected verification profile schema")

    profiles = data.get("profiles")
    if not isinstance(profiles, list):
        die("profiles must be a list")

    by_id = {}
    for profile in profiles:
        if not isinstance(profile, dict):
            die("each profile must be an object")
        profile_id = profile.get("id")
        if profile_id in by_id:
            die(f"duplicate profile id: {profile_id}")
        by_id[profile_id] = profile
        for key in ["id", "title", "use_when", "commands", "notes"]:
            if key not in profile:
                die(f"profile {profile_id!r} missing {key!r}")
        if not isinstance(profile["commands"], list) or not profile["commands"]:
            die(f"profile {profile_id!r} must have commands")
        if not isinstance(profile["notes"], list):
            die(f"profile {profile_id!r} notes must be a list")

    if list(by_id) != REQUIRED_IDS:
        die(f"profile ids/order mismatch: got {list(by_id)!r}")

    for profile_id, markers in REQUIRED_COMMAND_MARKERS.items():
        commands = "\n".join(by_id[profile_id]["commands"])
        for marker in markers:
            require(commands, marker, f"profile {profile_id}")

    doc = read(DOC)
    for profile_id in REQUIRED_IDS:
        require(doc, f"`{profile_id}`", "docs/verification-profiles.md")
        require(doc, by_id[profile_id]["title"], "docs/verification-profiles.md")
    require(doc, "docs/verification-profiles.json", "docs/verification-profiles.md")
    require(doc, "tools/check-verification-profiles.py", "docs/verification-profiles.md")

    docs_index = read(DOCS_INDEX)
    require(docs_index, "verification-profiles.md", "docs/index.md")

    contributing = read(CONTRIBUTING)
    require(contributing, "docs/verification-profiles.md", "CONTRIBUTING.md")
    require(contributing, "python tools\\check-verification-profiles.py", "CONTRIBUTING.md")

    tools_readme = read(TOOLS_README)
    require(tools_readme, "Verification profiles guard", "tools/README.md")
    require(tools_readme, "check-verification-profiles.py", "tools/README.md")

    release_checklist = read(RELEASE_CHECKLIST)
    require(release_checklist, "docs/verification-profiles.md", "docs/release-checklist.md")

    sanity = read(SANITY)
    require(sanity, "tools/check-verification-profiles.py", "tools/run-repo-sanity.py")
    require(sanity, "verification profiles", "tools/run-repo-sanity.py")

    parity = read(PARITY)
    require(parity, "tools/check-verification-profiles.py", "tools/check-ci-local-sanity-parity.py")

    print("verification profiles check: ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
