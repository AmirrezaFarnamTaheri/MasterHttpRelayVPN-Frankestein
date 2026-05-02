# Cloudflare Worker JSON Relay

`tools/cloudflare-worker-json-relay/` is an optional server-side relay variant
distilled from the `mhr-cfw` snapshot. It keeps the normal `mhrv-f`
`apps_script` client mode, but moves the final egress hop behind a Cloudflare
Worker:

```text
mhrv-f client -> Apps Script -> Cloudflare Worker -> target website
```

This can help when a site treats Google Apps Script egress poorly but is more
tolerant of Cloudflare Worker egress. It is not a replacement for full tunnel
mode, and it does not add raw TCP/UDP support.

## Deploy

1. Create a Cloudflare Worker.
2. Copy `tools/cloudflare-worker-json-relay/worker.js` into the Worker editor.
3. Add a Worker environment variable named `WORKER_AUTH_KEY` and set it to a
   strong random secret. Do not hardcode it in `worker.js`.
4. Deploy and copy the Worker URL.
5. Copy `assets/apps_script/CodeCloudflareWorker.gs` into Apps Script.
6. Set `WORKER_URL`, set the same `WORKER_AUTH_KEY`, and set `AUTH_KEY`.
7. Deploy Apps Script as a web app with access set to **Anyone**.
8. Open the Apps Script compatibility probe:

   ```text
   https://script.google.com/macros/s/DEPLOYMENT_ID/exec?compat=1
   ```

   Confirm `kind` is `apps_script_cloudflare_worker` before configuring the
   client. The probe exposes helper metadata only; it does not expose secrets.
9. Configure normal `mhrv-f` `apps_script` mode with the Apps Script deployment
   ID and the client-facing `AUTH_KEY`.

## Why The Worker Secret Matters

The original `mhr-cfw` Worker shape can become an open public fetch proxy if the
Worker URL is discovered. The bundled Worker rejects requests without
`WORKER_AUTH_KEY`, while Apps Script still validates the normal client-facing
`AUTH_KEY`.

Keep the two secrets separate:

- `AUTH_KEY`: client -> Apps Script.
- `WORKER_AUTH_KEY`: Apps Script -> Cloudflare Worker.

## Limits

- HTTP/HTTPS only. It does not carry Telegram MTProto, WebSocket upgrades, raw
  TCP, UDP, QUIC, or arbitrary VPN protocols.
- Traffic now depends on both Apps Script quota and Cloudflare Worker limits.
- Google sees requests to your Worker URL. Cloudflare sees the target URLs.
- Cloudflare-protected targets may still challenge or block Worker egress.
