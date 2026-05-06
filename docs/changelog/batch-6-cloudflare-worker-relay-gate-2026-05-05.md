# Batch 49 - Cloudflare Worker Relay Bridge Drift Gate

Date: 2026-05-05

## Summary

- Added `tools/check-cloudflare-worker-relay.py`.
- Wired the new guard into `tools/run-repo-sanity.py`.
- Added the new guard to `tools/check-ci-local-sanity-parity.py`.
- Documented the guard in `tools/README.md`.

## Contract Now Guarded

- `tools/cloudflare-worker-json-relay/worker.js` must require
  `WORKER_AUTH_KEY`.
- The Worker must keep loop and self-fetch protections.
- The Worker must strip forwarded/client-IP and Cloudflare identity headers.
- The Worker must keep JSON/base64 request and response handling.
- `assets/apps_script/CodeCloudflareWorker.gs` must keep
  `HELPER_KIND = "apps_script_cloudflare_worker"`.
- The Apps Script bridge must keep separate client-facing `AUTH_KEY` and
  Worker-facing `WORKER_AUTH_KEY` secrets.
- The Apps Script bridge must keep single and batch forwarding to the Worker.
- The Apps Script bridge must keep safe-method replay fallback after
  `fetchAll` failure.
- Docs, Desktop backend tools, and release checklist must keep describing this
  as optional `apps_script` egress, not a separate native mode or full tunnel
  replacement.

## Parity Notes

- Apps Script helper, Cloudflare Worker tool, Desktop backend tooling, docs, and
  release checklist now share one static bridge contract.
- The bridge remains optional and does not change default runtime behavior.

## Concurrency / Split-Brain Notes

- No runtime code changed.
- The guard prevents the Worker and Apps Script bridge from evolving as two
  independent relay protocols.

## Cleanup

- No generated build output was intentionally created.
- Removed Python `__pycache__` directories created by the local verification
  pass:
  - `.github/scripts/__pycache__`
  - `tools/__pycache__`
- No Gradle command was run.

## Verification

- `python tools/check-cloudflare-worker-relay.py`
- `python tools/check-ci-local-sanity-parity.py`
- `python tools/check-doc-links.py`
- `python tools/run-repo-sanity.py --skip-node`
- `python tools/run-repo-sanity.py`
- `python tools/check-repo-cleanliness.py`
