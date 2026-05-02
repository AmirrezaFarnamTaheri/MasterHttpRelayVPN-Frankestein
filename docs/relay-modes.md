# Relay Modes

`mhrv-f` has more than one transport. Pick the smallest mode that fits your
network and threat model.

## Quick Comparison

| Mode | Needs VPS | Needs local CA | Best for | Main trade-off |
|---|---:|---:|---|---|
| `apps_script` | No | Yes | Classic free setup through Google Apps Script | Google quotas and Apps Script deployment care |
| `vercel_edge` | No | Yes | Serverless no-VPS JSON relay on Vercel or Netlify | Platform policy/limits and base64 overhead |
| `direct` | No | Yes | Bootstrap access and limited CDN fronting through Google/Vercel/Fastly/Netlify-style edges | Not a full tunnel; unmatched traffic goes raw/direct |
| `full` | Yes, tunnel-node | No local MITM CA | Full-device tunnel with end-to-end tunnel-node control | Requires server infrastructure |

## Apps Script Mode

Use this when you can deploy a Google Apps Script web app.

Data path:

```text
Browser/app -> local mhrv-f proxy -> Google edge -> Apps Script -> destination
```

What you configure:

- `mode = "apps_script"`
- one or more `account_groups`
- each group has an `auth_key` and one or more deployment IDs
- a trusted local MITM CA for HTTPS browsing

Strengths:

- Free and familiar.
- Multiple deployment IDs and account groups can spread quota.
- Works with the existing Doctor/Test relay flow.

Watch for:

- Apps Script daily quotas and transient `504`/timeout behavior.
- Deployment access must be **Anyone**.
- `AUTH_KEY` must match exactly between `Code.gs` and config.

Optional Worker exit:

- `assets/apps_script/CodeCloudflareWorker.gs` keeps the same client-side
  `apps_script` mode but forwards the final fetch through a private Cloudflare
  Worker from `tools/cloudflare-worker-json-relay/`.
- Use it only when you explicitly want Cloudflare Worker egress. It adds
  another quota surface and another secret to manage.

## Serverless JSON Mode

Use this when you want a no-VPS serverless fetch relay that is separate from
Apps Script. The config mode is `vercel_edge` for compatibility, but the JSON
relay protocol works with the bundled Vercel and Netlify tools.

Data path:

```text
Browser/app -> local mhrv-f proxy -> Vercel/Netlify JSON relay -> destination
```

What you configure:

- deploy [`tools/vercel-json-relay`](../tools/vercel-json-relay/README.md) or
  [`tools/netlify-json-relay`](../tools/netlify-json-relay/README.md)
- set `AUTH_KEY` in the platform environment
- `mode = "vercel_edge"`
- `vercel.base_url = "https://your-project.vercel.app"` or
  `"https://your-site.netlify.app"`
- `vercel.auth_key = "same secret as AUTH_KEY"`
- a trusted local MITM CA for HTTPS browsing

Strengths:

- No VPS.
- Does not depend on Google Apps Script.
- Native Rust client uses the same proxy/MITM dispatch path as Apps Script.
- Can be hosted on Vercel or Netlify with the same client protocol.

Watch for:

- Platform protection, login, or routing pages return HTML, but the native
  client expects JSON. Disable Vercel Deployment Protection for the relay
  domain, or make sure Netlify routes `/api/api` to the Edge Function.
- Requests and responses are JSON/base64 encoded, so bandwidth and CPU overhead
  are higher than direct HTTP.
- Keep deployments private and use a long random key. This is a generic fetch
  relay, so do not expose it without authentication.
- Vercel/Netlify plan limits, Edge runtime limits, and platform policy still
  apply.

## Direct Mode

Use this when the first blocker is reaching setup pages, or when you only need
targets that work through a known SNI-fronted edge.

Data path:

```text
Browser/app -> local mhrv-f proxy -> Google or configured CDN edge -> target host
```

What you configure:

- `mode = "direct"`
- `google_ip`
- `front_domain`
- optional `fronting_groups` for Vercel, Fastly, Netlify/CloudFront, or another
  verified multi-tenant edge
