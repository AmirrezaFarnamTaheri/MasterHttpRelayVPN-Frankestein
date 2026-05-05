# Backend registry (deploy map)

This document is the **maintainer-facing registry** of relay **backends**: what
each artifact is for, how health is checked today, and where deeper docs live.

It complements:

- **[`docs/parity-matrix.md`](parity-matrix.md)** — which **modes** use which backend families on Desktop/Android/runtime.
- **[`docs/config-registry.md`](config-registry.md)** — serialized **`Config`** fields.

Product UI will evolve toward explicit **backend cards** (roadmap Batch **4**);
this page is the canonical narrative until those panels ship.

---

## Scope

| In registry | Out of registry |
|-------------|-----------------|
| Apps Script helpers (`Code*.gs`) | Raw donor trees ([`donor-absorption-matrix.md`](donor-absorption-matrix.md)) |
| Cloudflare Worker JSON exit | Generic CDN theory |
| Vercel / Netlify **JSON** relays (`tools/*-json-relay`) | User’s VPS SSH hardening |
| **tunnel-node** (full tunnel) | External Xray/V2Ray cores themselves |
| Vercel / Netlify **XHTTP** helpers | XHTTP wire protocol spec |

---

## Identifier conventions

| ID | Meaning |
|----|---------|
| **`apps_script`** | Native mode using Google Apps Script relay helpers. |
| **`serverless_json`** | Native **`vercel_edge`** mode talking to Vercel or Netlify **JSON** relay (config field remains **`vercel`**). |
| **`tunnel_node`** | **`full`** mode path involving **`tunnel-node`** + **`CodeFull.gs`**. |

Apps Script deployments expose a **`kind`** string via **`?compat=1`** (must match helper sources):

| Helper source | `HELPER_KIND` / `kind` |
|---------------|-------------------------|
| [`assets/apps_script/Code.gs`](../assets/apps_script/Code.gs) | `apps_script` |
| [`assets/apps_script/CodeCloudflareWorker.gs`](../assets/apps_script/CodeCloudflareWorker.gs) | `apps_script_cloudflare_worker` |
| [`assets/apps_script/CodeFull.gs`](../assets/apps_script/CodeFull.gs) | `apps_script_full` |

---

## Registry table (native backends)

Columns: **Backend** | **Modes** | **Primary config** | **Deploy / artifact** | **Health / probe (today)** | **Docs** | **Desktop** | **Android** |

| Backend | Modes | Primary config | Deploy / artifact | Health / probe | Docs | Desktop | Android |
|---------|-------|----------------|-------------------|----------------|------|---------|---------|
| **Apps Script relay** | `apps_script` | `account_groups[]`, `auth_key` per group | Deploy **`Code.gs`** web app; Anyone access | **`?compat=1`** on `/exec`; **`mhrv-f test`**; relay logs | [`README.md`](../README.md), [`relay-modes.md`](relay-modes.md), [`assets/apps_script/README.md`](../assets/apps_script/README.md) | Setup + Test relay | Deploy IDs + keys |
| **Apps Script + CF Worker exit** | `apps_script` | Same + Worker URL/auth in Apps Script properties | **`CodeCloudflareWorker.gs`** + [`tools/cloudflare-worker-json-relay/worker.js`](../tools/cloudflare-worker-json-relay/worker.js) | **`kind = apps_script_cloudflare_worker`** on compat probe; Worker env secrets | [`cloudflare-worker-json-relay.md`](cloudflare-worker-json-relay.md) | Backend tools + docs | Mirror desktop concept |
| **Apps Script full relay** | `full` | `account_groups[]` + tunnel URL/auth | **`CodeFull.gs`** + public **`tunnel-node`** | Doctor + optional **`doctor --tunnel-node-url`** → **`/health/details`** | [`relay-modes.md`](relay-modes.md), [`tunnel-node/README.md`](../tunnel-node/README.md), [`udpgw.md`](udpgw.md) | Full tunnel readiness | Same |
| **Serverless JSON (Vercel or Netlify)** | `vercel_edge` | `vercel.base_url`, `vercel.auth_key`, `vercel.relay_path`, TLS flags | [`tools/vercel-json-relay`](../tools/vercel-json-relay/README.md) or [`tools/netlify-json-relay`](../tools/netlify-json-relay/README.md) | **`mhrv-f test`** (JSON envelope); TLS/DNS failures in Doctor | [`vercel-json-relay.md`](vercel-json-relay.md), [`netlify-json-relay.md`](netlify-json-relay.md) | Setup Serverless JSON | Base URL + secret |
| **tunnel-node service** | `full` only (path component) | Tunnel URL + auth + UDP expectations | Rust binary + VPS deploy per **`tunnel-node/README.md`** | **`GET /health/details`** (TLS-capable origin); CLI Doctor integration | [`tunnel-node/README.md`](../tunnel-node/README.md), [`doctor.md`](doctor.md) | Doctor flags | JNI parity subset |

