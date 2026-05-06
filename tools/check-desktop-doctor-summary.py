#!/usr/bin/env python3
"""Guard the Desktop Monitor's structured Doctor summary card.

The Doctor JSON contract gives future bridges one shared diagnostic shape. The
Desktop Monitor should not regress to log-only Doctor output: it must keep the
latest typed DoctorReport in UI state and render a compact summary card. The
summary renderer itself lives in src/bin/ui_doctor.rs so the large desktop UI
file does not become the second source of truth for Doctor presentation.
"""

from __future__ import annotations

import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
UI = ROOT / "src" / "bin" / "ui.rs"
UI_DOCTOR = ROOT / "src" / "bin" / "ui_doctor.rs"


def die(msg: str) -> None:
    print(f"Desktop Doctor summary guard failed: {msg}", file=sys.stderr)
    raise SystemExit(1)


def require(text: str, needle: str, label: str) -> None:
    if needle not in text:
        die(f"missing {label}: {needle!r}")


def main() -> int:
    if not UI.is_file():
        die(f"missing UI source: {UI}")
    if not UI_DOCTOR.is_file():
        die(f"missing Doctor UI module: {UI_DOCTOR}")
    ui = UI.read_text(encoding="utf-8")
    ui_doctor = UI_DOCTOR.read_text(encoding="utf-8")

    require(
        ui,
        "last_doctor_report: Option<doctor::DoctorReport>",
        "typed DoctorReport state",
    )
    require(ui, "last_doctor_at: Option<Instant>", "Doctor timestamp state")
    require(
        ui,
        "use ui_doctor::{doctor_level_label, render_doctor_summary_card};",
        "Doctor summary module import",
    )
    require(
        ui_doctor,
        "pub(crate) fn doctor_level_label(",
        "shared Desktop Doctor level labels",
    )
    require(ui_doctor, "fn doctor_counts(", "Doctor severity count helper")
    require(
        ui_doctor,
        "pub(crate) fn render_doctor_summary_card(",
        "Monitor Doctor summary renderer",
    )
    require(
        ui,
        "render_doctor_summary_card(ui, doctor_report.as_ref(), doctor_at)",
        "Monitor tab renders the Doctor summary",
    )
    require(
        ui,
        "st.last_doctor_report = Some(report.clone())",
        "plain Doctor command stores report",
    )
    require(
        ui,
        "st.last_doctor_report = Some(after.clone())",
        "Doctor+Fix command stores post-fix report",
    )
    require(
        ui,
        "for it in &after.items",
        "Doctor+Fix preserves after report while logging",
    )

    print("Desktop Doctor summary guard: ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
