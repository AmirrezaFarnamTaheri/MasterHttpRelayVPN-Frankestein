#!/usr/bin/env python3
"""Run the same checks as the GitHub Actions `repo-sanity` job (locally).

Requires Python on PATH and Node.js for syntax/helper-test steps. Readiness uses
`pwsh` when available, otherwise Windows `powershell`.

When editing `.github/workflows/ci.yml` repo-sanity steps or adding drift gates,
update this script (and vice versa) so CI and local stay aligned.
Includes **`report-nova-proxy-config.py --demo`** after repository cleanliness.
"""

from __future__ import annotations

import argparse
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path


def die(msg: str, code: int = 1) -> None:
    print(msg, file=sys.stderr)
    raise SystemExit(code)


def run(cmd: list[str], *, cwd: Path, label: str) -> None:
    print(f"\n--- {label} ---", flush=True)
    print("$", " ".join(cmd), flush=True)
    r = subprocess.run(cmd, cwd=cwd)
    if r.returncode != 0:
        die(f"{label} failed (exit {r.returncode})", r.returncode)


def node_syntax_tools(root: Path, node: str) -> None:
    js_files = sorted((root / "tools").rglob("*.js"))
    for path in js_files:
        run([node, "--check", str(path)], cwd=root, label=f"node --check {path.relative_to(root)}")


def node_syntax_apps_script(root: Path, node: str) -> None:
    gs_files = sorted((root / "assets" / "apps_script").rglob("*.gs"))
    if not gs_files:
        return
    with tempfile.TemporaryDirectory() as tmp:
        tmp_path = Path(tmp)
        for gs in gs_files:
            out = tmp_path / (gs.stem + ".js")
            out.write_bytes(gs.read_bytes())
            run([node, "--check", str(out)], cwd=root, label=f"node --check {gs.relative_to(root)}")


def node_apps_script_tests(root: Path, node: str) -> None:
    test_dir = root / "assets" / "apps_script" / "tests"
    tests = sorted(test_dir.glob("*.js"))
    for test in tests:
        run([node, str(test)], cwd=root, label=f"apps_script test {test.name}")


