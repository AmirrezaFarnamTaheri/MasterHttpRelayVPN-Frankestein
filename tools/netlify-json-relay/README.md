# Netlify JSON relay

This optional tool deploys a Netlify Edge Function that speaks the same
authenticated JSON/base64 relay protocol as `tools/vercel-json-relay`.

Use it with the native desktop serverless JSON mode by selecting **Serverless JSON**
in the UI and pasting the Netlify site URL. The config key is still named
`vercel` for compatibility, but the relay protocol is platform-neutral.

## Quick start

1. Create a Netlify site from this folder or copy these files into a new repo.
2. Set the environment variable:

```text
AUTH_KEY=replace-with-a-long-random-secret
```

3. Deploy the site.
4. Open `https://your-site.netlify.app/api/api`; it should return JSON:

```json
{"ok":true,"name":"mhrv-f netlify-json-relay"}
```

5. In `mhrv-f-ui`, choose **Serverless JSON** and use:

- Base URL: `https://your-site.netlify.app`
- Relay path: `/api/api`
- Auth key: the same value as `AUTH_KEY`

## Files

- `netlify/edge-functions/api.js`: authenticated JSON relay handler
- `netlify.toml`: routes `/api/api` to the Edge Function
- `public/index.html`: minimal publish directory required by Netlify

## Notes

- This is a no-VPS fetch relay, not an XHTTP relay.
- It requires the local MITM CA for HTTPS proxying, same as Vercel JSON mode.
- Request and response bodies are base64 encoded inside JSON; expect overhead.
- Keep the relay private, authenticated, and within Netlify account limits and
  policy.
- If you need Xray/V2Ray XHTTP, use `tools/netlify-xhttp-relay` instead.
