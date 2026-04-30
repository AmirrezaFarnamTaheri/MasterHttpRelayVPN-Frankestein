# Cloudflare Worker JSON relay

This optional helper is distilled from the `mhr-cfw` snapshot. It keeps the
normal `mhrv-f` client protocol, but changes the server-side egress path:

```text
mhrv-f client -> Apps Script -> Cloudflare Worker -> target website
```

Use it only when you specifically want target sites to see Cloudflare Worker
egress instead of Google Apps Script egress. It adds another account, another
quota surface, and another place to keep secrets in sync.

## Deploy

1. Create a Cloudflare Worker.
2. Copy `worker.js` into the Worker editor.
3. Add a Worker environment variable named `WORKER_AUTH_KEY` with a strong
   random string.
4. Deploy the Worker and copy its URL.
5. Copy `assets/apps_script/CodeCloudflareWorker.gs` into Apps Script.
6. Set the same `WORKER_AUTH_KEY`, set `WORKER_URL`, and set `AUTH_KEY`.
7. Deploy the Apps Script web app as usual.
8. Use the Apps Script deployment ID and `AUTH_KEY` in normal `mhrv-f`
   `apps_script` mode.

## Security Notes

- Do not put `WORKER_AUTH_KEY` in source control. Keep it in the Worker
  environment settings and in your Apps Script copy of `CodeCloudflareWorker.gs`.
- The Worker rejects requests without the Worker secret, which prevents it from
  becoming an open public fetch proxy.
- The Apps Script `AUTH_KEY` is still the client-facing secret. The Worker
  secret is only for the Apps Script -> Worker hop.
- The Worker strips hop-by-hop, forwarding, Cloudflare identity, and client-IP
  headers before fetching the target URL.
