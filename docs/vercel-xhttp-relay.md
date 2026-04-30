# Vercel XHTTP Relay

`tools/vercel-xhttp-relay/` is an optional Vercel Edge Function for Xray/V2Ray
users. It streams XHTTP traffic from a Vercel domain to your own backend Xray
server. A Node streaming fallback is also available at
`tools/vercel-xhttp-relay-node/` for accounts or routes where the Edge runtime
is not a good fit.

This is separate from `mhrv-f`'s native proxy modes:

- **Vercel XHTTP relay**: needs your own Xray backend/VPS, configured as
  `TARGET_DOMAIN`.
- **Vercel JSON relay (`vercel_edge`)**: does not need a VPS, but uses local
  MITM + trusted CA and has JSON/base64/policy limits.

## XHTTP Only

This relay only works with Xray's `xhttp` transport. It does not work with
WebSocket, gRPC, TCP, mKCP, QUIC, or arbitrary UDP/TCP. Vercel Edge exposes
HTTP request/response streaming through `fetch`; it does not expose raw sockets
or protocol upgrades needed by those transports.

## How It Works

1. Your Xray client connects to a Vercel-fronted hostname over TLS.
2. The Edge Function forwards the same path/query and streamed body to
   `TARGET_DOMAIN`.
3. The backend Xray server handles the real XHTTP inbound and streams the
   response back through Vercel.

The bundled handler is deliberately tiny and streaming-first: it passes
`req.body` directly into Edge `fetch(..., { duplex: "half" })`, strips
hop-by-hop and Vercel forwarding headers in one pass, preserves the client's
real IP as `x-forwarded-for` when present, and uses `redirect: "manual"` so a
backend 3xx cannot break XHTTP framing.

## Deployment Path A: CLI

Requirements:

- A Vercel account.
- A working Xray server with XHTTP inbound.
- Vercel CLI: `npm i -g vercel`.

Deploy:

```sh
cd tools/vercel-xhttp-relay
vercel --prod
```

In Vercel project settings, add:

- `TARGET_DOMAIN`: full backend origin, for example
  `https://xray.example.com:2096`

Include the scheme and port. Do not include a trailing slash. Redeploy after
changing the environment variable.

## Deployment Path B: Dashboard Import

If command-line deployment is awkward:

1. Fork or copy the relay project to your GitHub account.
2. Open Vercel, choose **Add New -> Project**.
3. Import the GitHub repository.
4. In **Environment Variables**, add `TARGET_DOMAIN`.
5. Click **Deploy**.

Vercel will give you a hostname such as `your-app.vercel.app`.

## Client Configuration

The client talks to Vercel, while the path and UUID must still match the real
backend Xray inbound.

Typical field mapping for v2rayN/v2rayNG:

- **Address**: `vercel.com`
- **Port**: `443`
- **SNI**: `vercel.com`
- **Host**: `your-app.vercel.app` or your custom Vercel domain
- **Transport**: `xhttp`
- **Path**: the XHTTP inbound path on your backend server
- **Mode**: `auto`

Example VLESS share link:

```text
vless://YOUR-UUID@nextjs.org:443?mode=auto&path=%2Fyourpath&security=tls&encryption=none&insecure=1&host=your-app.vercel.app&type=xhttp&allowInsecure=1&sni=nextjs.org&alpn=h2%2Chttp%2F1.1&fp=chrome#vercel-xhttp-nextjs
```

Reachable edge names vary by ISP. If `vercel.com` is filtered or unstable, test
these Address/SNI pairs while keeping **Host** set to your actual Vercel project
domain:

```text
community.vercel.com
analytics.vercel.com
botid.vercel.com
blog.vercel.com
app.vercel.com
api.vercel.com
ai.vercel.com
cursor.com
nextjs.org
react.dev
```

Only keep a candidate that passes on your own network. Prefer
`allowInsecure = false`; use insecure TLS only as a temporary diagnostic step
when you already understand the certificate trade-off.

The desktop app can generate these profiles in **Backend tools -> XHTTP VLESS
generator**. The bundled static generator at
`tools/netlify-xhttp-relay/public/vless-generator.html` also includes a Vercel
preset for migrations and comparisons.

Example Xray outbound:

```json
{
  "protocol": "vless",
  "settings": {
    "vnext": [
      {
        "address": "vercel.com",
        "port": 443,
        "users": [{ "id": "YOUR-UUID", "encryption": "none" }]
      }
    ]
  },
  "streamSettings": {
    "network": "xhttp",
    "security": "tls",
    "tlsSettings": {
      "serverName": "vercel.com",
      "allowInsecure": false
    },
    "xhttpSettings": {
      "path": "/yourpath",
      "host": "your-app.vercel.app",
      "mode": "auto"
    }
  }
}
```

## Custom Domain Fallback

If `*.vercel.app` is blocked on your network, attach a custom domain to the
Vercel project. Then use that domain as the XHTTP `Host`, and optionally as
the client `Address`/`SNI` if that routes more reliably to Vercel on your
network.

## Node Runtime Fallback

Start with the Edge relay. If long streams fail in a way that appears specific
to the Edge runtime, deploy `tools/vercel-xhttp-relay-node/` instead. It
disables body parsing, enables response streaming, and uses Node streams to
pipe request and response bodies. It has different cold-start, region, duration,
and account-limit behavior from the Edge helper.

## Netlify Alternative

If Vercel domains are blocked or unreliable on your network, the repository
also includes a Netlify Edge XHTTP helper at `tools/netlify-xhttp-relay/`. See
[`docs/netlify-xhttp-relay.md`](netlify-xhttp-relay.md).

## Limitations And Trust Model

- You still operate and trust the backend Xray server.
- All relayed traffic counts against Vercel account quotas and policies.
- Vercel may log request metadata such as path, IP, timing, and status.
- Edge runtime CPU/time limits apply; streaming usually keeps CPU low, but a
  stalled backend can still fail.
- The Edge runtime does not expose WebSocket upgrades, arbitrary TCP sockets,
  or UDP sockets; changing client transport away from XHTTP requires a
  different relay architecture.
- This is not an anonymity system and not production infrastructure.

Use private deployments, comply with Vercel terms and local law, and keep a
fallback path in case Vercel changes Edge limits or policy enforcement.
