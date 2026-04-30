# Netlify XHTTP Relay

`tools/netlify-xhttp-relay/` is an optional Netlify Edge Function for
Xray/V2Ray users. It streams XHTTP traffic from a Netlify site or a custom
domain attached to Netlify to your own backend Xray server.

This helper is intentionally treated like the Vercel XHTTP helper:

- It is an external Xray/V2Ray deployment recipe, not a selectable native
  desktop mode.
- It requires your own Xray backend/VPS with an XHTTP inbound.
- It does not use Apps Script account groups, Netlify/Vercel JSON, `tunnel-node`,
  or local MITM certificates.
- It is useful only when the Netlify edge hostname is reachable from the client
  network and the backend Xray server is reachable from Netlify.

## XHTTP Only

This relay only works with Xray's `xhttp` transport. It does not work with
WebSocket, gRPC, TCP, mKCP, QUIC, or arbitrary UDP/TCP. Netlify Edge exposes
HTTP request/response streaming through `fetch`; it does not expose the raw
sockets or protocol upgrades needed by other proxy transports.

## How It Works

1. Your Xray client connects to a Netlify-hosted hostname over TLS.
2. The Netlify Edge Function forwards `/p4r34m` and `/p4r34m/*` requests to
   `TARGET_DOMAIN`.
3. Your backend Xray server handles the real XHTTP inbound and streams the
   response back through Netlify.

The handler strips hop-by-hop headers, Netlify forwarding headers, and protocol
upgrade headers, preserves a single `x-forwarded-for` value when available, and
uses `redirect: "manual"` so backend redirects cannot silently alter XHTTP
framing.

## Deployment Path A: CLI

Requirements:

- A Netlify account.
- A working Xray server with XHTTP inbound.
- Netlify CLI: `npm i -g netlify-cli`.

Deploy:

```sh
cd tools/netlify-xhttp-relay
netlify deploy --prod
```

In Netlify site settings, add:

- `TARGET_DOMAIN`: full backend origin, for example
  `https://xray.example.com:2096`

Include the scheme and port. Do not include a trailing slash. Redeploy after
changing the environment variable.

The deployed site also serves a small browser helper at
`https://your-site.netlify.app/vless-generator.html`. It generates VLESS share
links for tested Netlify Address/SNI candidates while keeping `Host` set to your
Netlify site. The same page also has a Vercel preset so users migrating between
the Vercel and Netlify XHTTP helpers can compare equivalent profile shapes.

## Deployment Path B: Dashboard Import

If command-line deployment is awkward:

1. Copy or fork `tools/netlify-xhttp-relay/` into a small repository.
2. Open Netlify, choose **Add new site -> Import an existing project**.
3. Import the repository.
4. In **Environment variables**, add `TARGET_DOMAIN`.
5. Deploy the site.

Netlify will give you a hostname such as `your-site.netlify.app`.

## Client Configuration

The client talks to Netlify, while the path and UUID must still match the real
backend Xray inbound.

Typical field mapping for v2rayN/v2rayNG:

- **Address**: your Netlify hostname/custom domain by default; if that is not
  reachable, test the field candidates below.
- **Port**: `443`.
- **SNI**: normally the same value as Address. If you use a field candidate as
  Address, use the same candidate as SNI.
- **Host**: your Netlify site hostname or attached custom domain. This should
  continue to identify the deployed Netlify relay unless your external Xray
  profile intentionally does something else.
- **Transport**: `xhttp`.
- **Path**: the XHTTP inbound path on your backend server.
- **Mode**: `auto`.
- **Security**: `tls`.
- **allowInsecure**: prefer `false`. Use `true` only for a deliberate
  mismatched Address/SNI/Host test where you accept the trust downgrade.

Field candidates reported as reachable for Netlify-XHTTP `Address` and `SNI`
testing:

```text
kubernetes.io
helm.sh
letsencrypt.org
docs.helm.sh
kubectl.docs.kubernetes.io
blog.helm.sh
kind.sigs.k8s.io
cluster-api.sigs.k8s.io
krew.sigs.k8s.io
gateway-api.sigs.k8s.io
scheduler-plugins.sigs.k8s.io
kustomize.sigs.k8s.io
image-builder.sigs.k8s.io
```

