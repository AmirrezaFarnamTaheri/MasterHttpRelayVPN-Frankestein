#!/usr/bin/env python3
"""Static guard for readiness IDs, repair targets, and UI consumers.

The generated readiness contract already proves Rust -> Android/docs parity.
This guard closes the frontend loop: Desktop and Android must keep using the
generated IDs/repair anchors instead of hand-written strings or orphaned UI
repair paths.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
READINESS_RS = ROOT / "src" / "readiness.rs"
DESKTOP_UI = ROOT / "src" / "bin" / "ui.rs"
DESKTOP_MODE = ROOT / "src" / "bin" / "ui_mode.rs"
ANDROID_IDS = ROOT / "android/app/src/main/java/com/farnam/mhrvf/ReadinessIds.kt"
ANDROID_HOME = ROOT / "android/app/src/main/java/com/farnam/mhrvf/ui/HomeScreen.kt"
MATRIX = ROOT / "docs" / "readiness-matrix.md"
RUNNER = ROOT / "tools" / "run-repo-sanity.py"


def die(msg: str) -> None:
    print(f"readiness UI contract check failed: {msg}", file=sys.stderr)
    raise SystemExit(1)


def read(path: Path) -> str:
    if not path.is_file():
        die(f"missing file: {path.relative_to(ROOT)}")
    return path.read_text(encoding="utf-8")


def require(text: str, needle: str, label: str) -> None:
    if needle not in text:
        die(f"missing {label}: {needle!r}")


def main() -> int:
    readiness = read(READINESS_RS)
    desktop = read(DESKTOP_UI)
    desktop_mode = read(DESKTOP_MODE)
    android_ids = read(ANDROID_IDS)
    android_home = read(ANDROID_HOME)
    matrix = read(MATRIX)
    runner = read(RUNNER)

    constants = re.findall(
        r'pub const ([A-Z0-9_]+): ReadinessId = "([^"]+)";',
        readiness,
    )
    if not constants:
        die("src/readiness.rs has no ReadinessId constants")

    names = [name for name, _value in constants]
    values = [value for _name, value in constants]
    if len(set(names)) != len(names):
        die("duplicate readiness constant names")
    if len(set(values)) != len(values):
        die("duplicate readiness ID values")

    for marker in (
        "pub type ReadinessId = &'static str;",
        "pub struct ReadinessRepair",
        "pub struct ReadinessRule",
        "pub const READINESS_RULES",
        "pub struct ReadinessRepairAnchor",
        "pub const READINESS_REPAIR_ANCHORS",
        "pub fn readiness_rules()",
        "pub fn repair_for_id(id: ReadinessId)",
        "pub fn repair_anchor_for_target(target: &str)",
        "fn readiness_rule_catalog_is_complete_and_matches_repairs()",
    ):
        require(readiness, marker, f"Rust readiness catalog marker {marker}")

    for name, value in constants:
        require(android_ids, f"const val {name} = \"{value}\"", f"Android generated ID {name}")
        require(matrix, f"| `{value}` |", f"readiness matrix row {value}")

    for marker in (
        "object ReadinessRepairTargets",
        "fun targetForId(id: String): String?",
        "object ReadinessRepairAnchors",
        "fun anchorForTarget(target: String): ReadinessRepairAnchor?",
        "const val ANDROID_CONNECTION_MODE = \"android.connection_mode\"",
    ):
        require(android_ids, marker, f"Android generated repair marker {marker}")

    for marker in (
        "mod ui_mode;",
        "mode_dashboard_panel",
    ):
        require(desktop, marker, f"Desktop readiness owner wiring {marker}")

    for marker in (
        "use mhrv_jni::readiness;",
        "struct ModeReadinessItem",
        "id: readiness::ReadinessId",
        "fn desktop_repair_action",
        "readiness::repair_for_id(item.id)",
        "readiness::repair_anchor_for_target(repair.target)",
        "fn repair_tab_for_target",
        "fn desktop_dashboard_uses_shared_readiness_ids",
        "fn desktop_repair_actions_route_to_expected_tabs",
    ):
        require(desktop_mode, marker, f"Desktop readiness consumer marker {marker}")

    for marker in (
        "import com.farnam.mhrvf.ReadinessIds",
        "import com.farnam.mhrvf.ReadinessRepairAnchors",
        "import com.farnam.mhrvf.ReadinessRepairTargets",
        "private fun androidRepairForId(id: String): AndroidReadinessRepair?",
        "ReadinessRepairTargets.targetForId(id)",
        "ReadinessRepairAnchors.anchorForTarget(target)?.android",
        "data class AndroidReadinessItem",
        "data class AndroidReadinessRepair",
    ):
        require(android_home, marker, f"Android readiness consumer marker {marker}")

    # Require Android's home UI to use the generated IDs for the high-risk
    # user-facing repair families. These are enough to catch reversion to raw
    # string IDs without forcing every validation-only ID to appear in mobile UI.
    android_required_ids = (
        "ACCOUNT_GROUPS_SCRIPT_IDS",
        "ACCOUNT_GROUPS_AUTH_KEY",
        "VERCEL_BASE_URL",
        "VERCEL_RELAY_PATH",
        "VERCEL_AUTH_KEY",
        "DIRECT_GOOGLE_IP",
        "DIRECT_FRONT_DOMAIN",
        "CA_TRUST",
        "ANDROID_APP_CA_TRUST",
        "LAN_EXPOSURE",
        "LAN_TOKEN",
        "LAN_ALLOWLIST",
        "FULL_CODEFULL_DEPLOYMENT",
        "FULL_TUNNEL_NODE_URL",
        "FULL_TUNNEL_AUTH",
        "FULL_UDP_SUPPORT",
        "FULL_TUNNEL_HEALTH",
    )
    for name in android_required_ids:
        require(android_home, f"ReadinessIds.{name}", f"Android Home readiness ID {name}")

    require(
        runner,
        "run_readiness_contract_check(root)",
        "repo-sanity generated readiness contract check",
    )
    require(
        runner,
        "tools/check-readiness-ui-contract.py",
        "repo-sanity readiness UI drift gate",
    )

    print(f"readiness UI contract check: ok ids={len(constants)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
