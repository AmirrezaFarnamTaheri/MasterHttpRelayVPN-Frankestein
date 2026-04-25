# MasterHttpRelayVPN-Frankestein

[![Latest release](https://img.shields.io/github/v/release/AmirrezaFarnamTaheri/MasterHttpRelayVPN-Frankestein?sort=semver&display_name=tag&logo=github&label=release)](https://github.com/AmirrezaFarnamTaheri/MasterHttpRelayVPN-Frankestein/releases/latest)
[![Downloads](https://img.shields.io/github/downloads/AmirrezaFarnamTaheri/MasterHttpRelayVPN-Frankestein/total?label=downloads&logo=github)](https://github.com/AmirrezaFarnamTaheri/MasterHttpRelayVPN-Frankestein/releases)
[![CI](https://github.com/AmirrezaFarnamTaheri/MasterHttpRelayVPN-Frankestein/actions/workflows/release.yml/badge.svg)](https://github.com/AmirrezaFarnamTaheri/MasterHttpRelayVPN-Frankestein/actions/workflows/release.yml)
[![License: MIT](https://img.shields.io/github/license/AmirrezaFarnamTaheri/MasterHttpRelayVPN-Frankestein?color=blue)](LICENSE)
[![Stars](https://img.shields.io/github/stars/AmirrezaFarnamTaheri/MasterHttpRelayVPN-Frankestein?style=flat&logo=github)](https://github.com/AmirrezaFarnamTaheri/MasterHttpRelayVPN-Frankestein/stargazers)
[![Support](https://img.shields.io/badge/❤️_Support-sh1n.org-red?style=flat)](https://sh1n.org/donate)

**MasterHttpRelayVPN-Frankestein** is the unified **mhrv-f** line: one CLI, one desktop UI, and one Android app—merging the relay, tunneling, and tooling ideas that grew around the original [MasterHttpRelayVPN](https://github.com/masterking32/MasterHttpRelayVPN) design. **Credit for the Apps Script relay concept and the reference `Code.gs` goes to [@masterking32](https://github.com/masterking32).**

Free DPI bypass via Google Apps Script as a remote relay, with TLS SNI concealment. Your ISP sees traffic toward `www.google.com`; the relay in your own Google account fetches the real destination for you.

Bug reports and contributions: [issues](https://github.com/AmirrezaFarnamTaheri/MasterHttpRelayVPN-Frankestein/issues).

**[English Guide](#setup-guide)** | **[راهنمای فارسی](#راهنمای-فارسی)**

## Why this exists

A single self-contained build keeps setup reliable on networks where language runtimes and package managers are hard to use. You run one downloaded binary; no separate interpreter install.

## How it works

```
Browser / Telegram / xray
        |
        | HTTP proxy (8085)  or  SOCKS5 (8086)
        v
mhrv-f (local)
        |
        | TLS to Google IP, SNI = www.google.com
        v                       ^
   DPI sees www.google.com      |
        |                       | Host: script.google.com (inside TLS)
        v                       |
  Google edge frontend ---------+
        |
        v
  Apps Script relay (your free Google account)
        |
        v
  Real destination
```

The censor's DPI sees `www.google.com` in the TLS SNI and lets it through. Google's frontend hosts both `www.google.com` and `script.google.com` on the same IP and routes by the HTTP `Host` header inside the encrypted stream.

For a handful of Google-owned domains (`google.com`, `youtube.com`, `fonts.googleapis.com`, …) the same tunnel is used directly instead of going through the Apps Script relay. This bypasses the per-fetch quota and fixes the "User-Agent is always `Google-Apps-Script`" problem for those domains. You can add more domains via the `hosts` map in config.

## Platforms

Linux (x86_64, aarch64), macOS (x86_64, aarch64), Windows (x86_64), **Android 7.0+** (universal APK covering arm64, armv7, x86_64, x86). Prebuilt binaries on the [releases page](https://github.com/AmirrezaFarnamTaheri/MasterHttpRelayVPN-Frankestein/releases).

**Android users** — grab `mhrv-f-android-universal-v*.apk` and follow the full walk-through in [docs/android.md](docs/android.md) (English) or [docs/android.fa.md](docs/android.fa.md) (فارسی). The Android build runs the exact same `mhrv-f` crate as the desktop (via JNI) and adds a TUN bridge via `tun2proxy`, so every app on the device routes its IP traffic through the proxy without per-app configuration.

> **Android and HTTPS in other apps:** TUN mode captures IP traffic, but *HTTPS* in arbitrary apps only works if the app trusts the user-installed CA. On Android 7+ (`minSdk = 24`), that requires the app to opt in (e.g. `networkSecurityConfig`). **Chrome and Firefox typically do**; many chat, social, and banking apps do not. For those, use `PROXY_ONLY` with the app’s own proxy settings, `google_only` when you only need Google services, or `upstream_socks5` to another path. This follows normal Android trust rules, not a limitation specific to this app.

## What's in a release

Each archive contains two binaries and a launcher script:

| file | purpose |
|---|---|
| `mhrv-f` / `mhrv-f.exe` | CLI. Headless use, servers, automation. Works on all platforms; no system deps on macOS/Windows. |
| `mhrv-f-ui` / `mhrv-f-ui.exe` | Desktop UI (egui). Config form, Start/Stop/Test buttons, live stats, log panel. |
| `run.sh` / `run.command` / `run.bat` | Platform launcher: installs the MITM CA (needs sudo/admin) and then starts the UI. Use this on first run. |

macOS archives also ship `mhrv-f.app` (in `*-app.zip`) — double-click to launch the UI without a terminal. You'll still need to run the CLI (`mhrv-f --install-cert`) or `run.command` once to install the CA.

> Screenshot note: the UI changes frequently; instead of shipping a stale screenshot, use the in-app **Help & walkthrough** panel and the docs hub at `docs/index.md`.

Linux UI also needs common desktop libraries available: `libxkbcommon`, `libwayland-client`, `libxcb`, `libgl`, `libx11`, `libgtk-3`. On most desktop distros these are already present; on a headless box install them via your package manager, or just use the CLI.

## Where things live

Config and the MITM CA live in the OS user-data dir:

- macOS: `~/Library/Application Support/mhrv-f/`
- Linux: `~/.config/mhrv-f/`
- Windows: `%APPDATA%\mhrv-f\`

Inside that dir:

- `config.json` — your settings (written by the UI's **Save** button or hand-edited)
- `ca/ca.crt`, `ca/ca.key` — the MITM root certificate. Only you have the private key.

The CLI also reads `config.json` in the current working directory if you prefer a local file next to the binary.

## Setup Guide

### Step 1 — Deploy the Apps Script relay (one-time)

This part is unchanged from the original project. Follow @masterking32's guide or the summary below:

1. Open <https://script.google.com> while signed into your Google account.
2. **New project**, delete the default code.
3. Copy the relay script from this repo: [`assets/apps_script/Code.gs`](assets/apps_script/Code.gs) (see [`assets/apps_script/README.md`](assets/apps_script/README.md) for the upstream relationship). Paste it into the Apps Script editor.
4. Change `const AUTH_KEY = "..."` to a strong secret only you know.
5. **Deploy → New deployment → Web app**.
   - Execute as: **Me**
   - Who has access: **Anyone**
6. Copy the **Deployment ID** (the long random string in the URL).

#### Can't reach `script.google.com` from your network?

If your ISP is already blocking Google Apps Script (or all of Google), you need Step 1's browser connection to succeed *before* you have a relay to use. `mhrv-f` ships a small bootstrap mode for exactly this: `google_only`.

1. Build / download the binary as in Step 2 below.
2. Copy [`config.google-only.example.json`](config.google-only.example.json) to `config.json` — no `script_id`, no `auth_key` required.
3. Run `mhrv-f serve` and set your browser's HTTP proxy to `127.0.0.1:8085`.
4. In `google_only` mode the proxy only relays `*.google.com`, `*.youtube.com`, and the other Google-edge hosts via the same SNI-rewrite tunnel the full client uses. Other traffic goes direct — no Apps Script relay exists yet.
5. Do Step 1 in your browser (the connection to `script.google.com` will be SNI-fronted). Deploy Code.gs, copy the Deployment ID.
6. In the desktop UI or the Android app (or by editing `config.json`) switch the mode back to `apps_script`, paste the Deployment ID and your auth key, and restart.

You can also verify reachability before even starting the proxy: `mhrv-f test-sni` probes `*.google.com` directly and works without any config beyond `google_ip` + `front_domain`.

### Step 2 — Download

Grab the archive for your platform from the [releases page](https://github.com/AmirrezaFarnamTaheri/MasterHttpRelayVPN-Frankestein/releases) and extract it.

Or build from source (from the repository root — the directory that contains this file and `Cargo.toml`):

```bash
git clone https://github.com/AmirrezaFarnamTaheri/MasterHttpRelayVPN-Frankestein.git
cd MasterHttpRelayVPN-Frankestein
cargo build --release --features ui
# Binaries: target/release/mhrv-f and target/release/mhrv-f-ui
```

If you renamed the folder locally (for example to `mhrv-f`), `cd` into that name instead; only the path changes.

### Step 3 — First run: install the MITM CA

To route your browser's HTTPS traffic through the Apps Script relay, `mhrv-f` has to terminate TLS locally on your machine, forward the request through the relay, and re-encrypt the response with a certificate your browser trusts. That requires a small **local** Certificate Authority.

**What actually happens on first run:**

- A fresh CA keypair (`ca/ca.crt` + `ca/ca.key`) is generated **on your machine**, in your user-data dir.
- The public `ca.crt` is added to your system trust store so browsers accept the per-site certificates `mhrv-f` mints on the fly. This is the step that needs sudo / Administrator.
- The private `ca.key` **never leaves your machine**. Nothing uploads it, nothing phones home, and no remote party — including the Apps Script relay — can use it to impersonate sites to you.
- You can revoke it at any time by deleting the CA from your OS keychain (macOS: Keychain Access → System → delete `mhrv-f`) / Windows cert store / `/etc/ca-certificates`, and removing the `ca/` folder.

The launcher does all of this for you and then starts the UI:

| platform | how |
|---|---|
| macOS | double-click `run.command` in Finder (or `./run.command` in a terminal) |
| Linux | `./run.sh` from a terminal |
| Windows | double-click `run.bat` |

It will ask for your password (sudo / UAC) **only** to trust the CA. After that the launcher also starts `mhrv-f-ui`. On later runs you don't need the launcher — the CA is already trusted, so you can open `mhrv-f.app` / `mhrv-f-ui.exe` / `mhrv-f-ui` directly.

If you prefer to do the CA step by hand:

```bash
# Linux / macOS
sudo ./mhrv-f --install-cert

# Windows (Administrator)
mhrv-f.exe --install-cert
```

Firefox keeps its own cert store; the installer also drops the CA into Firefox's NSS database via `certutil` (best-effort). If Firefox still complains, import `ca/ca.crt` manually via Settings → Privacy & Security → Certificates → View Certificates → Authorities → Import.

### Step 4 — Configure in the UI

Open the UI and fill in the form:

- **Multi-account pools (Advanced)** — add one or more **account groups**. Each group maps to one Google account (quota pool) and contains:
  - `auth_key` (must match `AUTH_KEY` inside that account's `Code.gs` / `CodeFull.gs`)
  - one or more deployment IDs (one per line)
  - optional `label`, `weight`, and `enabled`
- **Outage self-heal (Advanced)** — optional safety valve: if we see repeated eligible relay failures in a short window (`timeout`, `unreachable`, `overloaded`), `mhrv-f` drops its keep-alive pool and reconnects cleanly. Config knobs: `outage_reset_*`.
- **Adaptive runtime profile (Advanced)** — optional auto-tuning for a few hot-path knobs. Turn on `runtime_auto_tune` and pick `runtime_profile` (`eco`, `balanced`, `max_speed`) to set defaults for range-parallelism and relay timeouts (and parallel relay dispatch when `parallel_relay` is left at 0/1).
- **Google IP** — `216.239.38.120` is a solid default. Use the **scan** button to probe for a faster one from your network.
- **Front domain** — keep `www.google.com`.
- **HTTP port** / **SOCKS5 port** — defaults `8085` / `8086`.

Hit **Save**, then **Start**. Use **Test** any time to send one request end-to-end through the relay and report the result.

### Step 4 (alternative) — CLI only

Everything the UI does is also available in the CLI. Copy `config.example.json` to `config.json` (either next to the binary or into the user-data dir shown above), fill it in:

```json
{
  "config_version": 1,
  "mode": "apps_script",
  "google_ip": "216.239.38.120",
  "front_domain": "www.google.com",
  "account_groups": [
    {
      "label": "primary",
      "enabled": true,
      "weight": 1,
      "auth_key": "same-secret-as-in-code-gs",
      "script_ids": ["PASTE_DEPLOYMENT_ID_1", "PASTE_DEPLOYMENT_ID_2"]
    }
  ],
  "listen_host": "127.0.0.1",
  "listen_port": 8085,
  "socks5_port": 8086,
  "log_level": "info",
  "verify_ssl": true,
  "lan_token": null,
  "lan_allowlist": null,
  "outage_reset_enabled": true,
  "outage_reset_failure_threshold": 3,
  "outage_reset_window_ms": 5000,
  "outage_reset_cooldown_ms": 15000
}
```

Then:

```bash
./mhrv-f                   # serve (default)
./mhrv-f test              # one-shot end-to-end probe
./mhrv-f doctor            # guided diagnostics (first-run fix assistant)
./mhrv-f scan-ips          # rank Google frontend IPs by latency
./mhrv-f --install-cert    # reinstall the MITM CA
./mhrv-f --help
```

To scale quota and resilience, add more `account_groups` (one per Google account) and/or add more `script_ids` per group.

### Per-domain overrides (advanced)

Sometimes a domain needs special handling:

- A site breaks under MITM → force it **direct**.
- A site is flaky under range-chunking → **never chunk** it.
- A Google-owned host works better through the tunnel → force **sni_rewrite**.

Use `domain_overrides` in `config.json`:

```json
{
  "domain_overrides": [
    { "host": "bank.example", "force_route": "direct", "never_chunk": true },
    { "host": ".sensitive.example", "force_route": "direct" },
    { "host": ".cdn.example", "never_chunk": true },
    { "host": "fonts.gstatic.com", "force_route": "sni_rewrite" }
  ]
}
```

Rules are matched case-insensitively. `host` supports exact match (`"example.com"`) and leading-dot suffix match (`".example.com"` matches `example.com` and any subdomain). The **first match wins**.

### Resource governance (soft limiter)

If you want to reduce bursty quota spikes (especially with `parallel_relay` or large range-parallel downloads), you can optionally enable a **client-side relay call rate limit**:

```json
{
  "relay_rate_limit_qps": 8.0,
  "relay_rate_limit_burst": 16
}
```

This is a best-effort token bucket. It does not guarantee exact quotas, but it helps keep the client from stampeding your Apps Script deployments under load.

### Support bundle export (for issue reports)

If you need to report a bug, you can export an **anonymized** diagnostics bundle:

```bash
mhrv-f support-bundle
```

It writes a folder under your user-data directory (shown earlier in this README) at:

- `support-bundles/bundle-<timestamp>/`

The bundle includes:

- `meta.json` (version/platform)
- `config.redacted.json` (secrets removed; deployment IDs masked)
- `doctor.json` (doctor report)
- `status.json` (minimal status snapshot)

#### scan-ips configuration (optional)

By default, the scan-ips subcommand uses a static array of IPs.

You can enable dynamic IP discovery by setting fetch_ips_from_api to true in config.json:

```json
{
  "fetch_ips_from_api": true,
  "max_ips_to_scan": 100,
  "scan_batch_size": 100,
  "google_ip_validation": true
}
```

When enabled:

- Fetches goog.json from Google’s public IP ranges API
- Extracts all CIDRs and expands them to individual IPs
- Prioritizes IPs from famous Google domains (google.com, youtube.com, etc.)
- Randomly selects up to max_ips_to_scan candidates (prioritized IPs first)
- Tests only the selected candidates for connectivity and frontend validation. `google_ip_validation` checks response headers to confirm each IP behaves like a usable Google frontend.

By using this options you may find ips witch are faster than static array that is provided as default but there is no guarantee that this ips would work.


### Step 5 — Point your client at the proxy

The tool listens on **two** ports. Use whichever your client supports:

**HTTP proxy** (browsers, generic HTTP clients) — `127.0.0.1:8085`

- **Firefox** — Settings → Network Settings → **Manual proxy**. HTTP host `127.0.0.1`, port `8085`, tick **Also use this proxy for HTTPS**.
- **Chrome / Edge** — use the system proxy settings, or the **Proxy SwitchyOmega** extension.
- **macOS system-wide** — System Settings → Network → Wi-Fi → Details → Proxies → enable **Web Proxy (HTTP)** and **Secure Web Proxy (HTTPS)**, both `127.0.0.1:8085`.
- **Windows system-wide** — Settings → Network & Internet → Proxy → **Manual proxy setup**, address `127.0.0.1`, port `8085`.

**SOCKS5 proxy** (Telegram, xray, app-level clients) — `127.0.0.1:8086`, no auth.

- Works for HTTP, HTTPS, **and** non-HTTP protocols (Telegram's MTProto, raw TCP). The server auto-detects each connection: HTTP/HTTPS go through the Apps Script relay, SNI-rewritable domains go through the direct Google-edge tunnel, and anything else falls through to raw TCP.

## Telegram, IMAP, SSH — pair with xray (optional)

The Apps Script relay only speaks HTTP request/response, so non-HTTP protocols (Telegram MTProto, IMAP, SSH, arbitrary raw TCP) can't travel through it. Without anything else, those flows hit the direct-TCP fallback — which means they're not actually tunneled, and an ISP that blocks Telegram will still block them.

Fix: run a local [xray](https://github.com/XTLS/Xray-core) (or v2ray / sing-box) with a VLESS/Trojan/Shadowsocks outbound that goes to a VPS of your own, and point mhrv-f at xray's SOCKS5 inbound via the **Upstream SOCKS5** field (or the `upstream_socks5` config key). When set, raw-TCP flows coming through mhrv-f's SOCKS5 listener get chained into xray → the real tunnel, instead of connecting directly.

```
Telegram  ┐                                                    ┌─ Apps Script ── HTTP/HTTPS
          ├─ SOCKS5 :8086 ─┤ mhrv-f ├─ SNI rewrite ─── google.com, youtube.com, …
Browser   ┘                                                    └─ upstream SOCKS5 ─ xray ── VLESS ── your VPS   (Telegram, IMAP, SSH, raw TCP)
```

Example config fragment (both UI and JSON):

```json
{
  "upstream_socks5": "127.0.0.1:50529"
}
```

HTTP/HTTPS continues to route through the Apps Script relay (no change), and the SNI-rewrite tunnel for `google.com` / `youtube.com` / etc. keeps bypassing both — so YouTube stays as fast as before while Telegram gets a real tunnel.

## Full tunnel mode

Full tunnel mode (`"mode": "full"`) routes **all** traffic end-to-end through Apps Script and a remote [tunnel-node](tunnel-node/) — no MITM certificate needed. The trade-off is higher latency per request (every byte goes Apps Script → tunnel-node → destination), but it works for every protocol and every app without CA installation.

### How deployment IDs affect performance

Each Apps Script batch request takes ~2 seconds round-trip. In full mode, `mhrv-f` runs a **pipelined batch multiplexer** that fires multiple batch requests concurrently without waiting for the previous one to return. The number of in-flight batches (the *pipeline depth*) scales directly with the number of deployment IDs you configure:

```
pipeline_depth = number_of_script_ids  (clamped to 2..12)
```

| Deployments | Pipeline depth | Effective batch interval | Notes |
|-------------|---------------|------------------------|-------|
| 1 | 2 | ~1.0s | Minimum — still pipelines 2 batches |
| 3 | 3 | ~0.7s | Good for light browsing |
| 6 | 6 | ~0.3s | Recommended for daily use |
| 12 | 12 | ~0.17s | Maximum — diminishing returns past this |

More deployments = more concurrent batches = lower per-session latency. Each batch round-robins across your deployment IDs, so the load is spread evenly and you're less likely to hit a single deployment's quota ceiling.

**Resource guards** keep things safe:
- **50 ops max** per batch — if more sessions are active, the mux splits into multiple batches
- **4 MB payload cap** per batch — well under Apps Script's 50 MB limit
- **30 s timeout** per batch — a slow/dead target can't block other sessions forever

### Quick start

1. Deploy [`CodeFull.gs`](assets/apps_script/CodeFull.gs) as **3–12 Web App deployments** (same steps as `Code.gs`, but use the full-mode script that forwards to your tunnel-node). You can create multiple deployments on a single Google account — each "New deployment" produces its own ID. Going multi-account only matters for the daily quota (each Google account gets its own 20 000 `UrlFetchApp` calls/day on the free tier / 100 000 on Workspace); the pipeline depth itself scales fine on one account up to Apps Script's simultaneous-execution ceiling. Rule of thumb:
   - **Solo use** → 3–6 deployments on one account is plenty
   - **Shared with ~3 people** → 6 deployments on one account, bump to multi-account only if you start hitting quota alerts
   - **Shared with a group** → one account per heavy user (each with 1–2 deployments) is the clean scaling path
2. Deploy the [tunnel-node](tunnel-node/) on a VPS
3. Set `"mode": "full"` in your config with all deployment IDs:

```json
{
  "mode": "full",
  "script_id": ["id1", "id2", "id3", "id4", "id5", "id6"],
  "auth_key": "your-secret"
}
```

## Running on OpenWRT (or any musl distro)

The `*-linux-musl-*` archives ship a fully static CLI that runs on OpenWRT, Alpine, and any libc-less Linux userland. Put the binary on the router and start it as a service:

```sh
# From a machine that can reach your router:
scp mhrv-f root@192.168.1.1:/usr/bin/mhrv-f
scp mhrv-f.init root@192.168.1.1:/etc/init.d/mhrv-f
scp config.json root@192.168.1.1:/etc/mhrv-f/config.json

# On the router:
chmod +x /usr/bin/mhrv-f /etc/init.d/mhrv-f
/etc/init.d/mhrv-f enable
/etc/init.d/mhrv-f start
logread -e mhrv-f -f   # tail its logs
```

LAN devices then point their HTTP proxy at the router's LAN IP (default port `8085`) or their SOCKS5 at `<router-ip>:8086`. Set `listen_host` to `0.0.0.0` in `/etc/mhrv-f/config.json` so the router accepts LAN connections instead of localhost-only.

Memory footprint is ~15-20 MB resident — fine on anything with ≥128 MB RAM. No UI is shipped for musl (routers are headless).

## Diagnostics

- **`mhrv-f test`** — sends one request through the relay and reports success/latency. Use this first whenever something breaks — it isolates "relay is up" from "client config is wrong".
- **`mhrv-f doctor`** — guided first-run diagnostics. Checks config warnings, MITM CA trust, and runs the end-to-end relay probe, printing actionable fixes.
- **`mhrv-f rollback-config`** — restore the last-known-good config snapshot (saved automatically before UI overwrites config.json). Useful if you saved a broken config and can’t start.
- **`mhrv-f scan-ips`** — parallel TLS probe of 28 known Google frontend IPs, sorted by latency. Take the winner and put it in `google_ip`. The UI has the same thing behind the **scan** button next to the Google IP field.
- **`mhrv-f test-sni`** — parallel TLS probe of every SNI name in your rotation pool against the configured `google_ip`. Tells you which front-domain names actually pass through your ISP's DPI. The UI has the same thing in the **SNI pool…** floating window, with checkboxes, per-row **Test** buttons, and a **Keep ✓ only** button that auto-trims to what worked.
- **Periodic stats** are logged every 60 s at `info` level (relay calls, cache hit rate, bytes relayed, active vs. blacklisted scripts). The UI shows them live.

Docs:
- [Doctor guide (EN)](docs/doctor.md)
- [راهنمای دکتر (FA)](docs/doctor.fa.md)
- [راهنمای تکمیلی (فارسی)](docs/forum-cleaned.fa.md)
- [Optional future extensions](docs/optional-extensions.md)

### SNI pool editor

By default `mhrv-f` rotates through `{www, mail, drive, docs, calendar}.google.com` on outbound TLS connections to your Google IP, to avoid fingerprinting one name too heavily. Some of those may be locally blocked — e.g. `mail.google.com` has been specifically targeted in Iran at various times.

Either:

- Open the UI, click **SNI pool…**, hit **Test all**, then **Keep ✓ only** to auto-trim. Add custom names via the text field at the bottom. Save.
- Or edit `config.json` directly:

```json
{
  "sni_hosts": ["www.google.com", "drive.google.com", "docs.google.com"]
}
```

Leaving `sni_hosts` unset gives you the default auto-pool. Run `mhrv-f test-sni` to verify what works from your network before saving.

## Capabilities (what you can rely on)

`mhrv-f` is designed around one real-world goal: **make web browsing usable under heavy censorship, with the least possible setup friction**.

The core feature set:

- **Two local proxies**: HTTP proxy (CONNECT) and SOCKS5, both on localhost by default.
- **Apps Script relay** for non-Google domains, including **MITM** so browsers keep working normally.
- **Direct SNI-rewrite tunnel** for Google-owned domains (bypasses relay quota and avoids the fixed Apps Script User-Agent).
- **Multi-account pools** (`account_groups`) for quota resilience and faster recovery when one account or deployment is degraded.
- **Adaptive runtime tuning** (optional) + **failure-intelligent fallback** (automatic degradation + recovery).
- **Range-parallel downloads** (implemented) to make large media transfers viable when a single Apps Script fetch would stall.
- **Operational visibility**: local status endpoint + UI stats, including quota pressure counters and degradation reason.
- **Doctor + rollback**: `mhrv-f doctor` for guided checks and `mhrv-f rollback-config` to recover from a bad config.

Not in scope (by design):

- **HTTP/2 multiplexing in the relay path** (too many subtle hang cases for the incremental gain here).
- **Generic “domain fronting” modes** against arbitrary CDNs (mostly dead post-2024; this project targets Google’s edge specifically).

## Known limitations

These are inherent to the Apps Script + domain-fronting approach, not bugs in this client. The original Python version has the same issues.

- **User-Agent is fixed to `Google-Apps-Script`** for anything going through the relay. `UrlFetchApp.fetch()` does not allow overriding it. Consequence: sites that detect bots (e.g., Google search, some CAPTCHA flows) serve degraded / no-JS fallback pages to relayed requests. Workaround: add the affected domain to the `hosts` map so it's routed through the SNI-rewrite tunnel with your real browser's UA instead. `google.com`, `youtube.com`, `fonts.googleapis.com` are already there by default.
- **Video playback is slow and quota-limited** for anything that goes through the relay. YouTube HTML loads through the tunnel (fast), but chunks from `googlevideo.com` go through Apps Script. Each Apps Script consumer account has a ~2 M `UrlFetchApp` calls/day quota and a 50 MB body limit per fetch. Fine for text browsing, painful for 1080p. Rotate multiple `script_id`s for more headroom, or use a real VPN for video.
- **Brotli is stripped** from forwarded `Accept-Encoding` headers. Apps Script can decompress gzip, but not `br`, and forwarding `br` produces garbled responses. Minor size overhead.
- **WebSockets don't work** through the relay — it's single request/response JSON. Sites that upgrade to WS fail (ChatGPT streaming, Discord voice, etc.).
- **HSTS-preloaded / hard-pinned sites** will reject the MITM cert. Most sites are fine because the CA is trusted; a handful aren't.
- **Google / YouTube 2FA and sensitive logins** may trigger "unrecognized device" warnings because requests originate from Google's Apps Script IPs, not yours. Log in once via the tunnel (`google.com` is in the rewrite list) to avoid this.

## Security posture

- The MITM root stays **on your machine only**. The `ca/ca.key` private key is generated locally and never leaves the user-data dir.
- `auth_key` between the client and the Apps Script relay is a shared secret you pick. The server-side `Code.gs` rejects requests without a matching key.
- Traffic between your machine and Google's edge is standard TLS 1.3.
- What Google can see: the destination URL and headers of each request (because Apps Script fetches on your behalf). This is the same trust model as any hosted proxy — if that's not acceptable, use a self-hosted VPN instead.

## License

MIT. See [LICENSE](LICENSE).

## Credit

Original project: <https://github.com/masterking32/MasterHttpRelayVPN> by [@masterking32](https://github.com/masterking32). The idea, the Google Apps Script protocol, the proxy architecture, and the ongoing maintenance are all his. This Rust port exists purely to make client-side distribution easier.

## Support this project

If `mhrv-f` has been useful to you and you'd like to support continued development:

### [❤️ Support on sh1n.org](https://sh1n.org/donate)

Donations cover hosting, self-hosted CI runner costs, and continued maintenance. Starring the repo also helps signal that the project is worth keeping alive.

---

<div dir="rtl">

## راهنمای فارسی

### این ابزار چیست؟

یک پروکسی کوچک که روی سیستم خودتان اجرا می‌شود و ترافیک شما را از طریق یک اسکریپت رایگان که در حساب گوگل خودتان می‌سازید، عبور می‌دهد. `ISP` شما فقط یک اتصال `HTTPS` ساده به `www.google.com` می‌بیند و اجازه می‌دهد رد شود؛ در پشت پرده، اسکریپتی که خودتان منتشر می‌کنید سایت مقصد را برای شما می‌خواند و پاسخ را بازمی‌گرداند.

**MasterHttpRelayVPN-Frankestein** همان خط یکپارچهٔ **mhrv-f** است: **اعتبار ایده‌ی اصلی رله و `Code.gs`ی مرجع** برای [@masterking32](https://github.com/masterking32) و پروژهٔ [MasterHttpRelayVPN](https://github.com/masterking32/MasterHttpRelayVPN) است. شما اینجا یک کلایント آماده (دسکتاپ و اندروید) برای همان الگو دریافت می‌کنید.

باگ یا پیشنهاد: [issues](https://github.com/AmirrezaFarnamTaheri/MasterHttpRelayVPN-Frankestein/issues).

### برای چه کسی مفید است؟

- کسانی که در شبکه‌های تحت سانسور قوی (مثل ایران) زندگی می‌کنند
- کسی که می‌خواهد بدون `VPN` تجاری، بدون نصب پایتون، و بدون پرداخت پول عبور کند
- کسی که حتی یک حساب گوگل رایگان دارد

### چه چیز لازم دارید؟

۱. یک حساب گوگل (همان `Gmail` رایگان کافیست)  
۲. مرورگر (`Firefox`، `Chrome`، `Edge`، …) یا برنامه‌ای که `HTTP proxy` یا `SOCKS5` قبول کند  
۳. دسترسی به سیستم خودتان (مک / لینوکس / ویندوز)  

### پنج مرحله برای راه‌اندازی

#### مرحلهٔ ۱ — ساخت اسکریپت در گوگل (فقط یک بار)

۱. به <https://script.google.com> بروید و با حساب گوگل خودتان وارد شوید  
۲. روی **`New project`** کلیک کنید و کد پیش‌فرض را پاک کنید  
۳. محتوای [`assets/apps_script/Code.gs`](assets/apps_script/Code.gs) را از همین ریپو کپی کنید و در ویرایشگر Apps Script بچسبانید (راهنمای [`assets/apps_script/README.md`](assets/apps_script/README.md)).  
۴. بالای کد، خط `const AUTH_KEY = "..."` را پیدا کنید و مقدار آن را به یک رمز قوی و خاص خودتان تغییر دهید (یک رشتهٔ تصادفی حداقل ۱۶ کاراکتری کافی است، مثلاً `aK8f3xM9pQ2nL5vR`)  
۵. روی دکمهٔ آبی **`Deploy`** در بالا سمت راست کلیک کنید و **`New deployment`** را بزنید  
۶. **`Type`** را روی **`Web app`** بگذارید و این تنظیمات را اعمال کنید:  
- **`Execute as`**: **`Me`**  
- **`Who has access`**: **`Anyone`**

۷. روی **`Deploy`** کلیک کنید. گوگل یک **`Deployment ID`** نشان می‌دهد — رشتهٔ طولانی تصادفی که داخل آدرس `URL` است. کپی‌اش کنید؛ در برنامه لازم دارید  

> **نکته:** اگر نمی‌دانید رمز `AUTH_KEY` چه بگذارید، یک رشتهٔ تصادفی ۱۶ تا ۲۴ کاراکتری بسازید. مهم فقط این است که **دقیقاً همان رشته** را در برنامه هم وارد کنید.

#### به `script.google.com` هم دسترسی ندارید؟

اگر `ISP` شما از قبل `Apps Script` (یا کل گوگل) را مسدود کرده، برای مرحلهٔ ۱ باید مرورگرتان **اول** به `script.google.com` برسد — قبل از اینکه رله‌ای داشته باشید. `mhrv-f` یک حالت بوت‌استرپ کوچک دقیقاً برای همین دارد: `google_only`.

۱. برنامه را طبق مرحلهٔ ۲ پایین دانلود کنید

۲. فایل [`config.google-only.example.json`](config.google-only.example.json) را در کنار فایل اجرایی به نام `config.json` کپی کنید — نه `script_id` لازم دارد و نه `auth_key`

۳. برنامه را اجرا کنید و `HTTP proxy` مرورگرتان را روی `127.0.0.1:8085` تنظیم کنید

۴. در حالت `google_only`، پروکسی فقط `*.google.com`، `*.youtube.com` و بقیهٔ میزبان‌های لبهٔ گوگل را از طریق همان تونل بازنویسی `SNI` رد می‌کند. بقیهٔ ترافیک مستقیم می‌رود — هنوز رله‌ای در کار نیست

۵. حالا مرحلهٔ ۱ را در مرورگر انجام دهید (اتصال به `script.google.com` با `SNI` فرونت می‌شود). `Code.gs` را مستقر کنید و `Deployment ID` را کپی کنید

۶. در `UI` دسکتاپ یا اندروید (یا با ویرایش `config.json`) حالت را به `apps_script` برگردانید، `Deployment ID` و `auth_key` را بچسبانید و برنامه را دوباره راه‌اندازی کنید

برای بررسی قابلیت دسترسی قبل از راه‌اندازی پروکسی: دستور `mhrv-f test-sni` دامنه‌های `*.google.com` را مستقیماً تست می‌کند و فقط به `google_ip` و `front_domain` نیاز دارد.

#### مرحلهٔ ۲ — دانلود برنامه

به [صفحهٔ Releases](https://github.com/AmirrezaFarnamTaheri/MasterHttpRelayVPN-Frankestein/releases) بروید و آرشیو مناسب سیستم‌عامل خود را دانلود و از حالت فشرده خارج کنید:

| سیستم‌عامل | فایل مناسب |
|---|---|
| مک اپل‌سیلیکون (`M1` / `M2` / …) | `mhrv-f-macos-arm64-app.zip` (قابل دوبار کلیک در `Finder`) |
| مک اینتل | `mhrv-f-macos-amd64-app.zip` |
| ویندوز | `mhrv-f-windows-amd64.zip` |
| لینوکس معمولی (اوبونتو، مینت، دبیان، فدورا، آرچ، …) | `mhrv-f-linux-amd64.tar.gz` |
| لینوکس روی روتر (`OpenWRT`) یا `Alpine` | `mhrv-f-linux-musl-amd64.tar.gz` |

> اگر نمی‌دانید مک شما `M1/M2` است یا اینتل: منوی اپل → `About This Mac` → در خط **`Chip`** اگر **`Apple`** نوشته شده، `arm64` بگیرید؛ اگر **`Intel`**، `amd64`.  

> کاربران اوبونتو ۲۰.۰۴ یا سیستم‌های خیلی قدیمی که خطای `GLIBC not found` می‌گیرند: آرشیو `linux-musl-amd64` را دانلود کنید — اجرا می‌شود.  
#### مرحلهٔ ۳ — اجرای بار اول (نصب گواهی محلی)

برای اینکه برنامه بتواند ترافیک `HTTPS` مرورگر شما را باز کند و از طریق `Apps Script` رد کند، یک گواهی امنیتی کوچک **روی سیستم خودتان** می‌سازد و به سیستم‌عامل می‌گوید به آن اعتماد کند.

**کاری که باید بکنید (خودکار است):**

| سیستم‌عامل | روش |
|---|---|
| مک | روی `run.command` دو بار کلیک کنید |
| ویندوز | روی `run.bat` دو بار کلیک کنید |
| لینوکس | در ترمینال دستور `./run.sh` را اجرا کنید |

**فقط یک بار** رمز سیستم (`sudo` در مک/لینوکس یا `UAC` در ویندوز) می‌خواهد تا گواهی را نصب کند. بعد از آن برنامه باز می‌شود و در اجراهای بعدی می‌توانید مستقیماً از فایل اصلی (`mhrv-f.app` در مک، `mhrv-f-ui.exe` در ویندوز) استفاده کنید.

**امنیت این گواهی:**

- گواهی **کاملاً روی سیستم شما** ساخته می‌شود. کلید خصوصی هیچ‌وقت از کامپیوترتان خارج نمی‌شود
- هیچ سرور راه دوری — از جمله خود گوگل — نمی‌تواند با این گواهی خودش را جای سایت‌ها جا بزند
- هر وقت خواستید می‌توانید گواهی را حذف کنید (بخش **[حذف گواهی](#سوالات-رایج)** را ببینید)

> **اگر نمی‌خواهید از اسکریپت راه‌انداز استفاده کنید**، می‌توانید مرحلهٔ گواهی را دستی انجام دهید:
>
> - مک/لینوکس: `sudo ./mhrv-f --install-cert`
> - ویندوز (با `Run as administrator`): `mhrv-f.exe --install-cert`

#### مرحلهٔ ۴ — تنظیمات در برنامه

پنجرهٔ برنامه باز می‌شود. این فیلدها را پر کنید:

| فیلد | مقدار |
|---|---|
| **`Apps Script ID(s)`** | همان `Deployment ID` مرحلهٔ ۱ را paste کنید |
| **`Auth key`** | همان رمز `AUTH_KEY` که داخل `Code.gs` گذاشتید |
| **`Google IP`** | پیش‌فرض `216.239.38.120` معمولاً خوب است. دکمهٔ `scan` کنارش IPهای دیگر گوگل را تست می‌کند و سریع‌ترین را نشان می‌دهد |
| **`Front domain`** | پیش‌فرض `www.google.com` را نگه دارید |
| **`HTTP port`** / **`SOCKS5 port`** | پیش‌فرض‌های `8085` و `8086` خوب‌اند |

بعد روی **`Save config`** و سپس **`Start`** کلیک کنید. هر وقت خواستید وضعیت را تست کنید، دکمهٔ **`Test`** را بزنید — یک درخواست کامل می‌فرستد و نتیجه را نشان می‌دهد.

#### مرحلهٔ ۵ — تنظیم مرورگر یا اپلیکیشن

برنامه روی دو پورت منتظر است:

- **`HTTP proxy`** روی `127.0.0.1:8085` — برای مرورگرها
- **`SOCKS5 proxy`** روی `127.0.0.1:8086` — برای تلگرام / `xray` / بقیهٔ اپلیکیشن‌ها

**فایرفاکس (ساده‌ترین):**


#### پیکربندی scan-ips (اختیاری)
به‌طور پیش‌فرض، دستور scan-ips از آرایه‌ای ثابت از IPها استفاده می‌کند.

می‌توانید کشف پویای IP را با تنظیم fetch_ips_from_api روی true در config.json فعال کنید:

```json
{
  "fetch_ips_from_api": true,
  "max_ips_to_scan": 100,
  "scan_batch_size": 100,
  "google_ip_validation": true
}
```

زمانی که فعال باشد:

- فایل goog.json را از API محدوده‌های عمومی IP گوگل دریافت می‌کند
تمام CIDRها را استخراج کرده و به IPهای جداگانه تبدیل می‌کند
- به IPهای دامنه‌های معروف گوگل (google.com، youtube.com و غیره) اولویت می‌دهد
- به‌صورت تصادفی تا max_ips_to_scan کاندید انتخاب می‌کند (ابتدا IPهای اولویت‌دار)
- فقط کاندیدهای انتخاب‌شده را برای اتصال و اعتبارسنجی frontend تست می‌کند. `google_ip_validation` هدرهای پاسخ را بررسی می‌کند تا مطمئن شود IP مثل یک frontend قابل استفاده گوگل رفتار می‌کند.

با استفاده از این گزینه‌ها ممکن است IPهایی پیدا کنید که سریع‌تر از آرایه ثابت پیش‌فرض هستند اما تضمینی وجود ندارد که این IPها کار کنند.

#### ۵. تنظیم proxy در کلاینت
۱. منوی `Settings` را باز کنید، در خانهٔ جست‌وجو عبارت `proxy` را تایپ کنید  
۲. روی **`Network Settings`** کلیک کنید  
۳. گزینهٔ **`Manual proxy configuration`** را انتخاب کنید  
۴. در فیلد **`HTTP Proxy`** آدرس `127.0.0.1` و پورت `8085` را بگذارید  
۵. تیک **`Also use this proxy for HTTPS`** را بزنید  
۶. `OK`  
**کروم یا Edge:** از تنظیمات `proxy` سیستم‌عامل استفاده می‌کنند. ساده‌ترین راه نصب افزونهٔ **`Proxy SwitchyOmega`** و تنظیم آن روی `127.0.0.1:8085` است.

**تلگرام:**

۱. `Settings` → `Advanced` → `Connection type`
۲. **`Use custom proxy`** → **`SOCKS5`**
۳. هاست `127.0.0.1`، پورت `8086`، نام کاربری و رمز را خالی بگذارید
۴. `Save` بزنید

> **نکتهٔ مهم دربارهٔ تلگرام:** اگر فقط این ابزار را استفاده کنید، تلگرام ممکن است مرتب قطع و وصل شود، چون `Apps Script` پروتکل `MTProto` تلگرام را نمی‌فهمد. برای پایداری کامل تلگرام، بخش [**تلگرام پایدار با xray**](#تلگرام-و-غیره--جفت-کردن-با-xray) را ببینید.

### از کجا بفهمم کار می‌کند؟

۱. در پنجرهٔ برنامه، وضعیت باید **`Status: running`** باشد (سبز رنگ)
۲. دکمهٔ **`Test`** را بزنید — اگر سبز شد، سرویس سالم است
۳. در مرورگر به <https://icanhazip.com> بروید — `IP` نمایش داده‌شده باید متفاوت از `IP` واقعی شما باشد (آی‌پی گوگل)
۴. اگر مشکلی بود، پنل **`Recent log`** پایین برنامه را نگاه کنید

### تلگرام و غیره — جفت کردن با xray

‏ `Apps Script` فقط `HTTP` می‌فهمد، پس پروتکل‌های دیگر (مثل `MTProto` تلگرام، `IMAP` ایمیل، `SSH`، …) مستقیماً از آن رد نمی‌شوند. نتیجه: اگر `ISP` تلگرام را با `DPI` بلاک کرده باشد، همچنان بلاک است.  
**راه‌حل:** یک [`xray`](https://github.com/XTLS/Xray-core) (یا `v2ray` یا `sing-box`) روی سیستم خودتان اجرا کنید که با `VLESS` / `Trojan` / `Shadowsocks` به یک سرور `VPS` شخصی وصل می‌شود. بعد در برنامهٔ `mhrv-f`، فیلد **`Upstream SOCKS5`** را با آدرس `xray` پر کنید (مثلاً `127.0.0.1:50529`).

بعد از این کار، ترافیکی که `HTTP` نیست (مثل تلگرام) از `xray` عبور می‌کند و به سرور شما می‌رسد. ترافیک `HTTP/HTTPS` مثل قبل از `Apps Script` می‌رود، پس مرورگر شما دست نخورده کار می‌کند.

```json
{
  "upstream_socks5": "127.0.0.1:50529"
}
```

### ویرایشگر SNI pool

به‌صورت پیش‌فرض برنامه بین چند نام گوگل می‌چرخد (`www.google.com`، `mail.google.com`، `drive.google.com`، `docs.google.com`، `calendar.google.com`) تا اثر انگشت ترافیک شما یکنواخت نباشد. اما بعضی از این نام‌ها گاهی در شبکهٔ شما بلاک می‌شوند — مثلاً `mail.google.com` در ایران چند بار هدف قرار گرفته.

**برای بررسی و ویرایش:**

۱. روی دکمهٔ آبی **`SNI pool…`** در برنامه کلیک کنید
۲. دکمهٔ **`Test all`** را بزنید — هر نام را تست می‌کند و نتیجه را کنارش نشان می‌دهد (`ok` یا `fail`)
۳. دکمهٔ **`Keep working only`** را بزنید — همه نام‌هایی که پاسخ ندادند را غیرفعال می‌کند
۴. اگر نام جدیدی می‌خواهید اضافه کنید، در کادر پایین نام را بنویسید و **`+ Add`** بزنید — خودکار تست می‌شود
۵. با **`Save config`** در پنجرهٔ اصلی ذخیره کنید

### حالت تونل کامل (Full tunnel mode)

حالت `"mode": "full"` **تمام** ترافیک را سرتاسر از طریق `Apps Script` و یک [tunnel-node](tunnel-node/) روی سرور شما عبور می‌دهد — **بدون نیاز به نصب گواهی `MITM`**. تنها هزینه‌اش تأخیر بیشتر است (هر بایت از مسیر `Apps Script → tunnel-node → مقصد` می‌رود)، اما برای هر پروتکل و هر برنامه بدون نصب `CA` کار می‌کند.

#### چرا تعداد `Deployment ID` مهم است؟

هر درخواست دسته‌ای (`batch`) به `Apps Script` حدود ۲ ثانیه طول می‌کشد. در حالت `full`، برنامه یک **لولهٔ موازی** (`pipeline`) اجرا می‌کند که چند درخواست دسته‌ای را همزمان می‌فرستد بدون اینکه منتظر پاسخ قبلی بماند. تعداد درخواست‌های همزمان مستقیماً با تعداد `Deployment ID`ها رابطه دارد:

```
عمق لوله = تعداد Deployment IDها  (حداقل ۲، حداکثر ۱۲)
```

| تعداد Deployment | عمق لوله | فاصلهٔ مؤثر بین دسته‌ها | |
|-----------------|----------|------------------------|---|
| ۱ | ۲ | ~۱ ثانیه | حداقل |
| ۳ | ۳ | ~۰.۷ ثانیه | مناسب مرور سبک |
| ۶ | ۶ | ~۰.۳ ثانیه | توصیه‌شده برای استفادهٔ روزانه |
| ۱۲ | ۱۲ | ~۰.۱۷ ثانیه | حداکثر |

بیشتر `Deployment` = بیشتر درخواست همزمان = تأخیر کمتر برای هر نشست. هر دسته بین `ID`ها چرخش می‌کند (`round-robin`)، پس بار به‌طور یکنواخت توزیع می‌شود.

### اجرا روی OpenWRT (روتر)

اگر می‌خواهید برنامه را روی روترتان اجرا کنید تا همهٔ دستگاه‌های شبکه از آن استفاده کنند، آرشیو `mhrv-f-linux-musl-*.tar.gz` را دانلود کنید (این نسخه فایل اجرایی استاتیک دارد و بدون نصب هیچ وابستگی روی روتر کار می‌کند).

```sh
# از کامپیوتری که به روترتان دسترسی دارد:
scp mhrv-f root@192.168.1.1:/usr/bin/mhrv-f
scp mhrv-f.init root@192.168.1.1:/etc/init.d/mhrv-f
scp config.json root@192.168.1.1:/etc/mhrv-f/config.json

# روی خود روتر (ssh کنید به روتر):
chmod +x /usr/bin/mhrv-f /etc/init.d/mhrv-f
/etc/init.d/mhrv-f enable
/etc/init.d/mhrv-f start
logread -e mhrv-f -f
```

در فایل `config.json`، مقدار `listen_host` را به `0.0.0.0` تغییر دهید تا روتر از همهٔ دستگاه‌های `LAN` اتصال بپذیرد. بعد در هر دستگاه، `HTTP proxy` را روی آی‌پی روتر پورت `8085` (یا `SOCKS5` روی `8086`) تنظیم کنید.

مصرف حافظه حدود ۱۵ تا ۲۰ مگابایت است — روی هر روتری با حداقل ۱۲۸ مگابایت `RAM` اجرا می‌شود.

### اجرا روی اندروید

یک نسخهٔ اندروید هم داریم — همان `mhrv-f` ولی داخل یک برنامهٔ `Compose` با پل `TUN` از طریق [`tun2proxy`](https://crates.io/crates/tun2proxy). تمام ترافیک دستگاه (مرورگر، تلگرام، هر برنامه‌ای) خودکار از پروکسی رد می‌شود، بدون نیاز به تنظیم per-app.

**دانلود:** `mhrv-f-android-universal-v*.apk` از [صفحهٔ Releases](https://github.com/AmirrezaFarnamTaheri/MasterHttpRelayVPN-Frankestein/releases/latest) (یک APK جهانی، روی اندروید ۷.۰ و بالاتر، همهٔ معماری‌ها).

**راهنمای کامل فارسی:** [**`docs/android.fa.md`**](docs/android.fa.md) — نصب APK، دیپلوی `Apps Script`، تست `SNI`، نصب گواهی `MITM`، رفع اشکال و محدودیت‌ها.

راهنمای انگلیسی هم در [`docs/android.md`](docs/android.md) است.

جمع‌بندی سریع:

۱‏. APK را از `Releases` دانلود و نصب کنید (اگر اندروید «منبع ناشناس» گفت، در همان دیالوگ اجازه بدهید)  
۲‏. `Apps Script` را طبق [مرحلهٔ ۱ بالا](#مرحلهٔ-۱--ساخت-اسکریپت-در-گوگل-فقط-یک-بار) دیپلوی کنید (همان `Code.gs` + `AUTH_KEY`)  
۳‏. `/exec URL` و `auth_key` را در برنامه وارد کنید، **Auto-detect google_ip** را بزنید  
۴‏. **Install MITM certificate** — برنامه گواهی را در `Downloads` ذخیره می‌کند و `Settings` را باز می‌کند. در `Settings` عبارت `CA certificate` را جست‌وجو و از `Downloads` نصب کنید  
۵‏. **Start** → مجوز `VPN` را تأیید کنید → همه‌چیز کار می‌کند  


محدودیت‌های اندروید همان محدودیت‌های دسکتاپ + دو مورد اضافه: `IPv6` از `TUN` رد نمی‌شود (فقط `IPv4` روت می‌شود) و اکثر برنامه‌های غیر مرورگری (بانکی، `Netflix`، پیام‌رسان‌ها) به `CA` کاربری اعتماد نمی‌کنند. جزئیات در [`docs/android.fa.md`](docs/android.fa.md#محدودیت‌های-شناخته‌شده).

### سوالات رایج

**چرا باید گواهی نصب کنم؟ امن است؟**
برنامه برای اینکه بتواند ترافیک `HTTPS` شما را باز کند و از طریق `Apps Script` رد کند، به یک گواهی محلی نیاز دارد. این گواهی **فقط روی سیستم خودتان** ساخته می‌شود و کلید خصوصی هیچ‌وقت جایی ارسال نمی‌شود. هیچ کس — حتی خود گوگل — نمی‌تواند با این گواهی به ترافیک شما دسترسی پیدا کند.

**چطور گواهی را بعداً حذف کنم؟**

- **مک:** `Keychain Access` را باز کنید، در بخش `System` دنبال `mhrv-f` بگردید و حذف کنید. سپس پوشهٔ `~/Library/Application Support/mhrv-f/ca/` را پاک کنید
- **ویندوز:** `certmgr.msc` را اجرا کنید → `Trusted Root Certification Authorities` → `Certificates` → دنبال `mhrv-f` بگردید و حذف کنید
- **لینوکس:** فایل `/usr/local/share/ca-certificates/mhrv-f.crt` را حذف و `sudo update-ca-certificates` اجرا کنید

**چند `Deployment ID` لازم دارم؟**
یکی برای استفادهٔ عادی کافی است. سهمیهٔ روزانه `UrlFetchApp` برای حساب رایگان گوگل **۲۰٬۰۰۰ درخواست در روز** است (برای `Workspace` پولی ۱۰۰٬۰۰۰)، با محدودیت پاسخ ۵۰ مگابایت به ازای هر `fetch`. برای اکثر کاربران چند ساعت یوتیوب هم با یک `Deployment` کافی است. می‌توانید چند `Deployment` **در همان حساب** بسازید (هر بار `New deployment` یک `ID` جدید می‌دهد) — این روش در حالت `full` پهنای باند بهتری می‌دهد چون `pipeline depth` افزایش پیدا می‌کند و هر `Deployment` یک اجرای همزمان جدا در `Apps Script` می‌گیرد (تا سقف ۳۰ اجرای همزمان هر حساب). برای سهمیهٔ روزانهٔ بیشتر، در حساب‌های گوگل دیگر هم `Deployment` بسازید — هر حساب سهمیهٔ ۲۰ هزار درخواستی خودش را دارد. همهٔ `ID`ها را در فیلد `Apps Script ID(s)` وارد کنید — برنامه خودکار بینشان می‌چرخد. مرجع: <https://developers.google.com/apps-script/guides/services/quotas>

**یوتوب کار می‌کند؟ ویدیو پخش می‌شود؟**
صفحهٔ یوتوب سریع باز می‌شود (چون مستقیم از لبهٔ گوگل می‌آید). اما `chunk`های ویدیوی اصلی از `googlevideo.com` از طریق `Apps Script` می‌آیند و روزانه سهمیه دارند. برای تماشای گاه‌به‌گاه خوب است، برای ۱۰۸۰p پخش طولانی دردناک.

**‏`ChatGPT` یا `OpenAI` کار می‌کنند؟**
استریم زنده (`streaming`) آن‌ها کار نمی‌کند چون از `WebSocket` استفاده می‌کنند و `Apps Script` آن را پشتیبانی نمی‌کند. تنها راه‌حل: از `xray` استفاده کنید (بخش **تلگرام و غیره** را ببینید).

**خطای `GLIBC_2.39 not found` در لینوکس می‌گیرم. چه کنم؟**
از نسخهٔ `v0.7.1` به بعد این مشکل حل شده. اما اگر روی سیستم خیلی قدیمی هستید، آرشیو `mhrv-f-linux-musl-amd64.tar.gz` را دانلود کنید — این نسخه بدون نیاز به `glibc` روی هر لینوکسی اجرا می‌شود.

**می‌توانم با `CLI` هم استفاده کنم (بدون رابط گرافیکی)؟**
بله. فایل `config.example.json` را به `config.json` کپی کنید، مقادیر را پر کنید، و این دستورات را بزنید:

```bash
./mhrv-f                   # اجرای پروکسی
./mhrv-f test              # تست یک درخواست کامل
./mhrv-f scan-ips          # رتبه‌بندی IPهای گوگل بر اساس سرعت
./mhrv-f test-sni          # تست نام‌های SNI در pool
./mhrv-f --install-cert    # نصب مجدد گواهی
./mhrv-f --help
```

**چرا گاهی جست‌وجوی گوگل بدون `JavaScript` نشان داده می‌شود؟**
`Apps Script` مجبور است `User-Agent` درخواست‌های خود را روی `Google-Apps-Script` بگذارد. بعضی سایت‌ها این را به عنوان ربات شناسایی می‌کنند و نسخهٔ سادهٔ بدون `JavaScript` برمی‌گردانند. دامنه‌هایی که در لیست `SNI-rewrite` قرار گرفته‌اند (مثل `google.com`، `youtube.com`) از این مشکل در امان هستند چون مستقیماً از لبهٔ گوگل می‌آیند، نه از `Apps Script`.

**ورود به حساب گوگل با این ابزار ایمن است؟**
توصیه می‌شود اولین بار بدون این پروکسی یا با `VPN` واقعی وارد شوید، چون گوگل ممکن است `IP` `Apps Script` را به‌عنوان «دستگاه ناشناس» ببیند و هشدار بدهد. بعد از ورود اولیه، استفاده بی‌مشکل است.

### محدودیت‌های شناخته‌شده

این محدودیت‌ها ذاتی روش `Apps Script` هستند، نه باگ این برنامه. نسخهٔ اصلی پایتون هم دقیقاً همین محدودیت‌ها را دارد.

- ‏`User-Agent` همهٔ درخواست‌ها ثابت روی `Google-Apps-Script` است (گوگل اجازهٔ تغییر نمی‌دهد). بعضی سایت‌ها به‌خاطر این نسخهٔ ساده‌شدهٔ بدون `JavaScript` نشان می‌دهند  
- پخش ویدیو سهمیه دارد و ممکن است کند باشد (سهمیهٔ `UrlFetchApp` برای حساب رایگان ۲۰٬۰۰۰ درخواست در روز است — چند ساعت یوتیوب برای بیشتر کاربران)  
- فشرده‌سازی `Brotli` پشتیبانی نمی‌شود (فقط `gzip`)، سربار حجمی جزئی  
- ‏`WebSocket` از `Apps Script` عبور نمی‌کند (`ChatGPT` استریم، `Discord voice`، …)  
- سایت‌هایی که گواهی خود را `pin` کرده‌اند گواهی `MITM` برنامه را قبول نمی‌کنند (تعداد کمی‌اند)  
- ورود دومرحله‌ای گوگل ممکن است هشدار «دستگاه ناشناس» بدهد — اولین ورود را بدون این ابزار انجام دهید  
### امنیت

- ریشهٔ `MITM` **فقط روی سیستم شما می‌ماند**. کلید خصوصی هیچ‌وقت از سیستمتان خارج نمی‌شود
- `auth_key` یک رمز اختصاصی بین شما و اسکریپت شماست. کد سرور هر درخواستی را که این رمز را نداشته باشد رد می‌کند
- ترافیک بین شما و گوگل، `TLS 1.3` استاندارد است
- آنچه گوگل می‌بیند: آدرس `URL` و هدرهای درخواست شما (چون `Apps Script` به‌جای شما `fetch` می‌کند). این همان سطح اعتماد هر پروکسی میزبانی‌شده است — اگر قابل قبول نیست، از `VPN` روی سرور شخصی خودتان استفاده کنید

### اعتبار

پروژهٔ اصلی: <https://github.com/masterking32/MasterHttpRelayVPN> توسط [@masterking32](https://github.com/masterking32). ایده، پروتکل `Apps Script`، و معماری پروکسی همه متعلق به ایشان است. این پورت `Rust` فقط برای ساده‌تر کردن توزیع سمت کلاینت درست شده.

### حمایت از پروژه

اگر `mhrv-f` برای شما مفید بوده و می‌خواهید از ادامهٔ توسعه حمایت کنید:

### [❤️ حمایت در sh1n.org](https://sh1n.org/donate)

کمک‌ها صرف هزینه‌های میزبانی، سرور `CI` اختصاصی، و ادامهٔ نگهداری پروژه می‌شود. ستاره دادن به ریپو هم یک راه رایگان برای نشان دادن اینه که پروژه ارزش ادامه دادن داره.

</div>
