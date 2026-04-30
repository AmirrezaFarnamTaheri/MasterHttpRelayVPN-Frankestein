# Desktop UI Reference

`mhrv-f-ui` is the recommended first-run surface. It writes the same
`config.json` used by the CLI, starts/stops the local proxy, runs diagnostics,
and shows live logs.

## Main Actions

- **Top control panel**: shows the active mode, HTTP/SOCKS ports, LAN exposure
  state, runtime profile, run state, release link, and the most common actions.
- **Start / Stop**: run or stop the local proxy engine (`mhrv-f serve`).
- **Test relay**: run `mhrv-f test` for `apps_script` or `vercel_edge`.
- **Doctor**: run guided diagnostics (`mhrv-f doctor`).
- **Save config**: write the current form to the app config file.
- **Walkthrough**: reopen the first-run wizard after it has been hidden.
- **Doctor + Fix**: run diagnostics, apply safe local fixes, then run
  diagnostics again.
- **Install CA / Remove CA / Check CA**: generate/install the local MITM CA,
  remove OS/browser trust when uninstalling, and verify trust status. Stop the
  proxy before removing the CA.

## Tabs

The main panel is split into tabs so the UI does not become one dense vertical
form:

- **Setup**: mode picker, mode summary, Apps Script/serverless credentials, and
  backend/tool recipes. Start here when creating or changing a deployment.
- **Network**: Google edge IP, front domain, listener host/ports, LAN sharing,
  copyable HTTP/SOCKS endpoints, LAN token, and allowed IP/CIDR guardrails.
- **Advanced**: tuning knobs, upstream SOCKS5 chain, account groups, save
  config, and profiles. Open this only when setup already works or when you are
  adding quota/capacity.
- **Monitor**: traffic counters, dashboard, per-site stats, Test/Doctor,
  certificate/update status, and the Recent log panel.
- **Help & docs**: first-run explanations, mode decision cards, backend
  warnings, advanced option explanations, and trust/security notes.

The top Start/Stop/Test/Doctor/Save controls stay visible above the tabs. That
keeps emergency actions one click away while the detailed controls stay grouped
by task.

## First-Run Wizard

The wizard walks through four steps:

1. **Mode**: choose Apps Script, serverless JSON, or Full tunnel.
2. **Relay**: enter Apps Script account groups or serverless Base URL/auth key.
3. **CA**: install/check the local CA for modes that decrypt HTTPS locally.
4. **Diagnostics**: run Doctor and Test relay.

You can hide the wizard after setup. It can be re-opened by using a fresh config
or resetting the UI state.

## Mode Field

`mode` controls which sections are required:

- `apps_script`: classic relay through Google Apps Script. Requires account
  groups and local CA trust.
- `vercel_edge`: native serverless JSON relay. Works with the bundled Vercel or
  Netlify JSON tool. Requires Base URL, relay path, auth key, and local CA
  trust.
- `direct`: bootstrap mode for Google-owned hosts. No relay credentials.
- `full`: tunnel-node mode. Requires full-tunnel infrastructure; local MITM CA
  is not used.

The UI also shows a compact mode summary panel with:

- **Selected**: the active mode label.
- **Path**: the traffic path in plain language.
- **Setup**: the backend or credentials needed before Start.
- **Trust**: whether local CA trust is required.

Use this panel as the source of truth when switching modes. If the panel says a
VPS or serverless function is needed, deploy that component before expecting
Start/Test to pass.

## Backend Tools Section

The **Backend tools and deployment recipes** section separates native modes from
helper tools:

- **Apps Script Code.gs**: default backend for `apps_script`.
- **Cloudflare Worker exit**: optional Apps Script-compatible exit path using
  `CodeCloudflareWorker.gs`.
- **Vercel Edge JSON**: backend for native `vercel_edge`.
- **Netlify Edge JSON**: same native JSON protocol on Netlify Edge; use the
  Netlify site URL as Base URL.
- **Vercel XHTTP helper**: external Xray/V2Ray helper for a Vercel front, not a
  native UI mode.
- **Netlify XHTTP helper**: external Xray/V2Ray helper for a Netlify front, not
  a native UI mode.
- **Field notes**: cleaned candidate lists for Google SNI, Vercel edge names,
  and Netlify/Fastly/CloudFront external-client experiments.
- **tunnel-node**: VPS/server component for `full`.

This section is intentionally visible even when a tool is not selected. It
prevents mixing setup instructions, such as deploying an XHTTP helper and then
expecting it to work as `vercel_edge`.

Each row has an **open** action for the local script, tool folder, or guide.
Use those buttons as the shortest safe path from a selected mode to the exact
file you need to deploy: `Code.gs` for Apps Script, the JSON relay folders for
native `vercel_edge`, the XHTTP folders for external Xray/V2Ray, and
`tunnel-node` for full mode.

