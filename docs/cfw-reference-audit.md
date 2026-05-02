# CFW Reference Audit

The `mhr-cfw` reference folder was reviewed as a donor tree, not kept as a
second runnable product. The maintained integration lives in:

- `tools/cloudflare-worker-json-relay/worker.js`
- `assets/apps_script/CodeCloudflareWorker.gs`
- `docs/cloudflare-worker-json-relay.md`

## What Was Useful

| Donor component | Useful idea | Project status |
|---|---|---|
| `script/worker.js` | Worker exit between Apps Script and the target website | Ported, but hardened: Worker uses `WORKER_AUTH_KEY` from environment, rejects missing/invalid Worker secret, validates URL scheme, blocks Worker self-fetch, strips Cloudflare/client-IP headers, and uses chunked base64 helpers |
| `script/Code.gs` | Apps Script wrapper that preserves the normal client protocol while forwarding to Worker | Ported as `CodeCloudflareWorker.gs`, with diagnostic mode, decoy responses, stricter header stripping, batch support, manual redirects, and Worker-hop auth |
| `core/cert_installer.py` | Cross-platform CA install/check flows, including Firefox/NSS notes | Already covered by Rust cert installer and desktop Doctor; future improvement would be a first-class Firefox/NSS installer/check in Rust |
| `core/h2_transport.py` | Explicit HTTP/2 transport experimentation | Rust already uses HTTP/2 where relevant in relay transports; future improvement would be a UI-visible protocol diagnostic showing H1/H2 behavior per backend |
| `core/ws.py` and WebSocket tunnel paths | Raw WebSocket tunnel idea | Not ported into native mode because current project has `full` mode with `tunnel-node`; WebSocket could be a future separate backend only if it has tests and platform limits are clear |
| `core/domain_fronter.py` and `proxy_server.py` | Historical Python implementation of SNI rewrite and MITM relay | Already superseded by Rust `direct`, `fronting_groups`, Apps Script relay, and serverless JSON modes |
| `run.bat` / `run.sh` | Beginner launch scripts | Superseded by desktop app, installer scripts, and docs |

## What Was Not Kept

- Hardcoded sample secrets from the donor scripts.
- A second Python implementation of the local proxy.
- Duplicate Apps Script/Worker scripts whose behavior is already maintained in
  the Rust project.
- Raw donor docs with stale setup instructions.

## Highest-Value Future Additions

1. **Firefox/NSS certificate helper in Rust**: the donor references NSS store
   handling. Bringing this into Doctor would help Firefox users who trust the OS
   store but not the browser store.
2. **Cloudflare Worker health test**: add a small UI/CLI check that validates
   `WORKER_URL` + `WORKER_AUTH_KEY` before users redeploy Apps Script.
3. **Backend protocol diagnostics**: surface whether a relay path is using H1,
   H2, JSON/base64, XHTTP streaming, or full tunnel batching.
4. **Documented Worker quotas and failure modes**: Worker egress changes where
   target sites see traffic from, but it also adds Cloudflare limits and another
   secret. Keep it as an optional Apps Script companion, not a separate native
   mode.

## Compatibility Marker

`CodeCloudflareWorker.gs` now exposes the same helper metadata probe as the
other Apps Script helpers:

```text
https://script.google.com/macros/s/DEPLOYMENT_ID/exec?compat=1
```

For this variant, `kind` must be `apps_script_cloudflare_worker`. This gives
support and release checks a cheap way to confirm that a deployed Apps Script is
the maintained Cloudflare Worker bridge, not an older donor copy.
