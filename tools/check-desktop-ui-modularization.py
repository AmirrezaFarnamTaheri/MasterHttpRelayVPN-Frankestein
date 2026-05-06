#!/usr/bin/env python3
"""Guard the first Desktop UI modularization boundary."""

from __future__ import annotations

import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
UI = ROOT / "src" / "bin" / "ui.rs"
DOCTOR = ROOT / "src" / "bin" / "ui_doctor.rs"
FORMAT = ROOT / "src" / "bin" / "ui_format.rs"
FS = ROOT / "src" / "bin" / "ui_fs.rs"
HELP = ROOT / "src" / "bin" / "ui_help.rs"
MODE = ROOT / "src" / "bin" / "ui_mode.rs"
MONITOR = ROOT / "src" / "bin" / "ui_monitor.rs"
SETUP = ROOT / "src" / "bin" / "ui_setup.rs"
STYLE = ROOT / "src" / "bin" / "ui_style.rs"
TRUST = ROOT / "src" / "bin" / "ui_trust.rs"
XHTTP = ROOT / "src" / "bin" / "ui_xhttp.rs"
TOOLS_README = ROOT / "tools" / "README.md"
SOURCE_MAP = ROOT / "docs" / "tooling-source-map.json"
SANITY = ROOT / "tools" / "run-repo-sanity.py"
PARITY = ROOT / "tools" / "check-ci-local-sanity-parity.py"


def die(message: str) -> None:
    print(f"desktop UI modularization check failed: {message}", file=sys.stderr)
    raise SystemExit(1)


def read(path: Path) -> str:
    if not path.is_file():
        die(f"missing required file: {path.relative_to(ROOT)}")
    return path.read_text(encoding="utf-8")


def require(text: str, needle: str, label: str) -> None:
    if needle not in text:
        die(f"{label} missing {needle!r}")


def reject(text: str, needle: str, label: str) -> None:
    if needle in text:
        die(f"{label} must not contain {needle!r}")


