# Troubleshooting

Use this as a symptom-to-action map. Start with the quick checks, then jump to
the section that matches the symptom. When in doubt, run:

```bash
./mhrv-f doctor
./mhrv-f test
```

In `direct`, use `mhrv-f test-sni` instead of `mhrv-f test`. In `full`,
verify by browsing through the tunnel and checking the tunnel-node public IP.

## End-To-End Debug Checklist

Use this checklist before changing advanced settings:

1. Confirm the selected mode matches the backend you deployed.
2. Save config from the UI so the runtime reads the latest fields.
3. Run **Doctor** or `mhrv-f doctor`.
4. Run **Test relay** or `mhrv-f test` for `apps_script` and `vercel_edge`.
5. Run `mhrv-f test-sni` if Google edge connectivity is suspicious.
6. Start the proxy and test one browser profile through HTTP first.
7. Test SOCKS5 only after HTTP works.
8. Enable LAN sharing only after local-only mode works.
9. For Android, test VPN mode with one browser before adding app splitting.
10. For `full`, compare browser behavior with tunnel-node logs and an IP-check
    page.

Do not tune `parallel_relay`, range parallelism, DNS/DoH, or domain overrides
until the basic path passes. Those knobs are for known bottlenecks, not first
setup.

## Mismatch Checklist

Most confusing failures come from a mismatch between UI mode, deployed backend,
and docs being followed:

| If UI mode is | Backend must be | Common mismatch |
|---|---|---|
| `apps_script` | Apps Script `Code.gs` or `CodeCloudflareWorker.gs` | Pasting Vercel URL into Apps Script groups |
| `vercel_edge` | `tools/vercel-json-relay` on Vercel or `tools/netlify-json-relay` on Netlify | Protection/routing page returns HTML instead of JSON |
| `direct` | none | Expecting it to proxy the entire web |
| `full` | `CodeFull.gs` plus `tunnel-node` | Running JSON relay tests instead of checking tunnel-node/IP |
| XHTTP helper | Xray/V2Ray backend server | Expecting Netlify/Vercel XHTTP to be a native `mhrv-f` mode |

If the UI and backend do not match, stop and fix that first.

## Connect or Start Is Disabled

Likely causes:

- Apps Script mode has no enabled account group.
- Apps Script group has no deployment ID or auth key.
- Serverless JSON mode has no base URL or auth key.
- HTTP and SOCKS5 ports conflict.

Next actions:

- In the desktop UI, open the first-run wizard and complete **Mode** and
  **Relay**.
- For Apps Script, add one enabled account group with `AUTH_KEY` and deployment
  IDs.
- For serverless JSON, fill **Base URL**, **Relay path**, and **Auth key**.
- Click **Doctor**.

## `504 Relay Timeout`

Likely causes:

- Apps Script quota exhaustion or stale deployment.
- Google edge IP/SNI combination is no longer reachable.
- Vercel/Netlify Edge function or backend fetch timed out.
- Local network blocks the selected relay platform.

Next actions:

- Apps Script: redeploy a new version, add deployment IDs, or add another
  account group.
- Serverless JSON: check platform logs, redeploy, and verify the health endpoint.
- Run `mhrv-f test-sni` and `mhrv-f scan-ips`.
- Reduce burst knobs such as `parallel_relay` or enable
  `relay_rate_limit_qps` if quota spikes are the cause.

## HTML Returned Instead of JSON

Likely causes:

- Vercel Deployment Protection, Vercel auth, or a Netlify routing/protection
  page is enabled.
- Apps Script deployment requires sign-in.
- Apps Script returned its stock placeholder page after a stale deployment,
  `AUTH_KEY` mismatch, timeout, or quota tear.
- Apps Script returned a localized quota/error page. Persian `lang="fa"` or
  `dir="rtl"` HTML usually means the page came from Apps Script itself, not
  from `Code.gs`.
- Google served a Workspace landing page because the deployment owner account
  is restricted or needs action.
- A captive portal or backend challenge page intercepted the request.

Next actions:

- Vercel: disable Deployment Protection for the relay domain and redeploy.
- Netlify: confirm `/api/api` reaches the Edge Function and returns JSON.
- Apps Script: set web app access to **Anyone**.
- If logs mention the Apps Script placeholder/decoy body, set
  `DIAGNOSTIC_MODE=true` in `Code.gs` or `CodeFull.gs`, redeploy as a new
  version, then test again. Only an auth mismatch turns into explicit JSON
  `unauthorized`; quota, timeout, ISP truncation, and account restrictions keep
  returning their natural HTML/error bodies.
- For quota-looking pages, lower `parallel_concurrency`, add more deployments,
  or split deployments across more Google accounts.
- For Workspace landing pages, sign in to the deployment owner account and clear
  any Google action-required or verification prompt, or rotate to a healthier
  deployment owner.
- Check logs for response-quality hints such as `html_instead_of_json`,
  `cloudflare_challenge`, or `quota_or_limit`.

## Certificate Errors

Examples:

