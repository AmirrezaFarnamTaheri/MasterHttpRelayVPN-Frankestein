# Vercel JSON Relay

This folder contains the Vercel Edge Function used by `mhrv-f` mode
`vercel_edge`. It is a small authenticated JSON fetch relay:

- request: `{ "k": "...", "m": "GET", "u": "https://example.com", "h": {}, "b": "", "r": true }`
- response: `{ "s": 200, "h": { ... }, "b": "base64-body" }`

Unlike the XHTTP relay, this does not need your own VPS/backend server. The
trade-off is that desktop HTTPS still requires local MITM and a trusted
`mhrv-f` CA certificate, and all traffic counts against Vercel policy/limits.

If Vercel is not reachable on your network, `tools/netlify-json-relay` provides
the same JSON protocol on Netlify Edge.

## Deploy

1. Import or deploy this folder as a Vercel project.
2. In Vercel, set an environment variable:
   - `AUTH_KEY`: a long random secret.
3. Redeploy after changing `AUTH_KEY`.
4. In `config.json`, use:

```json
{
  "mode": "vercel_edge",
  "vercel": {
    "base_url": "https://your-project.vercel.app",
    "relay_path": "/api/api",
    "auth_key": "same value as AUTH_KEY",
    "verify_tls": true,
    "max_body_bytes": 4194304,
    "enable_batching": false
  }
}
```

## Health Check

```sh
curl -sS https://your-project.vercel.app/api/api
```

Expected: JSON similar to:

```json
{"ok":true,"name":"mhrv-f vercel-json-relay"}
```

## Relay Probe

```sh
curl -sS https://your-project.vercel.app/api/api \
  -H "content-type: application/json" \
  --data '{"k":"YOUR_AUTH_KEY","m":"GET","u":"https://example.com","h":{},"r":true}'
```

Expected: JSON with `s`, `h`, and base64 body `b`.

Batch probe:

```sh
curl -sS https://your-project.vercel.app/api/api \
  -H "content-type: application/json" \
  --data '{"k":"YOUR_AUTH_KEY","q":[{"m":"GET","u":"https://example.com","h":{},"r":true},{"m":"GET","u":"https://example.org","h":{},"r":true}]}'
```

Expected: JSON with a `q` array containing one relay response per item. Enable
`vercel.enable_batching` in `mhrv-f` after this succeeds.

Wrong key:

```sh
curl -i https://your-project.vercel.app/api/api \
  -H "content-type: application/json" \
  --data '{"k":"wrong","m":"GET","u":"https://example.com"}'
```

Expected: HTTP 401 JSON response:

```json
{"e":"unauthorized"}
```

## Deployment Protection Caveat

Vercel Deployment Protection or Vercel Authentication breaks this relay. The
symptom is that `mhrv-f` logs `HTML returned where JSON was expected` or
`non-JSON`, and the body preview looks like a Vercel login/protection page.

Fix: in Vercel project settings, disable Deployment Protection for the domain
used by `mhrv-f`, then redeploy. The relay already has its own `AUTH_KEY`; do
not put Vercel's HTML auth wall in front of the JSON endpoint.

## Notes

- Keep deployments private and use a strong `AUTH_KEY`.
- Vercel may log request metadata. Do not treat this as an anonymity system.
- Base64 adds bandwidth overhead. Large downloads are better served by the
  Apps Script/full-tunnel paths or a purpose-built transport.
- Read and follow Vercel's terms and local law.