- no Apps Script or Vercel relay credentials

This is not meant to browse the whole web. Use it to reach `script.google.com`,
deploy Apps Script, test SNI reachability, access Google-owned services that
work through the built-in SNI rewrite path, or access domains listed in a
validated `fronting_groups` block.

Start from [`config.direct.example.json`](../config.direct.example.json) for
Google bootstrap, or
[`config.fronting-groups.example.json`](../config.fronting-groups.example.json)
for Vercel/Fastly/Netlify examples. Details and safety checks live in
[`docs/fronting-groups.md`](fronting-groups.md).

## Full Mode

Use this when you operate a tunnel-node and want a full tunnel rather than a
local MITM HTTP relay.

Data path:

```text
Device -> local mhrv-f/tun2proxy -> Apps Script tunnel channel -> tunnel-node -> internet
```

What you configure:

- `mode = "full"`
- Apps Script full-tunnel deployment credentials
- tunnel-node server
- Android/desktop full-tunnel plumbing as documented in `tunnel-node/README.md`

`mhrv-f test` refuses this mode because a single JSON HTTP probe cannot prove
the full tunnel. Verify by starting the tunnel, browsing, and checking that an
IP-check page shows the tunnel-node public IP.

Full-mode readiness is intentionally split between local blockers and
external checks:

- `account_groups.*` still blocks Start when no full-mode Apps Script
  deployment ID or `AUTH_KEY` is configured.
- `full.codefull_deployment` reminds you that each deployment ID must point to
  `CodeFull.gs`, not the classic `Code.gs` relay.
- `full.tunnel_node_url` means `CodeFull.gs` must set `TUNNEL_SERVER_URL` to
  the public tunnel-node origin.
- `full.tunnel_auth` means `TUNNEL_AUTH_KEY` must match exactly between
  `CodeFull.gs` and tunnel-node.
- `full.udp_support` warns when no SOCKS5 listener is configured for clients
  that expect SOCKS5 UDP ASSOCIATE.
- `full.tunnel_health` is the final smoke test: run Doctor with
  `--tunnel-node-url`, start full mode, open an IP-check page, and compare with
  tunnel-node logs.

The `full.*` rows are warnings rather than blockers because the client config
cannot directly inspect Apps Script constants or VPS environment variables.
Tunnel-node also exposes `/health/details` with version and capability flags
for operator dashboards and `mhrv-f doctor --tunnel-node-url
https://<tunnel-node>`.

Do not confuse `full` mode with `upstream_socks5`. Full mode owns the tunnel
path:

```text
client -> Apps Script tunnel channel -> tunnel-node -> internet
```

`upstream_socks5` is only a fallback/chaining option for the local proxy when a
working SOCKS5 proxy already exists somewhere else. It does not create a full
tunnel, does not replace tunnel-node, and is normally irrelevant once `full`
mode is active.

If chat apps work in full mode but browser searches do not, suspect DNS/SNI
routing rather than Apps Script relay auth. Run the tunnel-node logs, check
udpgw/DNS handling, keep `front_domain` as a hostname, and temporarily route
Google search hosts through the tunnel-node with `passthrough_hosts` or
`domain_overrides` while you isolate the broken path.

Full mode also keeps a short negative cache for structurally unreachable
destinations reported by tunnel-node. This prevents repeated OS/browser probes
to known-failing targets, such as IPv6-only hosts from an IPv4-only VPS, from
spending Apps Script batches over and over.

Full-mode batching is adaptive: the client waits briefly for neighboring tunnel
ops and extends that wait while new work arrives, up to the configured
`coalesce_max_ms`. Tunnel-node likewise uses a 15 second long-poll and a short
adaptive straggler settle window, which reduces empty polling without making
quick responses pay the worst-case delay.