## Serverless JSON Relay Section

Visible when `mode = "vercel_edge"`.

- **Base URL** -> `vercel.base_url`
  - Examples: `https://your-project.vercel.app`,
    `https://your-site.netlify.app`
  - Use only the origin, not `/api/api`.
- **Relay path** -> `vercel.relay_path`
  - Default: `/api/api`
- **Auth key** -> `vercel.auth_key`
  - Must match the Vercel or Netlify environment variable `AUTH_KEY`.
- **Verify TLS certificate** -> `vercel.verify_tls`
  - Keep enabled unless debugging a known local TLS interception issue.
- **Max body MB** -> `vercel.max_body_bytes`
  - Guardrail for a single client request body sent through JSON/base64.
  - Default: `4` MiB. Raise it only when a real upload or large POST fails at
    the relay size guardrail.

Important: platform protection pages are HTML; the native client expects JSON.
For Vercel, disable Deployment Protection for the relay domain. For Netlify,
confirm `/api/api` reaches the Edge Function and returns the JSON health body.

## Apps Script Relay Credentials

Modern configs use account groups:

- **Account groups** -> `account_groups[]`
- Each group has:
  - `label`
  - `enabled`
  - `weight`
  - `auth_key`
  - `script_ids`

One group usually represents one Google account/quota pool. The group's
`auth_key` is shared by every deployment ID in that group. Multiple deployment
IDs inside one group rotate/fail over within the same account pool; multiple
groups represent separate account pools and can be weighted for capacity or
kept as backups. Keep this distinction clear so a key mismatch in one account
does not look like a global client failure.

## Network Fields

- **Google IP** -> `google_ip`
  - Google edge IPv4 used for SNI-fronted connections.
- **Front domain** -> `front_domain`
  - Usually `www.google.com`; keep this unless you are deliberately testing
    another SNI.
- **HTTP port** -> `listen_port`
- **SOCKS5 port** -> `socks5_port`
- **Listen host** -> `listen_host`
  - Keep `127.0.0.1` unless sharing to LAN intentionally.

## Sharing And Per-App Routing

The desktop UI includes a dedicated **Sharing and per-app routing** section.

- **Local only** sets `listen_host = "127.0.0.1"`.
- **Share on LAN** sets `listen_host = "0.0.0.0"`.
- The UI shows copyable HTTP and SOCKS endpoint strings. In local-only mode the
  host is `127.0.0.1`; in LAN mode it reminds you to replace the host with the
  desktop's actual LAN IP.
- **LAN token** writes `lan_token` for HTTP clients that can send the
  `X-MHRV-F-Token` header.
- **Allowed IPs** writes `lan_allowlist`; use one exact IP or CIDR per line.

SOCKS5 cannot carry the HTTP token header, so LAN SOCKS5 sharing needs
`lan_allowlist`. The runtime enforces exact IP and CIDR entries.

Desktop per-app routing is app-level proxy opt-in: point one app/browser profile
at the HTTP/SOCKS port and leave other apps direct. Transparent desktop per-app
capture requires OS-specific packet filtering and is intentionally not presented
as a one-click toggle.

Full guide: [`docs/sharing-and-per-app-routing.md`](sharing-and-per-app-routing.md).

## Diagnostics Tools

- **Scan IPs** -> `mhrv-f scan-ips`
- **Test SNI pool** -> `mhrv-f test-sni`
- **Scan SNI** -> `mhrv-f scan-sni`
- **Recent log**: compact UI log stream for Doctor/Test/Start/Stop feedback.

## Advanced Sections

- **Profiles**: save/load named config snapshots.
- **SNI pool tester**: find SNI hostnames that work from your network.
- **Advanced options**: performance, quota, LAN exposure, routing overrides,
  runtime auto-tune, YouTube relay policy, outage reset, and update checking.

Profile loading restores the full current form surface, including Vercel
settings, runtime tuning, LAN token/allowlist, outage reset settings, scan
settings, and account groups. That keeps profile switching from becoming a
partial state merge.

## Common UI Warnings

- **Cannot test**: required fields are missing for the selected mode.
- **HTML instead of JSON**: relay endpoint is protected, requires sign-in, or a
  challenge page intercepted it.
- **CA not trusted**: install the local CA, restart browsers, and rerun Doctor.
- **Remove CA failed**: rerun from an elevated shell with `mhrv-f --remove-cert`
  or remove the CA manually from the OS certificate manager.
- **Full mode test refused**: expected. Verify full mode by browsing through the
  tunnel and checking the tunnel-node public IP.
