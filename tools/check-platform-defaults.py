#!/usr/bin/env python3
"""Verify Rust/Kotlin compiled defaults match docs/platform-defaults.json."""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
CONTRACT = ROOT / "docs" / "platform-defaults.json"
RUST_CFG = ROOT / "src" / "config.rs"
KOTLIN = ROOT / "android" / "app" / "src" / "main" / "java" / "com" / "farnam" / "mhrvf" / "ConfigStore.kt"


def _rust_src() -> str:
    return RUST_CFG.read_text(encoding="utf-8")


def _rust_fn_string(func: str) -> str:
    body = _rust_src()
    m = re.search(rf'fn\s+{re.escape(func)}\s*\([^)]*\)\s*->\s*String\s*\{{\s*"([^"]+)"\.into', body)
    if not m:
        raise SystemExit(f"{RUST_CFG}: missing string literal body for fn {func}")
    return m.group(1)


def _rust_fn_bool(func: str) -> bool:
    body = _rust_src()
    m = re.search(rf'fn\s+{re.escape(func)}\s*\([^)]*\)\s*->\s*bool\s*\{{\s*(true|false)\s*\n', body)
    if not m:
        raise SystemExit(f"{RUST_CFG}: missing bool literal body for fn {func}")
    return m.group(1) == "true"


def _rust_fn_u16(func: str) -> int:
    body = _rust_src()
    m = re.search(rf"fn\s+{re.escape(func)}\s*\([^)]*\)\s*->\s*u16\s*\{{\s*(\d+)\s*", body)
    if not m:
        raise SystemExit(f"{RUST_CFG}: missing u16 literal for fn {func}")
    return int(m.group(1))


def _kotlin_google_const(text: str) -> str:
    m = re.search(r'private const val DEFAULT_ANDROID_GOOGLE_IP\s*=\s*"([^"]+)"', text)
    if not m:
        raise SystemExit(f"{KOTLIN}: DEFAULT_ANDROID_GOOGLE_IP not found")
    return m.group(1)


def _kotlin_opt_int_fallback(key_snake: str, expect: int, *, min_occurrences: int = 2) -> None:
    kt = KOTLIN.read_text(encoding="utf-8")
    pat = rf"optInt\s*\(\s*[\"']{re.escape(key_snake)}[\"']\s*,\s*(\d+)\s*\)"
    matches = [int(x) for x in re.findall(pat, kt)]
    if len(matches) < min_occurrences:
        raise SystemExit(
            f"{KOTLIN}: expected at least {min_occurrences} optInt({key_snake!r}, …) usages, found {len(matches)}"
        )
    for i, v in enumerate(matches):
        if v != expect:
            raise SystemExit(f"{KOTLIN}: optInt({key_snake!r})[{i}]={v}, expected {expect}")


def _kotlin_opt_bool_fallback(key_snake: str, expect: bool, *, min_occurrences: int = 2) -> None:
    kt = KOTLIN.read_text(encoding="utf-8")
    want = "true" if expect else "false"
    pat = rf"optBoolean\s*\(\s*[\"']{re.escape(key_snake)}[\"']\s*,\s*(true|false)\s*\)"
    matches = re.findall(pat, kt)
    if len(matches) < min_occurrences:
        raise SystemExit(
            f"{KOTLIN}: expected ≥{min_occurrences} optBoolean({key_snake!r}), found {len(matches)}"
        )
    for i, tok in enumerate(matches):
        lit = tok == "true"
        if lit != expect:
            raise SystemExit(f"{KOTLIN}: optBoolean({key_snake!r})[{i}]={tok}, expected {want}")


def _kotlin_socks5_json_fallback(expect: int) -> None:
    kt = KOTLIN.read_text(encoding="utf-8")
    ms = list(
        re.finditer(
            r'optionalPositiveInt\(obj\s*,\s*"socks5_port"\s*,\s*(\d+)\s*\)',
            kt,
        )
    )
    if len(ms) < 2:
        raise SystemExit(
            f"{KOTLIN}: expected ≥2 optionalPositiveInt(\"socks5_port\", …); found {len(ms)}"
        )
    for i, m in enumerate(ms):
        v = int(m.group(1))
        if v != expect:
            raise SystemExit(f"{KOTLIN}: socks5_port fallback[{i}]={v}, expected {expect}")