- `NET::ERR_CERT_AUTHORITY_INVALID`
- browser says the connection is not private
- Firefox works differently than Chrome or the OS browser

Likely causes:

- The local MITM CA is not trusted.
- Firefox is using its own NSS database and has not picked up OS trust yet.
- On Android 7+, the app you are testing does not trust user-installed CAs.

Next actions:

- Run `mhrv-f --install-cert` or click **Install CA** in the UI.
- Restart browsers after installing the CA.
- For Firefox, ensure `certutil` is available or import `ca/ca.crt` manually
  under Authorities.
- Never share `ca/ca.key`.

## Connected but Sites Do Not Load

Likely causes:

- Wrong `google_ip`.
- `front_domain` was changed to an IP or blocked hostname.
- DNS poisoning returned a bad Google frontend.
- Relay auth key is wrong.

Next actions:

- Keep `front_domain = "www.google.com"` unless you know why changing it helps.
- Run `mhrv-f test-sni`.
- Run `mhrv-f scan-ips` and paste a reachable IP.
- Re-copy the relay auth key with no trailing spaces.

## Telegram or Another Native App Shows "Updating"

Likely causes:

- `apps_script` and `vercel_edge` are HTTP fetch relays. They handle browser
  HTTP/HTTPS through local MITM, but they do not carry arbitrary native raw-TCP
  protocols such as Telegram MTProto.
- The app is using the SOCKS5 listener for raw TCP and there is no working
  `upstream_socks5`, so the fallback path connects directly.
- On Android, the app may not trust user-installed CAs, so HTTPS MITM is
  rejected even though browsers work.

Next actions:

- For browser-style HTTPS, use a browser that trusts the local CA.
- For raw TCP apps, use `full` mode with tunnel-node, or provide an existing
  local tunnel through `upstream_socks5`.
- If the app supports MTProto-over-TLS or another HTTPS-shaped transport, use
  that instead of native MTProto where possible.

## Full Mode: Chat Works but Google Search Does Not

This usually points to the full-tunnel DNS/SNI path, not to Apps Script JSON
relay auth.

Check:

- tunnel-node logs show traffic for the failing browser request
- `front_domain` is still a hostname such as `www.google.com`, not an IP
- udpgw/DNS handling is enabled and versions match on client and tunnel-node
- no local DNS/TUN route is accidentally colliding with the `198.18.0.0/15`
  virtual address space used by tun2proxy/udpgw paths

Temporary workaround while debugging:

```json
{
  "domain_overrides": [
    { "host": ".google.com", "force_route": "full_tunnel" },
    { "host": ".gstatic.com", "force_route": "full_tunnel" }
  ]
}
```

Then restart the tunnel and compare tunnel-node logs with browser behavior.

## Browser Hangs in SOCKS Mode but Native Chat Works

Likely causes:

- The browser or tool is using `socks5://` instead of `socks5h://`, so DNS is
  resolved locally and the proxy only receives IP literals.
- The target app has a separate "proxy DNS through SOCKS" checkbox and it is
  disabled.

Next actions:

- In Firefox, enable **Proxy DNS when using SOCKS v5**.
- In command-line tools, prefer `socks5h://127.0.0.1:PORT` over
  `socks5://127.0.0.1:PORT`.
- If you still see failures only for browser/search traffic, keep debugging
  the full-mode DNS/SNI path above.

## YouTube Loads but Video Buffers

Likely causes:

- `googlevideo.com` requests are byte-heavy and can burn relay quota.
- Too much range parallelism creates quota bursts.
- Your ISP treats YouTube paths differently from generic Google hosts.

Next actions:

- Add more Apps Script deployment IDs or account groups.
- Lower `parallel_relay` and `range_parallelism`, or enable
  `relay_rate_limit_qps`.
- Try `youtube_via_relay` only when YouTube HTML/API requests hit Restricted
  Mode or SNI-policy trouble. The toggle intentionally leaves `ytimg.com`
  assets on SNI rewrite and does not force `googlevideo.com` onto the normal
  Google frontend IP.

## `script.google.com` Is Blocked

Use `direct` mode:

1. Start `mhrv-f` in `direct`.
2. Set the browser proxy to the local HTTP port.
3. Open `script.google.com`.
4. Deploy Apps Script.
5. Switch back to `apps_script` and fill the new deployment ID/key.

## Serverless JSON Works in Curl but Not in `mhrv-f`

Check:

- `mode` is exactly `vercel_edge`.
- `vercel.base_url` has no path; use `https://project.vercel.app` or
  `https://site.netlify.app`.
- `vercel.relay_path` is `/api/api`.
- `vercel.auth_key` matches `AUTH_KEY`.
- the local CA is installed for HTTPS browsing.

Run:

```bash
./mhrv-f test
./mhrv-f doctor
```

## UDP Not Working in Full Tunnel

Likely causes:

- tunnel-node/client version mismatch
- udpgw disabled or unsupported
- UDP path is blocked beyond the local client

Next actions:

- Upgrade client and tunnel-node together.
- Confirm the tunnel-node logs show udpgw activity.
- Review [`docs/udpgw.md`](udpgw.md).