**Direct mode** (`direct`) uses **fronting_groups** and Google bootstrap IPs rather than a separate deployable “relay backend” row — document under [`fronting-groups.md`](fronting-groups.md) and parity matrix.

---

## Adjacent tools (not selectable runtime backends)

These ship in-repo but map to **external** clients or hybrid setups:

| Tool | Purpose | Docs |
|------|---------|------|
| **Vercel XHTTP relay** | Helper URL for Xray/V2Ray XHTTP presets | [`vercel-xhttp-relay.md`](vercel-xhttp-relay.md) |
| **Netlify XHTTP relay** | Same for Netlify edge | [`netlify-xhttp-relay.md`](netlify-xhttp-relay.md) |
| **Vercel XHTTP Node relay** | Alternate deploy flavor | [`tools/vercel-xhttp-relay-node`](../tools/vercel-xhttp-relay-node/README.md) if present |

Treat mis-deployment (protection pages, wrong env vars) as **backend-adjacent**
failures — Doctor/`test` messages should stay aligned with these docs.

---

## Unified health result (design target)

Today, probes return different shapes (**compat JSON**, **`test`** stdout,
**`/health/details`** tunnel-node JSON). The target contract for Observatory /
Backend Registry UI:

| Field | Type | Meaning |
|-------|------|---------|
| `backend_id` | string | Stable ID from table above (e.g. `apps_script`, `serverless_json`, `tunnel_node`). |
| `status` | enum | `ok`, `degraded`, `fail`, `skipped`. |
| `latency_ms` | optional number | Round-trip where meaningful. |
| `failure_reason` | optional string | Short machine-readable slug + human text in UI. |
| `config_snapshot_hash` | optional string | Hash/fingerprint of config subset used for probe (`ConfigWire` fingerprint pattern). |
| `docs_anchor` | string | Markdown path + heading anchor for repair. |
| `repair_hint_id` | optional string | Readiness / Doctor correlation ID. |

Implementations should **invalidate** cached results when config changes mid-session (Trust Center parity principle).

---

## Failure modes cheat-sheet

| Symptom cluster | Likely backend | First checks |
|-----------------|----------------|--------------|
| HTML login page instead of relay JSON | Apps Script access / Worker routing | Deploy access Anyone; Worker URL/auth |
| HTTP 413 / body too large | Serverless JSON relay | `max_body_bytes`, docs for relay limits |
| TLS handshake errors on relay hostname | Any HTTPS backend | Certificate pinning expectations, `verify_tls`, DNS |
| Tunnel connects but UDP apps fail | tunnel-node / udpgw | SOCKS5 UDP listener, [`udpgw.md`](udpgw.md) |
| Good compat probe but browsing fails | Local CA / MITM path | [`trust-center.md`](trust-center.md) |

---

## See also

- [`doctor.md`](doctor.md) — structured checks + **`--tunnel-node-url`**
- [`release-checklist.md`](release-checklist.md) — helper compat review
- [`trust-center.md`](trust-center.md) — MITM trust surfaces

Last reviewed: 2026-05-03
