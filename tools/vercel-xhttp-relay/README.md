# Vercel XHTTP relay (Edge)

This folder contains a Vercel Edge Function that streams XHTTP traffic to your
backend Xray server.

It uses Vercel as a reachable front in censored networks. It is an optional
external relay tool, not part of `mhrv-f`'s native runtime.

## Quick start

1. Create a Vercel project from this folder or copy these files into a new repo.
2. Set the environment variable:

- `TARGET_DOMAIN`: for example `https://xray.example.com:2096`

3. Deploy.

## Files

- `api/index.js`: edge handler, streaming relay
- `vercel.json`: routes all paths to `/api/index`
- `package.json`: metadata, no runtime dependencies

## Notes

- XHTTP only. It will not work for WS/gRPC/TCP transports.
- This is not part of `mhrv-f`'s runtime; it is an optional external relay tool.
- In VLESS/Xray clients, keep `Host` set to your deployed Vercel project
  domain. If `vercel.com` as Address/SNI is unstable, test reachable Vercel or
  framework edge names such as `community.vercel.com`, `analytics.vercel.com`,
  `botid.vercel.com`, `blog.vercel.com`, `app.vercel.com`, `api.vercel.com`,
  `ai.vercel.com`, `cursor.com`, `nextjs.org`, or `react.dev`.
- Prefer normal certificate validation. Use insecure TLS only as a short
  diagnostic step, not as the saved production profile.
- The desktop app includes **Backend tools -> XHTTP VLESS generator** with a
  Vercel preset. The static helper at
  `tools/netlify-xhttp-relay/public/vless-generator.html` can generate the same
  Vercel-shaped links when you want a browser-only tool.
- The handler streams `req.body` to `TARGET_DOMAIN` with `duplex: "half"` and
  `redirect: "manual"` so XHTTP frames are not buffered or rewritten.
- Hop-by-hop headers, Vercel forwarding headers, and protocol-upgrade headers
  are stripped before forwarding; `x-real-ip` and `x-forwarded-for` are
  collapsed to a single `x-forwarded-for` value when available.
- If the Edge runtime is not a good fit for your account or route, compare
  `tools/vercel-xhttp-relay-node/`.