def main() -> int:
    ui = read(UI)
    doctor = read(DOCTOR)
    fmt = read(FORMAT)
    fs = read(FS)
    help_ui = read(HELP)
    mode = read(MODE)
    monitor = read(MONITOR)
    setup = read(SETUP)
    style = read(STYLE)
    trust = read(TRUST)
    xhttp = read(XHTTP)
    tools = read(TOOLS_README)
    source_map = read(SOURCE_MAP)
    sanity = read(SANITY)
    parity = read(PARITY)

    require(ui, "mod ui_doctor;", "src/bin/ui.rs")
    require(ui, "mod ui_format;", "src/bin/ui.rs")
    require(ui, "mod ui_fs;", "src/bin/ui.rs")
    require(ui, "mod ui_help;", "src/bin/ui.rs")
    require(ui, "mod ui_mode;", "src/bin/ui.rs")
    require(ui, "mod ui_monitor;", "src/bin/ui.rs")
    require(ui, "mod ui_setup;", "src/bin/ui.rs")
    require(ui, "mod ui_style;", "src/bin/ui.rs")
    require(ui, "mod ui_trust;", "src/bin/ui.rs")
    require(ui, "mod ui_xhttp;", "src/bin/ui.rs")
    require(
        ui,
        "use ui_doctor::{doctor_level_label, render_doctor_summary_card};",
        "src/bin/ui.rs",
    )
    require(ui, "use ui_format::{fmt_bytes, fmt_duration};", "src/bin/ui.rs")
    require(
        ui,
        "use ui_fs::{downloads_dir, reveal_in_file_manager};",
        "src/bin/ui.rs",
    )
    require(
        ui,
        "use ui_help::{backend_tool_entries, help_walkthrough, render_tool_help_row};",
        "src/bin/ui.rs",
    )
    require(
        ui,
        "use ui_mode::{ghost_action, info_chip, mode_dashboard_panel, mode_summary, mode_summary_panel};",
        "src/bin/ui.rs",
    )
    require(
        ui,
        "use ui_monitor::{",
        "src/bin/ui.rs",
    )
    require(ui, "traffic_stat_rows", "src/bin/ui.rs")
    require(ui, "quota_calls_per_hour", "src/bin/ui.rs")
    require(ui, "degradation_changes", "src/bin/ui.rs")
    require(ui, "notable_failure_lines", "src/bin/ui.rs")
    require(ui, "use ui_setup::show_first_run_wizard;", "src/bin/ui.rs")
    require(ui, "apply_ui_theme, form_row, help_callout", "src/bin/ui.rs")
    require(ui, "primary_button", "src/bin/ui.rs")
    require(ui, "section", "src/bin/ui.rs")
    require(ui, "ACCENT", "src/bin/ui.rs")
    require(ui, "use ui_trust::trust_center_tab;", "src/bin/ui.rs")
    require(ui, "poll_xhttp_cloud_deploy, xhttp_vless_generator", "src/bin/ui.rs")
    require(ui, "XhttpDeployPipe, XhttpGeneratorForm", "src/bin/ui.rs")
    reject(ui, "fn fmt_duration(", "src/bin/ui.rs")
    reject(ui, "fn fmt_bytes(", "src/bin/ui.rs")
    reject(ui, "fn open_local_resource(", "src/bin/ui.rs")
    reject(ui, "fn reveal_in_file_manager(", "src/bin/ui.rs")
    reject(ui, "fn downloads_dir(", "src/bin/ui.rs")
    reject(ui, "fn tool_help_row(", "src/bin/ui.rs")
    reject(ui, "fn help_walkthrough(", "src/bin/ui.rs")
    reject(ui, "fn mode_summary(", "src/bin/ui.rs")
    reject(ui, "fn mode_summary_panel(", "src/bin/ui.rs")
    reject(ui, "struct ModeReadinessItem", "src/bin/ui.rs")
    reject(ui, "struct ModeDashboard", "src/bin/ui.rs")
    reject(ui, "fn mode_dashboard(", "src/bin/ui.rs")
    reject(ui, "fn desktop_repair_action(", "src/bin/ui.rs")
    reject(ui, "fn mode_dashboard_panel(", "src/bin/ui.rs")
    reject(ui, "fn info_chip(", "src/bin/ui.rs")
    reject(ui, "fn ghost_action(", "src/bin/ui.rs")
    reject(ui, "fn traffic_stat_rows(", "src/bin/ui.rs")
    reject(ui, "fn quota_calls_per_hour(", "src/bin/ui.rs")
    reject(ui, "fn degradation_changes(", "src/bin/ui.rs")
    reject(ui, "fn notable_failure_lines(", "src/bin/ui.rs")
    reject(ui, "fn show_first_run_wizard(", "src/bin/ui.rs")
    reject(ui, "fn doctor_level_label(", "src/bin/ui.rs")
    reject(ui, "fn doctor_level_color(", "src/bin/ui.rs")
    reject(ui, "fn doctor_counts(", "src/bin/ui.rs")
    reject(ui, "fn render_doctor_summary_card(", "src/bin/ui.rs")
    reject(ui, "const ACCENT:", "src/bin/ui.rs")
    reject(ui, "const TEXT_LABEL:", "src/bin/ui.rs")
    reject(ui, "const TEXT_MUTED:", "src/bin/ui.rs")
    reject(ui, "const CARD_FILL:", "src/bin/ui.rs")
    reject(ui, "const CARD_STROKE:", "src/bin/ui.rs")
    reject(ui, "fn apply_ui_theme(", "src/bin/ui.rs")
    reject(ui, "fn section_title_bar(", "src/bin/ui.rs")
    reject(ui, "fn section(", "src/bin/ui.rs")
    reject(ui, "fn help_subheading(", "src/bin/ui.rs")
    reject(ui, "fn help_muted(", "src/bin/ui.rs")
    reject(ui, "fn help_callout(", "src/bin/ui.rs")
    reject(ui, "fn mode_goal_card(", "src/bin/ui.rs")
    reject(ui, "fn primary_button(", "src/bin/ui.rs")
    reject(ui, "fn form_row(", "src/bin/ui.rs")
    reject(ui, "fn trust_status_label(", "src/bin/ui.rs")
    reject(ui, "fn bool_status(", "src/bin/ui.rs")
    reject(ui, "fn trust_center_tab(", "src/bin/ui.rs")
    reject(ui, "fn trust_center_snapshot_panel(", "src/bin/ui.rs")
    reject(ui, "fn support_bundle_preview(", "src/bin/ui.rs")
    reject(ui, "support_bundle::preview_manifest", "src/bin/ui.rs")
    reject(ui, "trust_center::snapshot", "src/bin/ui.rs")
    reject(ui, "TrustStatus", "src/bin/ui.rs")
    reject(ui, "const NETLIFY_XHTTP_CANDIDATES", "src/bin/ui.rs")
    reject(ui, "const VERCEL_XHTTP_CANDIDATES", "src/bin/ui.rs")
    reject(ui, "struct XhttpGeneratorForm", "src/bin/ui.rs")
    reject(ui, "impl Default for XhttpGeneratorForm", "src/bin/ui.rs")
    reject(ui, "fn xhttp_platform_defaults(", "src/bin/ui.rs")
    reject(ui, "fn normalize_xhttp_host(", "src/bin/ui.rs")
    reject(ui, "fn normalize_xhttp_path(", "src/bin/ui.rs")
    reject(ui, "fn encode_uri_component(", "src/bin/ui.rs")
    reject(ui, "fn generate_xhttp_vless_links(", "src/bin/ui.rs")
    reject(ui, "fn generate_xhttp_deploy_notes(", "src/bin/ui.rs")
    reject(ui, "struct XhttpDeployPipe", "src/bin/ui.rs")
    reject(ui, "fn xhttp_vless_generator(", "src/bin/ui.rs")
    reject(ui, "fn poll_xhttp_cloud_deploy(", "src/bin/ui.rs")
    reject(ui, "xhttp_cloud_deploy::", "src/bin/ui.rs")
    reject(ui, "XhttpDeployWorkerMsg", "src/bin/ui.rs")

    require(doctor, "pub(crate) fn doctor_level_label", "src/bin/ui_doctor.rs")
    require(
        doctor,
        "pub(crate) fn render_doctor_summary_card",
        "src/bin/ui_doctor.rs",
    )
    require(doctor, "doctor_counts_groups_levels", "src/bin/ui_doctor.rs")
    require(doctor, "doctor_level_labels_are_stable", "src/bin/ui_doctor.rs")

    require(fmt, "pub(crate) fn fmt_duration", "src/bin/ui_format.rs")
    require(fmt, "pub(crate) fn fmt_bytes", "src/bin/ui_format.rs")
    require(fmt, "duration_is_hh_mm_ss", "src/bin/ui_format.rs")
    require(fmt, "bytes_use_existing_units_and_precision", "src/bin/ui_format.rs")

    require(fs, "pub(crate) fn downloads_dir", "src/bin/ui_fs.rs")
    require(fs, "pub(crate) fn reveal_in_file_manager", "src/bin/ui_fs.rs")
    require(fs, "pub(crate) fn open_local_resource", "src/bin/ui_fs.rs")
    require(fs, "missing_relative_resource_returns_none", "src/bin/ui_fs.rs")

    require(help_ui, "pub(crate) struct ToolHelpEntry", "src/bin/ui_help.rs")
    require(help_ui, "pub(crate) fn backend_tool_entries", "src/bin/ui_help.rs")
    require(help_ui, "pub(crate) fn render_tool_help_row", "src/bin/ui_help.rs")
    require(help_ui, "pub(crate) fn help_walkthrough", "src/bin/ui_help.rs")
    require(help_ui, "Open docs/trust-center.md", "src/bin/ui_help.rs")
    require(help_ui, "Open docs/backend-registry.md", "src/bin/ui_help.rs")
    require(help_ui, "Open full advanced reference", "src/bin/ui_help.rs")
    require(
        help_ui,
        "backend_tool_entries_cover_expected_catalog",
        "src/bin/ui_help.rs",
    )
    require(
        help_ui,
        "backend_tool_entries_keep_local_paths",
        "src/bin/ui_help.rs",
    )

    require(mode, "pub(crate) fn mode_summary", "src/bin/ui_mode.rs")
    require(mode, "pub(crate) fn mode_summary_panel", "src/bin/ui_mode.rs")
    require(mode, "pub(crate) fn mode_dashboard_panel", "src/bin/ui_mode.rs")
    require(mode, "pub(crate) fn info_chip", "src/bin/ui_mode.rs")
    require(mode, "pub(crate) fn ghost_action", "src/bin/ui_mode.rs")
    require(mode, "fn mode_dashboard", "src/bin/ui_mode.rs")
    require(mode, "fn desktop_repair_action", "src/bin/ui_mode.rs")
    require(mode, "desktop_dashboard_uses_shared_readiness_ids", "src/bin/ui_mode.rs")
    require(
        mode,
        "desktop_direct_dashboard_matches_auto_default_readiness",
        "src/bin/ui_mode.rs",
    )
    require(
        mode,
        "desktop_repair_actions_route_to_expected_tabs",
        "src/bin/ui_mode.rs",
    )

    require(monitor, "pub(crate) fn traffic_stat_rows", "src/bin/ui_monitor.rs")
    require(monitor, "pub(crate) fn quota_calls_per_hour", "src/bin/ui_monitor.rs")
    require(monitor, "pub(crate) fn degradation_changes", "src/bin/ui_monitor.rs")
    require(monitor, "pub(crate) fn notable_failure_lines", "src/bin/ui_monitor.rs")
    require(monitor, "traffic_rows_keep_expected_metric_labels", "src/bin/ui_monitor.rs")
    require(monitor, "quota_rate_handles_midnight_reset_boundary", "src/bin/ui_monitor.rs")
    require(
        monitor,
        "degradation_changes_collapse_adjacent_duplicates_and_cap",
        "src/bin/ui_monitor.rs",
    )
    require(
        monitor,
        "notable_failure_lines_filters_reverses_and_caps",
        "src/bin/ui_monitor.rs",
    )

    require(setup, "pub(crate) fn show_first_run_wizard", "src/bin/ui_setup.rs")
    require(setup, "Cmd::Test", "src/bin/ui_setup.rs")
    require(setup, "Cmd::Doctor", "src/bin/ui_setup.rs")
    require(setup, "Cmd::InstallCa", "src/bin/ui_setup.rs")
    require(setup, "Cmd::CheckCaTrusted", "src/bin/ui_setup.rs")
    require(setup, "wizard_steps_keep_expected_order", "src/bin/ui_setup.rs")

    require(style, "pub(crate) const ACCENT", "src/bin/ui_style.rs")
    require(style, "pub(crate) const TEXT_LABEL", "src/bin/ui_style.rs")
    require(style, "pub(crate) fn apply_ui_theme", "src/bin/ui_style.rs")
    require(style, "pub(crate) fn section", "src/bin/ui_style.rs")
    require(style, "pub(crate) fn help_muted", "src/bin/ui_style.rs")
    require(style, "pub(crate) fn primary_button", "src/bin/ui_style.rs")
    require(style, "pub(crate) fn form_row", "src/bin/ui_style.rs")
    require(style, "style_tokens_keep_expected_palette", "src/bin/ui_style.rs")
    require(style, "form_row_metrics_stay_stable", "src/bin/ui_style.rs")

    require(
        trust,
        "pub(crate) fn trust_center_snapshot_panel",
        "src/bin/ui_trust.rs",
    )
    require(trust, "pub(crate) fn support_bundle_preview", "src/bin/ui_trust.rs")
    require(trust, "pub(crate) fn trust_center_tab", "src/bin/ui_trust.rs")
    require(trust, "Cmd::InstallCa", "src/bin/ui_trust.rs")
    require(trust, "Cmd::RemoveCa", "src/bin/ui_trust.rs")
    require(trust, "Cmd::CheckCaTrusted", "src/bin/ui_trust.rs")
    require(trust, "trust_status_labels_are_stable", "src/bin/ui_trust.rs")
    require(trust, "trust_tab_docs_links_are_local_reference_paths", "src/bin/ui_trust.rs")
    require(trust, "support_manifest_has_expected_redaction_contract", "src/bin/ui_trust.rs")
    require(trust, "support_bundle::preview_manifest", "src/bin/ui_trust.rs")
    require(trust, "trust_center::snapshot", "src/bin/ui_trust.rs")

    require(xhttp, "pub(crate) struct XhttpGeneratorForm", "src/bin/ui_xhttp.rs")
    require(
        xhttp,
        "pub(crate) fn xhttp_platform_defaults",
        "src/bin/ui_xhttp.rs",
    )
    require(
        xhttp,
        "pub(crate) fn generate_xhttp_vless_links",
        "src/bin/ui_xhttp.rs",
    )
    require(
        xhttp,
        "pub(crate) fn generate_xhttp_deploy_notes",
        "src/bin/ui_xhttp.rs",
    )
    require(xhttp, "pub(crate) struct XhttpDeployPipe", "src/bin/ui_xhttp.rs")
    require(
        xhttp,
        "pub(crate) fn xhttp_vless_generator",
        "src/bin/ui_xhttp.rs",
    )
    require(
        xhttp,
        "pub(crate) fn poll_xhttp_cloud_deploy",
        "src/bin/ui_xhttp.rs",
    )
    require(xhttp, "xhttp_cloud_deploy::deploy_vercel_xhttp", "src/bin/ui_xhttp.rs")
    require(xhttp, "xhttp_cloud_deploy::deploy_netlify_xhttp", "src/bin/ui_xhttp.rs")
    require(
        xhttp,
        "generate_xhttp_links_deduplicates_candidates",
        "src/bin/ui_xhttp.rs",
    )
    require(
        xhttp,
        "generate_xhttp_deploy_notes_requires_full_target_url",
        "src/bin/ui_xhttp.rs",
    )

    require(tools, "Desktop UI modularization guard", "tools/README.md")
    require(tools, "check-desktop-ui-modularization.py", "tools/README.md")
    require(source_map, "src/bin/ui.rs", "docs/tooling-source-map.json")
    require(source_map, "tools/check-desktop-ui-modularization.py", "docs/tooling-source-map.json")
    require(sanity, "tools/check-desktop-ui-modularization.py", "tools/run-repo-sanity.py")
    require(parity, "tools/check-desktop-ui-modularization.py", "tools/check-ci-local-sanity-parity.py")

    print("desktop UI modularization check: ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
