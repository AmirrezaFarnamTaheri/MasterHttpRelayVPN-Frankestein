#!/usr/bin/env python3
"""Generate docs/platform-defaults.md from docs/platform-defaults.json."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[1]
SRC = ROOT / "docs" / "platform-defaults.json"
OUT = ROOT / "docs" / "platform-defaults.md"


def esc(s: str) -> str:
    return str(s).replace("|", "\\|").replace("\n", " ")


def render_kv_table(title: str, rows: list[tuple[str, Any]]) -> str:
    lines = [f"## {title}", "", "| Setting | Value |", "| --- | --- |"]
    for k, v in rows:
        lines.append(f"| `{esc(str(k))}` | {esc(v)} |")
    lines.append("")
    return "\n".join(lines)


def render(spec: dict[str, Any]) -> str:
    parts: list[str] = []
    parts.append("# Platform defaults")
    parts.append("")
    parts.append(
        f"Contract version `{spec.get('contract_version')}`. "
        f"Canonical JSON: `{SRC.relative_to(ROOT).as_posix()}`."
    )
    parts.append(
        "CI runs `python3 tools/check-platform-defaults.py` (code versus JSON) and "
        "`python3 tools/generate-platform-defaults-doc.py -Check` (this page versus JSON)."
    )
    parts.append("")
    parts.append("## Purpose")
    parts.append("")
    parts.append(str(spec.get("purpose", "")))
    parts.append("")

    shared = spec["shared"]
    parity = spec["parity_shared_defaults"]
    rd = spec["rust_desktop_cli"]
    android = spec["android"]

    socks_cell = (
        "`null` (SOCKS5 listener disabled until `socks5_port` is set)"
        if rd["socks5_port_when_json_field_absent"] is None
        else rd["socks5_port_when_json_field_absent"]
    )
    parity_rows = sorted(parity.items(), key=lambda kv: kv[0])
    sections = [
        render_kv_table(
            "Shared expectations",
            list(shared.items()),
        ),
        render_kv_table(
            "Same on Rust + Android (`verify_ssl`, relay path, QUIC/DoH toggles)",
            [(k, v) for k, v in parity_rows],
        ),
        render_kv_table(
            "Rust Desktop / CLI (`serde` defaults in `src/config.rs`)",
            [
                ("google_ip_default", rd["google_ip_default"]),
                ("listen_port_default", rd["listen_port_default"]),
                ("log_level_default", rd["log_level_default"]),
                ("socks5 when JSON omits socks5_port", socks_cell),
                ("parallel_relay when JSON omits field", rd["parallel_relay_when_field_absent"]),
                ("coalesce_step_ms when JSON omits field", rd["coalesce_step_ms_when_field_absent"]),
                ("coalesce_max_ms when JSON omits field", rd["coalesce_max_ms_when_field_absent"]),
                ("notes SOCKS5/examples", rd["socks5_note"]),
                ("notes parallel_relay", rd["parallel_relay_note"]),
                ("notes coalesce", rd["coalesce_note"]),
            ],
        ),
        render_kv_table(
            "Android (`MhrvConfig` + importer fallbacks in `ConfigStore.kt`)",
            [
                ("google_ip_default", android["google_ip_default"]),
                ("listen_port_default", android["listen_port_default"]),
                ("socks5_port_default", android["socks5_port_default"]),
                ("log_level_default", android["log_level_default"]),
                ("parallel_relay_default", android["parallel_relay_default"]),
                ("coalesce_step_ms_default", android["coalesce_step_ms_default"]),
                ("coalesce_max_ms_default", android["coalesce_max_ms_default"]),
            ],
        ),
    ]
    parts.append("\n".join(s.strip("\n") for s in sections).strip())
    parts.append("## Rationale (intentional differences)")
    parts.append("")
    parts.append(f"> **Ports**: {esc(android['rationale_ports_vs_desktop'])}")
    parts.append("")
    parts.append(f"> **Google edge IP preset**: {esc(android['rationale_google_ip_vs_rust'])}")
    parts.append("")
    parts.append(f"> **parallel_relay**: {esc(android['rationale_parallel_relay_vs_rust'])}")
    parts.append("")
    parts.append(f"> **Coalesce timers**: {esc(android['rationale_coalesce_ms_vs_rust'])}")
    parts.append("")
    return "\n".join(parts).rstrip() + "\n"


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("-Check", action="store_true")
    args = parser.parse_args()

    spec = json.loads(SRC.read_text(encoding="utf-8"))
    out = render(spec)

    if args.Check:
        if not OUT.exists() or OUT.read_text(encoding="utf-8") != out:
            print(f"stale: {OUT.relative_to(ROOT)}")
            return 1
        print(f"ok platform-defaults doc {OUT.relative_to(ROOT)}")
        return 0

    OUT.write_text(out, encoding="utf-8")
    print(f"wrote {OUT}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