def run_readiness_contract_check(root: Path) -> None:
    script = root / "tools" / "generate-readiness-contract.ps1"
    candidates = [
        ["pwsh", "-NoProfile", "-File", str(script), "-Check"],
        [
            "powershell",
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-File",
            str(script),
            "-Check",
        ],
    ]
    for cmd in candidates:
        if shutil.which(cmd[0]):
            run(cmd, cwd=root, label="readiness contract (-Check)")
            return
    die(
        "Readiness contract check needs pwsh or powershell on PATH "
        "(install PowerShell 7, or use Windows PowerShell)."
    )


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--skip-node",
        action="store_true",
        help="Skip JavaScript/GS syntax and Apps Script helper tests (Python gates only).",
    )
    parser.add_argument(
        "--skip-readiness",
        action="store_true",
        help="Skip generate-readiness-contract.ps1 -Check.",
    )
    args = parser.parse_args()

    root = Path(__file__).resolve().parent.parent
    py = sys.executable

    if not args.skip_node:
        node = shutil.which("node")
        if not node:
            die("node not found on PATH (use --skip-node for Python-only checks)")
        node_syntax_tools(root, node)
        node_syntax_apps_script(root, node)
        node_apps_script_tests(root, node)

    python_steps = [
        ("repository cleanliness", [py, "tools/check-repo-cleanliness.py"]),
        ("Nova donor settings demo report", [py, "tools/report-nova-proxy-config.py", "--demo"]),
        ("markdown local links", [py, "tools/check-doc-links.py"]),
        ("markdown heading anchors", [py, "tools/check-doc-anchors.py"]),
        ("changelog H1 headings", [py, "tools/check-changelog-headings.py"]),
        ("generated changelog index (-Check)", [py, "tools/generate-changelog-index.py", "-Check"]),
        ("release/changelog governance", [py, "tools/check-release-governance.py"]),
        ("contributor/security repo governance", [py, "tools/check-repo-governance.py"]),
        ("ADR governance", [py, "tools/check-adr-governance.py"]),
        ("verification profiles", [py, "tools/check-verification-profiles.py"]),
        ("change-impact checklist", [py, "tools/check-change-impact-checklist.py"]),
        ("tooling source map", [py, "tools/check-tooling-source-map.py"]),
        ("android config key drift", [py, "tools/check-android-config-keys.py"]),
        ("Android ownedKeys list vs registry", [py, "tools/check-android-owned-keys-list.py"]),
        ("Android string resource parity", [py, "tools/check-android-string-resource-parity.py"]),
        (
            "Android hard-coded copy inventory (-Check)",
            [py, "tools/generate-android-hardcoded-copy-inventory.py", "-Check"],
        ),
        ("Android QR/deep-link config sharing", [py, "tools/check-android-config-sharing.py"]),
        ("Android VPN lifecycle/teardown guard", [py, "tools/check-android-vpn-lifecycle.py"]),
        ("Apps Script relay hardening", [py, "tools/check-apps-script-hardening.py"]),
        ("Cloudflare Worker relay bridge", [py, "tools/check-cloudflare-worker-relay.py"]),
        ("Desktop LAN sharing UI guard", [py, "tools/check-lan-sharing-ui.py"]),
        ("fronting-groups starter example", [py, "tools/check-fronting-groups-example.py"]),
        ("full-mode coalesce tuning", [py, "tools/check-coalesce-tuning.py"]),
        ("Rust/Android DEFAULT_GOOGLE_SNI_POOL parity", [py, "tools/check-sni-default-pool.py"]),
        ("platform defaults vs Rust/Kotlin", [py, "tools/check-platform-defaults.py"]),
        ("generated platform defaults doc (-Check)", [py, "tools/generate-platform-defaults-doc.py", "-Check"]),
        (
            "Android platform-defaults JVM test (static)",
            [py, "tools/check-android-platform-defaults-test-static.py"],
        ),
        ("Android support redaction owner", [py, "tools/check-android-support-redaction.py"]),
        ("Android support snapshot schema", [py, "tools/check-android-support-snapshot-schema.py"]),
        ("status/stats JSON contract", [py, "tools/check-status-stats-json-contract.py"]),
        ("Doctor JSON contract", [py, "tools/check-doctor-json-contract.py"]),
        ("Android Doctor JNI bridge", [py, "tools/check-android-doctor-jni-bridge.py"]),
        ("Android Doctor summary UI", [py, "tools/check-android-doctor-summary-ui.py"]),
        ("Desktop Doctor summary card", [py, "tools/check-desktop-doctor-summary.py"]),
        ("Desktop Test Relay mode guard", [py, "tools/check-desktop-test-relay-mode-guard.py"]),
        ("Desktop UI modularization guard", [py, "tools/check-desktop-ui-modularization.py"]),
        ("canonical relay-mode vocabulary", [py, "tools/check-mode-vocabulary.py"]),
        ("mode example fixtures", [py, "tools/check-mode-example-fixtures.py"]),
        ("readiness UI contract", [py, "tools/check-readiness-ui-contract.py"]),
        ("tunnel-node drain/concurrency guard", [py, "tools/check-tunnel-node-drain-concurrency.py"]),
        ("CI/local sanity parity", [py, "tools/check-ci-local-sanity-parity.py"]),
        ("Telegram release notify renderer", [py, "tools/check-telegram-release-notify.py"]),
        ("README Persian guide links", [py, "tools/check-readme-persian-guides.py"]),
        ("generated config registry docs (-Check)", [py, "tools/generate-config-registry.py", "-Check"]),
        ("config registry nested_fields vs Rust structs", [py, "tools/check-config-registry-nested-fields.py"]),
        ("config registry map fields value_semantics", [py, "tools/check-config-registry-map-semantics.py"]),
        ("Desktop ConfigWire vs config-registry roots", [py, "tools/check-config-wire-vs-registry.py"]),
        ("generated parity matrix docs (-Check)", [py, "tools/generate-parity-matrix.py", "-Check"]),
        ("parity matrix references", [py, "tools/check-parity-matrix.py"]),
    ]
    for label, cmd in python_steps:
        run(cmd, cwd=root, label=label)

    if not args.skip_readiness:
        run_readiness_contract_check(root)

    run(
        [py, "tools/check-json-xml-android-stale.py"],
        cwd=root,
        label="JSON/XML/Android strings/stale scan",
    )

    print("\nrepo-sanity (local): ok", flush=True)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