Known browser DNS-over-HTTPS hosts are bypassed on TCP/443 unless
`tunnel_doh = true`. If a browser has secure DNS enabled and search behaves
differently from chat apps, check this setting together with `bypass_doh_hosts`.
For app-level SOCKS5 UDP, `block_quic = true` can also drop UDP/443 locally so
QUIC clients fall back to the regular TCP/HTTPS path instead of spending tunnel
batches on HTTP/3 probes.

## Rule of Thumb

1. If you can deploy Apps Script and quotas are acceptable, start with
   `apps_script`.
2. If Apps Script is unavailable but Vercel or Netlify is reachable, try the
   serverless JSON mode.
3. If you cannot even reach `script.google.com`, use `direct` long enough to
   bootstrap. Stay in `direct` only when your target sites are covered by
   Google SNI rewrite or verified fronting groups.
4. If you need full-device routing without local HTTPS MITM, use `full` with a
   tunnel-node.

## Default Value Suggestions

Start from these values before tuning. Change one thing at a time, then run
Doctor/Test again.

| Method | Parameter | Suggested default |
|---|---|---|
| Apps Script | `mode` | `apps_script` |
| Apps Script | `front_domain` | `www.google.com` |
| Apps Script | `google_ip` | auto-detected Google frontend or `216.239.38.120` as a known candidate |
| Apps Script | `listen_host` | `127.0.0.1` |
| Apps Script | `listen_port` / `socks5_port` | `8085` / `8086` on desktop; Android defaults are shown in app |
| Apps Script | `verify_ssl` | `true` |
| Apps Script | `parallel_relay` | `0` or `1` first; use `2` only after multiple IDs/accounts are healthy |
| Apps Script | `relay_rate_limit_qps` | unset first; add a limit only when quota/504 storms appear |
| Serverless JSON | `mode` | `vercel_edge` |
| Serverless JSON | `base_url` | `https://your-project.vercel.app` or `https://your-site.netlify.app` |
| Serverless JSON | `relay_path` | `/api/api` |
| Serverless JSON | `auth_key` | same long random secret as platform `AUTH_KEY` |
| Serverless JSON | `verify_tls` | `true` |
| Serverless JSON | `max_body_bytes` | 4 MiB first (`4194304` bytes); raise only when a real POST/upload needs it |
| Direct | `mode` | `direct` |
| Direct | `front_domain` | `www.google.com` |
| Direct | `sni_hosts` | leave empty, then test built-in Google pool |
| Direct + fronting groups | `fronting_groups` | start from `config.fronting-groups.example.json` and verify each edge |
| Full tunnel | `mode` | `full` |
| Full tunnel | `account_groups` | at least one Apps Script full-mode group with its own secret and deployment IDs |
| Full tunnel | `tunnel-node` | run on your VPS with `TUNNEL_AUTH_KEY` set |
| Full tunnel | `coalesce_step_ms` / `coalesce_max_ms` | leave defaults unless latency logs show batching problems |
| Desktop sharing | `listen_host` | `127.0.0.1`; use `0.0.0.0` only with token or allowlist |
| Android routing | Connection type | VPN/TUN for whole-device routing; Proxy-only for per-app opt-in |
| External Vercel XHTTP | Address/SNI | `react.dev`, `nextjs.org`, or another tested candidate |
| External Vercel XHTTP | Host | your Vercel app domain |
| External Vercel/Netlify XHTTP | Generator | use the desktop **Backend tools -> XHTTP VLESS generator**, or `tools/netlify-xhttp-relay/public/vless-generator.html` |
| External Netlify XHTTP | Address/SNI | your Netlify domain first; then test `kubernetes.io`, `helm.sh`, `letsencrypt.org`, and the documented Helm/Kubernetes/SIG subdomains |
| External Netlify XHTTP | Host | your Netlify site/custom domain |
| External XHTTP | Port / transport / mode | `443` / `xhttp` / `auto` |
| External XHTTP | `allowInsecure` | `false` first; `true` only for intentional mismatched-front tests |

## Setup Recipes

### Recipe: Apps Script