The bundled generator uses this list by default. Keep only the candidates that
actually connect from your own network.

The desktop app exposes the same generator under **Backend tools -> XHTTP VLESS
generator**, so this is not just a separate static script.

Example VLESS share link:

```text
vless://YOUR-UUID@kubernetes.io:443?encryption=none&security=tls&sni=kubernetes.io&alpn=h2%2Chttp%2F1.1&insecure=1&allowInsecure=1&type=xhttp&path=%2Fp4r34m&host=your-site.netlify.app&mode=auto#netlify-xhttp-kubernetes
```

Example Xray outbound:

```json
{
  "protocol": "vless",
  "settings": {
    "vnext": [
      {
        "address": "kubernetes.io",
        "port": 443,
        "users": [{ "id": "YOUR-UUID", "encryption": "none" }]
      }
    ]
  },
  "streamSettings": {
    "network": "xhttp",
    "security": "tls",
    "tlsSettings": {
      "serverName": "kubernetes.io",
      "allowInsecure": true
    },
    "xhttpSettings": {
      "path": "/p4r34m",
      "host": "your-site.netlify.app",
      "mode": "auto"
    }
  }
}
```

Prefer normal certificate validation. Use insecure TLS only as a short
diagnostic step when you already understand the certificate trade-off.

## Suggested Defaults

Use these as the first profile before experimenting:

| Field | Default suggestion |
|---|---|
| `TARGET_DOMAIN` | `https://xray.example.com:2096` |
| Address | `your-site.netlify.app` or your attached custom domain |
| Port | `443` |
| SNI | same as Address |
| Host | `your-site.netlify.app` or attached custom domain |
| Transport | `xhttp` |
| Path | `/p4r34m` unless your backend uses a different XHTTP path |
| Mode | `auto` |
| Security | `tls` |
| allowInsecure | `false` for normal Netlify host; `true` for tested mismatched Address/SNI candidates |

## Custom Domain Fallback

If `*.netlify.app` is blocked or unstable on your network, attach a custom
domain to the Netlify site. Then use that custom domain as `Host`, and
optionally as `Address`/`SNI` if it routes more reliably to Netlify.

## Field Notes: Netlify, Fastly, CloudFront

The pasted MITM-DomainFronting/Xray diffs describe a different class of client
configuration from this Netlify XHTTP helper.

- This Netlify helper forwards XHTTP to `TARGET_DOMAIN`.
- The external MITM/fronting configs route `geosite:netlify` through a
  CloudFront-like TLS repack path with a `letsencrypt.org` front and
  AWS/CloudFront certificate names.
- Earlier revisions of that external config added Fastly-oriented routes for
  GitHub, GitHub assets, Reddit, and Fastly IP ranges using `www.python.org` as
  the front.
- v2rayNG imports of those external configs may depend on Hev TUN being enabled
  and the default local SOCKS port staying at `10808`.

Keep those ideas in Xray/v2rayNG. They do not become `mhrv-f` native modes and
they are not required for `tools/netlify-xhttp-relay/`.

## Netlify JSON Is Different

If you want the native no-VPS JSON relay used by the desktop UI, use
[`tools/netlify-json-relay`](../tools/netlify-json-relay/README.md) instead.
That tool does not require an Xray backend, but it does require the local MITM
CA for HTTPS proxying.

## Limitations And Trust Model

- You still operate and trust the backend Xray server.
- All relayed traffic counts against Netlify account quotas and policies.
- Netlify may log request metadata such as path, IP, timing, and status.
- Edge runtime limits apply; stalled backends can still fail.
- The Edge runtime does not expose raw TCP sockets, UDP sockets, or arbitrary
  protocol upgrades.
- This is not an anonymity system and not production infrastructure.

Use private deployments, comply with Netlify terms and local law, and keep a
fallback path in case Netlify changes Edge limits or policy enforcement.
