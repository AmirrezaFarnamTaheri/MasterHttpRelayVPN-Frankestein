# Vercel XHTTP relay, Node streaming runtime

This optional tool is a Vercel Serverless Function variant of the bundled
`tools/vercel-xhttp-relay` Edge Function. Use it only when the Edge relay is
not a good fit for a specific account, route, or streaming behavior.

It still requires your own Xray/V2Ray backend with an XHTTP inbound.

## Deploy

1. Create a Vercel project from this folder.
2. Set:

```text
TARGET_DOMAIN=https://xray.example.com:2096
```

3. Deploy.

The function disables Vercel body parsing and enables response streaming. It
uses Node streams to pass the incoming request body to `fetch`, then pipelines
the backend response back to the client.

In VLESS/Xray clients, keep `Host` set to your deployed Vercel project domain.
If `vercel.com` as Address/SNI is unstable, test reachable edge names such as
`community.vercel.com`, `analytics.vercel.com`, `botid.vercel.com`,
`blog.vercel.com`, `app.vercel.com`, `api.vercel.com`, `ai.vercel.com`,
`cursor.com`, `nextjs.org`, or `react.dev`. Prefer normal certificate
validation; insecure TLS should only be used for short diagnostics.

## Differences from the Edge relay

- This uses the Node runtime instead of the Edge runtime.
- `vercel.json` requests 128 MB memory and a 60 second function duration.
- It may have different regional placement, cold starts, and account limits.
- It strips the same hop-by-hop and Vercel forwarding headers as the Edge
  helper and keeps backend redirects manual.

For most users, start with `tools/vercel-xhttp-relay` first.
