# Netlify XHTTP relay

This optional tool deploys a Netlify Edge Function that streams XHTTP traffic to
your own Xray/V2Ray backend. It is useful when a Netlify domain or a custom
domain attached to Netlify is reachable from a network where your backend is
not.

This is separate from `mhrv-f` native relay modes:

- It requires your own Xray/V2Ray backend with an XHTTP inbound.
- It does not use Apps Script, Netlify/Vercel JSON, `tunnel-node`, or local
  MITM.
- It is an external fronting helper for clients that already support XHTTP.

## Quick start

1. Create a Netlify site from this folder or copy these files into a new repo.
2. Set an environment variable:

```text
TARGET_DOMAIN=https://xray.example.com:2096
```

Include the scheme and port. Do not include a trailing slash.

3. Deploy the site.

The `netlify.toml` file routes every path to `netlify/edge-functions/relay.js`.
The relay path defaults to `/p4r34m`. The public site root stays available for
the included helper page at `/vless-generator.html`.

## Client mapping

Keep the path, UUID, and XHTTP settings aligned with your real backend. The
client connects to Netlify, and Netlify forwards the same path and query to
`TARGET_DOMAIN`.

Typical v2rayN/v2rayNG fields:

- Address: your Netlify site hostname/custom domain first; if blocked, test
  the reachable front candidates below
- Port: `443`
- SNI: same as Address
- Host: your Netlify site hostname or custom domain
- Transport: `xhttp`
- Path: the XHTTP inbound path on your backend
- Recommended path for this bundled helper: `/p4r34m`
- Mode: `auto`
- allowInsecure: `false` first; use `true` only for deliberate mismatched
  Address/SNI/Host testing

Example:

```text
vless://YOUR-UUID@kubernetes.io:443?encryption=none&security=tls&sni=kubernetes.io&alpn=h2%2Chttp%2F1.1&insecure=1&allowInsecure=1&type=xhttp&path=%2Fp4r34m&host=your-site.netlify.app&mode=auto#netlify-xhttp-kubernetes
```

If `*.netlify.app` is unreliable, attach a custom domain to the Netlify site
and use that custom domain as `Host`, and optionally as `Address`/`SNI`.
Prefer normal certificate validation; insecure TLS is only a diagnostic knob.

After deploying, open:

```text
https://your-site.netlify.app/vless-generator.html
```

Paste your backend UUID and Netlify hostname. The page generates one VLESS link
per tested Address/SNI candidate. It also includes a Vercel preset for comparing
or migrating XHTTP profiles. The desktop app exposes the same workflow in
**Backend tools -> XHTTP VLESS generator**.

Reported Address/SNI candidates worth testing when the direct Netlify hostname
does not connect:

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

## Field notes

- Treat this like the Vercel XHTTP helper: it needs an Xray/V2Ray XHTTP backend
  and is not a native `mhrv-f` mode.
- External MITM/fronting configs may route `geosite:netlify` through a
  CloudFront-like TLS repack using `letsencrypt.org` plus AWS/CloudFront
  certificate names. Keep that in the Xray/v2rayNG client; it is not required by
  this helper.
- Fastly-oriented external rules for GitHub, GitHub assets, Reddit, or Fastly
  ranges are separate from Netlify XHTTP.
- Some v2rayNG imported configs depend on Hev TUN being enabled and local port
  `10808` staying unchanged.

## Notes

- XHTTP only. This relay does not support WebSocket, gRPC, raw TCP, UDP, QUIC,
  or arbitrary proxy protocols.
- Request and response bodies are streamed; the relay does not buffer payloads
  intentionally.
- Hop-by-hop headers, Netlify forwarding headers, and protocol upgrade headers
  are stripped before forwarding.
- Redirects are kept manual so backend 3xx responses cannot silently rewrite
  XHTTP framing.
- Netlify account quotas, Edge Function limits, logging, and policy rules still
  apply.

For the native no-VPS JSON relay used by the desktop UI, use
`tools/netlify-json-relay` instead.
