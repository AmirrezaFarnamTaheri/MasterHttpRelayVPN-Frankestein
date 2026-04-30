# Vercel Edge JSON Relay

`vercel_edge` is a native `mhrv-f` mode that sends decrypted local HTTP requests
to a small authenticated JSON Edge function. The bundled Vercel tool is one
host for that protocol; Netlify can host the same protocol via
[`tools/netlify-json-relay`](../tools/netlify-json-relay/README.md). It is a
no-VPS alternative to Apps Script, not an Xray/XHTTP front.

## Architecture

```text
Browser/app
  -> local mhrv-f proxy
  -> local MITM for HTTPS
  -> Vercel JSON relay client
  -> Vercel Edge function
  -> destination website
```

The Vercel function accepts JSON requests like:

```json
{
  "k": "AUTH_KEY",
  "m": "GET",
  "u": "https://example.com/",
  "h": {},
  "b": ""
}
```

It returns:

```json
{
  "s": 200,
  "h": { "content-type": "text/html" },
  "b": "base64-response-body"
}
```

## Deploy the Relay

The bundled tool lives at [`tools/vercel-json-relay`](../tools/vercel-json-relay/README.md).

1. Create a new Vercel project from `tools/vercel-json-relay/`.
2. Set environment variable `AUTH_KEY` to a long random secret.
3. Deploy.
4. Open `https://your-project.vercel.app/api/api`; it should return a JSON
   health response.
5. Disable Vercel Deployment Protection for the relay domain.

Deployment Protection is important: if Vercel returns a login/protection HTML
page, `mhrv-f` cannot parse the expected JSON response and local requests fail
with a gateway-style error.

## Configure `mhrv-f`

Minimal config:

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

The desktop UI exposes the same fields under **Mode -> Serverless JSON** and
**Serverless JSON relay**.

Set `vercel.enable_batching` to `true` if you want the client to group page
bursts into short `q` batches. The bundled relay supports this; if an older
deployment does not, the client logs the mismatch and falls back to single JSON
fetches.

## Verify

From the UI:

1. Select **Serverless JSON (no VPS)**.
2. Fill **Base URL** and **Auth key**.
3. Install/check the local CA.
4. Click **Test relay**.
5. Click **Doctor** if the test fails.

From the CLI:

```bash
./mhrv-f test
./mhrv-f doctor
```

Common failure signatures:

- `401`: `AUTH_KEY` in Vercel does not match `vercel.auth_key`.
- HTML instead of JSON: Deployment Protection, Vercel auth, or another proxy
  page is intercepting the function.
- Timeout: Vercel cannot reach the destination site, the deployment is cold/slow,
  or the client network cannot reach Vercel reliably.
- Body too large: increase `vercel.max_body_bytes` carefully, or use a smaller
  request/upload.

## Security and Limits

- This is a generic authenticated fetch relay. Keep the key private.
- Do not publish a shared public relay. Private per-user deployments are safer.
- The local MITM CA can decrypt HTTPS for traffic routed through `mhrv-f`; only
  install it on devices you control.
- JSON/base64 adds overhead. Expect more bandwidth and CPU use than direct HTTP.
- Vercel plan limits, Edge runtime limits, and Vercel acceptable-use policy
  still apply.

## Netlify JSON Alternative

If Vercel domains are blocked or unreliable on your network, deploy
[`tools/netlify-json-relay`](../tools/netlify-json-relay/README.md) and keep the
same desktop mode. Use the Netlify site URL as `vercel.base_url`, keep
`relay_path = "/api/api"`, and set the same `AUTH_KEY`.

## Difference from Vercel XHTTP

Vercel JSON relay:

- used by native `mhrv-f` `vercel_edge` mode
- no VPS backend required
- requires local MITM CA for HTTPS proxying

Vercel XHTTP relay:

- used by Xray clients
- requires your own Xray backend/VPS in `TARGET_DOMAIN`
- does not implement the `mhrv-f` JSON fetch protocol

See [`docs/vercel-xhttp-relay.md`](vercel-xhttp-relay.md) for the XHTTP tool.
