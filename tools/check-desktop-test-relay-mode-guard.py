#!/usr/bin/env python3
"""Static Desktop Test Relay mode guard.

The Desktop Test Relay button is a relay-path probe. In `full` and `direct`
modes it must show an explanatory skip message instead of running the relay test
and painting a healthy non-relay mode as a red failure. This check keeps that UI
contract in repo-sanity without needing to launch egui.
"""

from __future__ import annotations

import re
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
UI = ROOT / "src/bin/ui.rs"
DOC = ROOT / "docs/relay-modes.md"


def die(message: str) -> None:
    raise SystemExit(f"desktop test relay mode guard failed: {message}")


def require_contains(text: str, needle: str, label: str) -> None:
    if needle not in text:
        die(f"{label} is missing required marker: {needle!r}")


def extract_test_arm(text: str) -> str:
    start = text.find("Ok(Cmd::Test(cfg)) => {")
    if start == -1:
        die("could not locate Ok(Cmd::Test(cfg)) arm")
    next_arm = text.find("\n            Ok(Cmd::Doctor", start)
    if next_arm == -1:
        die("could not find end of Cmd::Test arm before Cmd::Doctor")
    return text[start:next_arm]


def main() -> int:
    ui = UI.read_text(encoding="utf-8")
    arm = extract_test_arm(ui)

    required = [
        "let mode_explainer = match cfg.mode_kind().ok()",
        "Some(mhrv_jni::config::Mode::Full)",
        "Some(mhrv_jni::config::Mode::Direct)",
        "Test Relay is wired only for apps_script mode",
        "full mode the data plane is tunnel-node",
        "direct mode there is no Apps Script relay",
        "st.last_test_ok = None",
        "st.last_test_msg = msg.into()",
        'push_log(&shared, &format!("[ui] test skipped: {}", msg))',
        "continue;",
        "let ok = test_cmd::run(&cfg).await",
    ]
    for marker in required:
        require_contains(arm, marker, "src/bin/ui.rs Cmd::Test arm")

    skip_idx = arm.find('push_log(&shared, &format!("[ui] test skipped: {}", msg))')
    continue_idx = arm.find("continue;", skip_idx)
    run_idx = arm.find("let ok = test_cmd::run(&cfg).await")
    if not (skip_idx < continue_idx < run_idx):
        die("skip branch must log, continue, and only then leave test_cmd::run in the apps_script path")

    if re.search(r"Some\(mhrv_jni::config::Mode::(Full|Direct)\).*last_test_ok\s*=\s*Some\(false\)", arm, re.S):
        die("full/direct Test Relay skip must not set last_test_ok = Some(false)")

    doc = re.sub(r"\s+", " ", DOC.read_text(encoding="utf-8"))
    for marker in [
        "The desktop **Test Relay** action is an Apps Script / serverless JSON probe.",
        "does not run in `full` mode",
        "does not run in `direct` mode",
        "https://whatismyipaddress.com",
    ]:
        require_contains(doc, marker, "docs/relay-modes.md")

    print("desktop test relay mode guard: ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