def main() -> int:
    spec = json.loads(CONTRACT.read_text(encoding="utf-8"))
    shared = spec["shared"]
    parity = spec["parity_shared_defaults"]
    rust_expect = spec["rust_desktop_cli"]
    and_expect = spec["android"]

    errs: list[str] = []
    kotlin_txt = ""

    try:
        g_rust = _rust_fn_string("default_google_ip")
        relay_rust = _rust_fn_string("default_vercel_relay_path")
        fd_rust = _rust_fn_string("default_front_domain")
        lh_rust = _rust_fn_string("default_listen_host")
        lp_rust = _rust_fn_u16("default_listen_port")
        ll_rust = _rust_fn_string("default_log_level")
        verify_rust_fn = _rust_fn_bool("default_verify_ssl")
        tunnel_rust_fn = _rust_fn_bool("default_tunnel_doh")

        kotlin_txt = KOTLIN.read_text(encoding="utf-8")

        kotlin_ip_const = _kotlin_google_const(kotlin_txt)
        m_ip = re.search(
            r"val\s+googleIp:\s*String\s*=\s*([^\n,]+),",
            kotlin_txt,
        )
        if not m_ip:
            raise SystemExit(f"{KOTLIN}: could not parse MhrvConfig.googleIp default expr")
        google_ip_ctor = m_ip.group(1).strip()
        mp = re.search(
            r"val\s+listenPort:\s*Int\s*=\s*(\d+),",
            kotlin_txt,
        )
        if not mp:
            raise SystemExit(f"{KOTLIN}: listenPort default not found")
        ms = re.search(
            r"val\s+socks5Port:\s*Int\?\s*=\s*(\d+)\s*,",
            kotlin_txt,
        )
        if not ms:
            raise SystemExit(f"{KOTLIN}: socks5Port default not found")
        mfd = re.search(
            r'val\s+frontDomain:\s*String\s*=\s*"([^"]+)",',
            kotlin_txt,
        )
        if not mfd:
            raise SystemExit(f"{KOTLIN}: frontDomain default not found")
        mlh = re.search(
            r'val\s+listenHost:\s*String\s*=\s*"([^"]+)",',
            kotlin_txt,
        )
        if not mlh:
            raise SystemExit(f"{KOTLIN}: listenHost default not found")
        mll = re.search(
            r'val\s+logLevel:\s*String\s*=\s*"([^"]+)",',
            kotlin_txt,
        )
        if not mll:
            raise SystemExit(f"{KOTLIN}: logLevel default not found")
        mv = re.search(
            r"val\s+verifySsl:\s*Boolean\s*=\s*(true|false),",
            kotlin_txt,
        )
        myt = re.search(
            r"val\s+youtubeViaRelay:\s*Boolean\s*=\s*(true|false),",
            kotlin_txt,
        )
        mbq = re.search(
            r"val\s+blockQuic:\s*Boolean\s*=\s*(true|false),",
            kotlin_txt,
        )
        mtd = re.search(
            r"val\s+tunnelDoh:\s*Boolean\s*=\s*(true|false),",
            kotlin_txt,
        )
        mrpath = re.search(
            r"val\s+serverlessRelayPath:\s*String\s*=\s*([^\n,]+),",
            kotlin_txt,
        )
        mpr = re.search(
            r"val\s+parallelRelay:\s*Int\s*=\s*(\d+),",
            kotlin_txt,
        )
        mcs = re.search(
            r"val\s+coalesceStepMs:\s*Int\s*=\s*(\d+),",
            kotlin_txt,
        )
        mcm = re.search(
            r"val\s+coalesceMaxMs:\s*Int\s*=\s*(\d+),",
            kotlin_txt,
        )
        for lbl, mobj in (
            ("verifySsl", mv),
            ("youtubeViaRelay", myt),
            ("blockQuic", mbq),
            ("tunnelDoh", mtd),
            ("serverlessRelayPath", mrpath),
            ("parallelRelay", mpr),
            ("coalesceStepMs", mcs),
            ("coalesceMaxMs", mcm),
        ):
            if mobj is None:
                raise SystemExit(f"{KOTLIN}: could not parse MhrvConfig.{lbl} default")
    except SystemExit as e:
        print(str(e), file=sys.stderr)
        return 1

    try:
        _kotlin_opt_int_fallback("listen_port", int(and_expect["listen_port_default"]))
        _kotlin_socks5_json_fallback(int(and_expect["socks5_port_default"]))
        _kotlin_opt_int_fallback("parallel_relay", int(and_expect["parallel_relay_default"]))
        _kotlin_opt_int_fallback("coalesce_step_ms", int(and_expect["coalesce_step_ms_default"]))
        _kotlin_opt_int_fallback("coalesce_max_ms", int(and_expect["coalesce_max_ms_default"]))
        _kotlin_opt_bool_fallback("verify_ssl", bool(parity["verify_ssl"]))
        _kotlin_opt_bool_fallback("youtube_via_relay", bool(parity["youtube_via_relay"]))
        _kotlin_opt_bool_fallback("block_quic", bool(parity["block_quic"]))
        _kotlin_opt_bool_fallback("tunnel_doh", bool(parity["tunnel_doh"]))
    except SystemExit as e:
        print(str(e), file=sys.stderr)
        return 1

    rust_body = _rust_src()
    if rust_expect["socks5_port_when_json_field_absent"] is not None:
        errs.append("JSON rust socks5 sentinel must be JSON null unless schema changes.")
    elif not re.search(
        r"#\[serde\(default\)\]\s*\n\s*pub\s+socks5_port:\s*Option<u16>",
        rust_body,
    ):
        errs.append(f"{RUST_CFG}: expected #[serde(default)] pub socks5_port: Option<u16>")

    if not re.search(
        r"#\[serde\(default\)\]\s*\n\s*pub\s+parallel_relay:\s*u8",
        rust_body,
    ):
        errs.append(f"{RUST_CFG}: expected #[serde(default)] pub parallel_relay: u8")
    if rust_expect["parallel_relay_when_field_absent"] != 0:
        errs.append("Contract parallel_relay_when_field_absent must stay 0 (serde default).")
    if not re.search(
        r"#\[serde\(default\)\]\s*\n\s*pub\s+coalesce_step_ms:\s*u16",
        rust_body,
    ):
        errs.append(f"{RUST_CFG}: expected #[serde(default)] pub coalesce_step_ms: u16")
    if not re.search(
        r"#\[serde\(default\)\]\s*\n\s*pub\s+coalesce_max_ms:\s*u16",
        rust_body,
    ):
        errs.append(f"{RUST_CFG}: expected #[serde(default)] pub coalesce_max_ms: u16")

    # #[serde(default = "default_verify_ssl")] on verify_ssl
    if not re.search(
        r"#\[serde\(default\s*=\s*\"default_verify_ssl\"\)\]\s*\n\s*pub\s+verify_ssl:\s*bool",
        rust_body,
    ):
        errs.append(f"{RUST_CFG}: verify_ssl must default via default_verify_ssl")
    # #[serde(default = "default_tunnel_doh")] on tunnel_doh
    if not re.search(
        r"#\[serde\(default\s*=\s*\"default_tunnel_doh\"\)\]\s*\n\s*pub\s+tunnel_doh:\s*bool",
        rust_body,
    ):
        errs.append(f"{RUST_CFG}: tunnel_doh must default via default_tunnel_doh")
    for field in ("youtube_via_relay", "block_quic"):
        if not re.search(
            rf"#\[serde\(default\)\]\s*\n\s*pub\s+{re.escape(field)}:\s*bool",
            rust_body,
        ):
            errs.append(f"{RUST_CFG}: expected #[serde(default)] pub {field}: bool")

    m_rel_const = re.search(r'private const val DEFAULT_RELAY_PATH\s*=\s*"([^"]+)"', kotlin_txt)
    if not m_rel_const:
        errs.append(f"{KOTLIN}: DEFAULT_RELAY_PATH const not found")
    elif m_rel_const.group(1) != parity["serverless_relay_path"]:
        errs.append(
            "Kotlin DEFAULT_RELAY_PATH vs parity_shared_defaults.serverless_relay_path: "
            + f"code={m_rel_const.group(1)!r}"
        )

    if relay_rust != parity["serverless_relay_path"]:
        errs.append(f"Rust default_vercel_relay_path: code={relay_rust!r} json={parity['serverless_relay_path']!r}")
    if verify_rust_fn != bool(parity["verify_ssl"]):
        errs.append("Rust default_verify_ssl disagrees with parity_shared_defaults.verify_ssl")
    if tunnel_rust_fn != bool(parity["tunnel_doh"]):
        errs.append("Rust default_tunnel_doh disagrees with parity_shared_defaults.tunnel_doh")

    # Rust derives false booleans implicitly; schema still documents expected values:
    expect_false = {"youtube_via_relay", "block_quic"}
    for k in expect_false:
        if parity[k]:
            errs.append(f"parity_shared_defaults.{k} must remain false")

    # Kotlin ctor vs parity + android-only numerics / paths
    if mv.group(1) != str(parity["verify_ssl"]).lower():
        errs.append(f"MhrvConfig.verifySsl default {mv.group(1)} vs parity verify_ssl")

    yt_lit = myt.group(1) == "true"
    bq_lit = mbq.group(1) == "true"
    td_lit = mtd.group(1) == "true"

    if yt_lit != parity["youtube_via_relay"]:
        errs.append(f"MhrvConfig.youtubeViaRelay ctor vs parity mismatch")
    if bq_lit != parity["block_quic"]:
        errs.append(f"MhrvConfig.blockQuic ctor vs parity mismatch")
    if td_lit != parity["tunnel_doh"]:
        errs.append(f"MhrvConfig.tunnelDoh ctor vs parity mismatch")

    rpath_ctor = mrpath.group(1).strip()
    if rpath_ctor != "DEFAULT_RELAY_PATH":
        errs.append(
            "MhrvConfig.serverlessRelayPath must default to DEFAULT_RELAY_PATH "
            + f"(match Rust default_vercel_relay_path); saw {rpath_ctor!r}"
        )

    if int(mpr.group(1)) != int(and_expect["parallel_relay_default"]):
        errs.append("MhrvConfig.parallelRelay vs JSON android.parallel_relay_default")
    if int(mcs.group(1)) != int(and_expect["coalesce_step_ms_default"]):
        errs.append("MhrvConfig.coalesceStepMs vs JSON")
    if int(mcm.group(1)) != int(and_expect["coalesce_max_ms_default"]):
        errs.append("MhrvConfig.coalesceMaxMs vs JSON")

    # Rust scalar defaults previously checked
    if g_rust != rust_expect["google_ip_default"]:
        errs.append(f"Rust default_google_ip: code={g_rust!r} json={rust_expect['google_ip_default']!r}")
    if lp_rust != rust_expect["listen_port_default"]:
        errs.append(f"Rust default_listen_port: code={lp_rust} json={rust_expect['listen_port_default']}")
    if fd_rust != shared["front_domain"]:
        errs.append(f"Rust default_front_domain vs json.shared.front_domain")
    if lh_rust != shared["listen_host_loopback"]:
        errs.append(f"Rust default_listen_host vs shared.listen_host_loopback")
    if ll_rust != rust_expect["log_level_default"]:
        errs.append(f"Rust default_log_level vs json rust_desktop_cli.log_level_default")

    if kotlin_ip_const != and_expect["google_ip_default"]:
        errs.append("Kotlin DEFAULT_ANDROID_GOOGLE_IP vs json android.google_ip_default")
    if google_ip_ctor != "DEFAULT_ANDROID_GOOGLE_IP":
        errs.append(
            "MhrvConfig.googleIp must default to DEFAULT_ANDROID_GOOGLE_IP; "
            + f"saw {google_ip_ctor!r}"
        )
    if int(mp.group(1)) != int(and_expect["listen_port_default"]):
        errs.append("MhrvConfig.listenPort vs JSON")
    socks_code = int(ms.group(1))
    socks_json = int(and_expect["socks5_port_default"])
    if socks_code != socks_json:
        errs.append("MhrvConfig.socks5Port vs JSON")
    if mfd.group(1) != shared["front_domain"]:
        errs.append("MhrvConfig.frontDomain vs JSON shared.front_domain")
    if mlh.group(1) != shared["listen_host_loopback"]:
        errs.append("MhrvConfig.listenHost vs JSON")
    if mll.group(1) != and_expect["log_level_default"]:
        errs.append("MhrvConfig.logLevel vs JSON android.log_level_default")

    if errs:
        print("\n".join(errs), file=sys.stderr)
        return 1
    print("ok platform-defaults contract matches Rust + Kotlin")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