1. Deploy `assets/apps_script/Code.gs` as a Web app.
2. Set a long random `AUTH_KEY` in the script.
3. Publish as **Execute as: Me** and **Who has access: Anyone**.
4. In the desktop UI **Setup** tab, choose **Apps Script**.
5. In the **Advanced** tab, add an account group under **Multi-account pools**.
6. Paste the same auth key and one or more deployment IDs.
7. Install/check the local CA, save config, run Doctor, then Test relay.

### Recipe: Apps Script With Cloudflare Worker Exit

1. Deploy `tools/cloudflare-worker-json-relay`.
2. Set its `WORKER_AUTH_KEY`.
3. Deploy `assets/apps_script/CodeCloudflareWorker.gs` in Apps Script.
4. Put the Worker URL and Worker key into that Apps Script file.
5. Keep desktop mode as `apps_script`; the client does not need a new mode.

### Recipe: Vercel Edge JSON

1. Deploy `tools/vercel-json-relay`.
2. Set Vercel environment variable `AUTH_KEY`.
3. Disable Vercel Deployment Protection for the relay domain.
4. In the desktop UI **Setup** tab, choose **Serverless JSON**.
5. Paste Base URL, relay path `/api/api`, and the same auth key.
6. Install/check the local CA, save config, run Doctor, then Test relay.

### Recipe: Netlify Edge JSON

1. Deploy `tools/netlify-json-relay`.
2. Set Netlify environment variable `AUTH_KEY`.
3. Confirm `/api/api` returns a JSON health response.
4. In the desktop UI **Setup** tab, choose **Serverless JSON**.
5. Paste the Netlify Base URL, relay path `/api/api`, and the same auth key.
6. Install/check the local CA, save config, run Doctor, then Test relay.

### Recipe: Full Tunnel

1. Deploy the full-mode Apps Script file.
2. Build and run `tunnel-node` on the VPS.
3. Set matching authentication on both sides.
4. In the client, choose `full` and configure the account group.
5. Run `mhrv-f doctor --tunnel-node-url https://<tunnel-node>` to verify
   `/health/details` before starting.
6. Start the tunnel and verify with an IP-check page and tunnel-node logs.

### Recipe: Vercel XHTTP Helper

1. Operate a working Xray/V2Ray XHTTP backend server.
2. Deploy the Vercel Edge or Node XHTTP helper.
3. Configure your Xray/V2Ray client to use the Vercel helper hostname/path.
4. Generate profiles from the desktop **Backend tools -> XHTTP VLESS
   generator** with the Vercel preset, or hand-edit Address/SNI candidates from
   the list in `docs/field-notes.md`.
5. Do not select a special `mhrv-f` mode for this; it is an external tool.

Details: [`docs/vercel-xhttp-relay.md`](vercel-xhttp-relay.md).

### Recipe: Netlify XHTTP Helper

1. Operate a working Xray/V2Ray XHTTP backend server.
2. Deploy the Netlify Edge XHTTP helper.
3. Configure your Xray/V2Ray client to use the Netlify hostname/path or an
   attached custom domain.
4. Start with your Netlify site as `Address`/`SNI`/`Host`. If that front is
   unreliable in your external client, use the desktop **Backend tools -> XHTTP
   VLESS generator** with the Netlify preset, or open the deployed helper page at
   `/vless-generator.html`.
5. Keep Netlify/Fastly/CloudFront MITM-fronting rules in Xray/v2rayNG; they are
   not native desktop modes.

Details: [`docs/netlify-xhttp-relay.md`](netlify-xhttp-relay.md) and
[`docs/field-notes.md`](field-notes.md).

## Per-App And LAN Behavior

The selected relay mode decides the backend path. Per-app and LAN behavior is a
separate local-client decision:

- Desktop per-app routing is app-level proxy opt-in through the local HTTP or
  SOCKS5 listener.
- Android VPN mode uses native app splitting.
- Android Proxy-only mode is manual per-app proxy opt-in.
- Desktop or phone LAN sharing exposes the proxy listener to trusted devices;
  use allowlists for SOCKS5.

Details: [`docs/sharing-and-per-app-routing.md`](sharing-and-per-app-routing.md).
