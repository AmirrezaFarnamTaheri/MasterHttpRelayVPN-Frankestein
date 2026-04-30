# Netlify Edge JSON Relay

`tools/netlify-json-relay/` is a Netlify Edge Function that implements the same
authenticated JSON/base64 fetch protocol as the bundled Vercel JSON relay. It
is a no-VPS native relay option for `mhrv-f`, separate from Xray/XHTTP.

The desktop mode is still named `vercel_edge` for config compatibility, but the
client is generic: it posts JSON to whatever `base_url + relay_path` you set.
That means a Netlify JSON deployment can use the same native client path.

## Architecture

```text
Browser/app
  -> local mhrv-f proxy
  -> local MITM for HTTPS
  -> JSON relay client
  -> Netlify Edge function
  -> destination website
```

The Netlify function accepts:

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

It also supports the short batch envelope `{ "k": "...", "q": [...] }`, matching
the Vercel JSON relay.

## Deploy The Relay

1. Create a Netlify site from `tools/netlify-json-relay/`.
2. Set environment variable `AUTH_KEY` to a long random secret.
3. Deploy.
4. Open `https://your-site.netlify.app/api/api`; it should return a JSON health
   response.

If `*.netlify.app` is unreliable, attach a custom domain to the Netlify site and
use that as the Base URL.

## Configure `mhrv-f`

Use the existing native serverless JSON mode:

```json
{
  "mode": "vercel_edge",
  "vercel": {
    "base_url": "https://your-site.netlify.app",
    "relay_path": "/api/api",
    "auth_key": "same value as AUTH_KEY",
    "verify_tls": true,
    "max_body_bytes": 4194304,
    "enable_batching": false
  }
}
```

In the desktop UI, choose **Serverless JSON**, paste the
Netlify Base URL, keep relay path `/api/api`, and enter the same auth key.

## Verify

From the UI:

1. Select the serverless JSON mode.
2. Fill Base URL and Auth key.
3. Install/check the local CA.
4. Click **Test relay**.
5. Click **Doctor** if the test fails.

From the CLI:

```bash
./mhrv-f test
./mhrv-f doctor
```

Common failure signatures:

- `401`: `AUTH_KEY` in Netlify does not match `vercel.auth_key`.
- HTML instead of JSON: the Netlify route did not reach the Edge Function, or
  another page intercepted `/api/api`.
- Timeout: Netlify cannot reach the destination site, the deployment is
  cold/slow, or the client network cannot reach Netlify reliably.
- Body too large: increase `vercel.max_body_bytes` carefully, or use a smaller
  request/upload.

## Difference From Netlify XHTTP

Netlify JSON relay:

- used by native `mhrv-f` serverless JSON mode
- no VPS backend required
- requires local MITM CA for HTTPS proxying

Netlify XHTTP relay:

- used by Xray/V2Ray clients
- requires your own Xray backend/VPS in `TARGET_DOMAIN`
- does not implement the `mhrv-f` JSON fetch protocol

See [`docs/netlify-xhttp-relay.md`](netlify-xhttp-relay.md) for the XHTTP tool.
