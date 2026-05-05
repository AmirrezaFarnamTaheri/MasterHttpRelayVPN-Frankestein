# Donor absorption matrix

This document classifies **reference-only** donor trees in the repository so
maintainers do not half-port features, binaries, or stale configs by accident.
It is the **canonical** absorption policy for:

- `mhr-cfw-main/`
- `Nova-Proxy-App-main/`
- `youtube-domain-fronting-patch-main/`

---

## Purpose and audience

| Audience | Use this doc to … |
|----------|-------------------|
| **Maintainers** | Decide port vs docs vs reject before touching donor code; keep CI hygiene and product source-of-truth aligned. |
| **Contributors** | Understand why donor folders exist but are not “part of the build”. |
| **Support / docs** | Point users at **supported** paths (`docs/relay-modes.md`, helpers under `tools/`, Apps Script under `assets/apps_script/`) instead of donor READMEs. |

---

## At-a-glance summary

| Tree | Upstream flavor | License (in-repo file) | Relation to `mhrv-f` | CI hygiene |
|------|-----------------|-------------------------|----------------------|------------|
| **`mhr-cfw-main/`** | Historical Python proxy + Worker + Apps Script | Follow upstream repo (mirror; see donor `README`) | Worker/GAS path **superseded** by hardened `tools/cloudflare-worker-json-relay/` + `CodeCloudflareWorker.gs` | **`check-repo-cleanliness.py`** does **not** descend into this root |
| **`Nova-Proxy-App-main/`** | Go / Wails desktop proxy (Nova) | **MIT** (`Nova-Proxy-App-main/LICENSE`) | **Concept donor only** — modes and UX ideas, not code import | Same skip |
| **`youtube-domain-fronting-patch-main/`** | Cronet libc patches for official YouTube APKs | **GPL-3.0** (`youtube-domain-fronting-patch-main/LICENSE`) | **Docs-only / reject code** — see [`youtube-external-patching.md`](youtube-external-patching.md) | Same skip |

Mirror stubs (human orientation): [`mhr-cfw-main/DONOR_REFERENCE.md`](../mhr-cfw-main/DONOR_REFERENCE.md), [`Nova-Proxy-App-main/DONOR_REFERENCE.md`](../Nova-Proxy-App-main/DONOR_REFERENCE.md), [`youtube-domain-fronting-patch-main/DONOR_REFERENCE.md`](../youtube-domain-fronting-patch-main/DONOR_REFERENCE.md).

---

## Policy

| Rule | Detail |
|------|--------|
| **Source of truth** | Product behavior lives in Rust (`src/`), Android (`android/`), maintained helpers (`assets/apps_script/`, `tools/*-relay/`, `tunnel-node/`), and **this repo’s** docs — not in donor folders. |
| **Hygiene / CI** | `tools/check-repo-cleanliness.py` **does not descend** into donor roots (see **Appendix A**). Trees remain in git for classification and diff review; scanners skip them to stay fast and avoid upstream binary/noise false positives. |
| **Binaries** | Donor executables, patched `.so` / Cronet blobs, bundled `warp`/VPN binaries, and similar are **quarantined**: they must never become implicit runtime dependencies of `mhrv-f`. |
| **GPL / patch stacks** | GPL-licensed patch workflows are **documentation-only** references here unless legal + architecture review explicitly approves vendoring (default: **do not**). |
| **MIT Nova tree** | MIT is permissive, but **license ≠ product fit**. Nova remains reference-only; copying Go/Wails modules without tests, threat modeling, and mode alignment is still **`reject`** for blind imports. |

---

## Status vocabulary

| Status | Meaning | Typical next step |
|--------|---------|-------------------|
| **`port_now`** | Approved for immediate implementation using project-native code and tests. | Open tracked task; implement in Rust/Android/docs with parity gates. |
| **`port_concept`** | Useful idea; implement later with a clean design (no blind copy-paste). | Record in roadmap (Route Advisor, Trust Center, Observatory, etc.). |
| **`docs_only`** | Capture concepts in project docs; **do not** vendor donor code. | Extend `docs/*`; link from [`docs/index.md`](index.md). |
| **`reject`** | Do not bring into the product (superseded, unsafe, wrong license surface, or split-brain). | Close path; point users/maintainers at canonical artifacts only. |
| **`quarantine`** | Reference material only; keep outside runtime/build expectations. | Keep README/`DONOR_REFERENCE.md`; never wire into `Cargo.toml`/Gradle/npm product deps. |

---

## Operational tooling matrix

| Tool / doc | Role |
|------------|------|
| **`tools/check-repo-cleanliness.py`** | Root hygiene walk; **prunes** donor roots (`DONOR_REFERENCE_ROOTS`) and archive dirs per script header. |
| **`tools/report-nova-proxy-config.py`** | Read-only JSON triage vs **`docs/config-registry.json`** (`--demo`, `--path`, optional `--no-nested`). |
| **`docs/cfw-reference-audit.md`** | Historical CFW donor audit (Worker/GAS ported concepts). |
| **`docs/youtube-external-patching.md`** | User-facing GPL Cronet patch context (no code vendoring). |
| **`docs/cloudflare-worker-json-relay.md`** | Maintained Worker bridge documentation. |

---

## `mhr-cfw-main` — Cloudflare Worker + Apps Script donor

### Role

Historical **MasterHttpRelay + Cloudflare Worker** snapshot. The **maintained**
integration is **not** this folder — see [`cloudflare-worker-json-relay.md`](cloudflare-worker-json-relay.md), [`tools/cloudflare-worker-json-relay/worker.js`](../tools/cloudflare-worker-json-relay/worker.js), [`assets/apps_script/CodeCloudflareWorker.gs`](../assets/apps_script/CodeCloudflareWorker.gs).

[`docs/cfw-reference-audit.md`](cfw-reference-audit.md) lists ported concepts; note it refers to **`core/*`** paths from an older layout — this mirror uses **`src/*`** for Python sources.

### Repository inventory (representative paths)

| Path (under `mhr-cfw-main/`) | Role | Absorption status | Risk if copied blindly |
|------------------------------|------|-------------------|-------------------------|
| `script/worker.js` | Worker egress | **`reject`** vendor | Open-fetch / weak-auth patterns fixed in maintained Worker |
| `script/Code.gs` | Apps Script hop | **`reject`** vendor | Superseded by `CodeCloudflareWorker.gs` |
| `src/proxy_server.py`, `src/mitm.py` | Python MITM relay | **`reject`** | Second runtime vs Rust `mhrv-f` |
| `src/domain_fronter.py` | SNI/fronting experimentation | **`docs_only`** | Rust `direct` / `fronting_groups` supersede |
| `src/cert_installer.py` | CA install / NSS notes | **`docs_only`** / **`port_concept`** (NSS depth) | Secrets UX / OS-specific installs |
| `src/h2_transport.py` | HTTP/2 experiments | **`docs_only`** | Observatory could surface H1/H2 later |
| `src/google_ip_scanner.py`, `src/constants.py`, `src/codec.py` | Supporting Python | **`docs_only`** | Logic duplicated differently in Rust |
| `src/lan_utils.py`, `src/logging_utils.py` | LAN / logging helpers | **`docs_only`** | Align with [`sharing-and-per-app-routing.md`](sharing-and-per-app-routing.md) |
| `main.py`, `setup.py`, `requirements.txt`, `run.bat`, `run.sh` | Launcher / packaging | **`reject`** | Desktop installer + `mhrv-f-ui` replace |
| `config.example.json` | Sample config | **`docs_only`** | Use root `config*.example.json` + registry |

### Detailed feature migration matrix

| Donor capability / artifact | Status | `mhrv-f` counterpart (today or planned) | Notes |
|-----------------------------|--------|----------------------------------------|-------|
| Worker JSON relay exit + Apps Script bridge | **`reject`** (vendor) + **`docs_only`** | `CodeCloudflareWorker.gs` + maintained Worker | Compatibility probe `kind = apps_script_cloudflare_worker`; see cfw audit |
| Firefox / NSS trust hints | **`port_concept`** | Doctor + future Trust Center | Donor CA installer referenced NSS; Rust installer differs |
| CA lifecycle UX (install / verify copy) | **`docs_only`** | [`safety-security.md`](safety-security.md), desktop CA flows | Extract checklist wording only |
| LAN listener explanation patterns | **`port_concept`** | LAN docs + Android LAN readiness | Must respect token + allowlist model |
| H1 vs H2 vs SNI pathway visibility | **`port_concept`** | Observatory / diagnostics | Must not invent protocol claims without measurements |
| WebSocket-centric tunnel (if present upstream) | **`docs_only`** | `full` + `tunnel-node` | Different architecture |
| Parallel Python MITM stack | **`reject`** | N/A | Canonical runtime is Rust |

### Explicit non-goals

- Do **not** add donor `requirements.txt` flows as supported setup.
- Do **not** revive donor `worker.js` / `Code.gs` as parallel “official” scripts.

---

## `Nova-Proxy-App-main` — Nova Proxy reference mirror

### Role

**Nova Proxy** is a separate MIT-licensed Go/Wails product (see donor `README.md`
feature tables — transparent vs MITM vs QUIC vs TLS-RF vs reverse proxy vs WARP).
This repo keeps a **trimmed mirror** for terminology and UX research.

### Repository inventory by subsystem

| Subsystem / path | Role | Absorption stance |
|------------------|------|-------------------|
| `main.go`, `go.mod`, `wails.json`, `winres/` | App shell | **`reject`** import |
| `proxy/*.go` | Core proxy (routing, TLS fragment, tun flow, CF pool, DoH, GFW list, Warp managers, cert verification hooks) | **`docs_only`** / **`port_concept`** per row below |
| `sysproxy/*.go`, `autostart_*.go` | OS proxy integration / autostart | **`port_concept`** (snapshot/restore UX on desktop) |
| `config/settings.json` | Checked-in sample settings | **`docs_only`** demo input for report tool |
| `sni-server/` | Separate Go + Worker snippet area | **`docs_only`** | Not wired into `mhrv-f` tunnel |
| `rules/` | Rules snapshot removed | **`quarantine`** | [`rules/README.md`](../Nova-Proxy-App-main/rules/README.md) |
| `LICENSE` | MIT | Inform licensing of **mirror only** | Does not authorize silent code merge |

### Capability matrix (Nova product → absorption)

Nova markets many modes; **`mhrv-f`** intentionally keeps four config modes (`apps_script`, `vercel_edge`, `direct`, `full`). Map Nova concepts accordingly:

| Nova concept (from donor README / layout) | Status | Native mapping / guidance |
|-------------------------------------------|--------|---------------------------|
| Transparent / DNS-focused path | **`docs_only`** | Compare to `direct` + DNS/fronting docs — not identical |
| MITM + local CA + custom SNI/ECH ideas | **`docs_only`** / **`port_concept`** | Align CA story with Trust Center; no ECH claims without engineering |
| QUIC / HTTP/3 replay path | **`docs_only`** | `block_quic` and UDP paths differ; see [`udpgw.md`](udpgw.md), relay docs |
| TLS-RF / fragmentation style tricks | **`docs_only`** | DPI circumvention taxonomy — **not** a promised `mhrv-f` feature |
| Reverse “Server” relay template `/{token}/{host}/{path}` | **`docs_only`** | Different from Apps Script JSON relay — do not confuse users |
| WARP / Masque integration | **`reject`** product port | External CLOUDFLARE WARP binaries & ToS surface — do not bundle |
| Auto routing / rule precedence | **`port_concept`** | Becomes **Route Advisor** + structured readiness — not Nova rule import |
| GeoIP / GeoSite style lists | **`docs_only`** | Maintenance burden; donor lists rot quickly |
| Cloudflare IP pool probing (`proxy/cf_pool.go`, `cloudflare_config` in settings) | **`port_concept`** | Observatory candidate — **no** hardcoded API keys, respectful probing |
| Certificate verification policy UI (`cert_verify` hooks) | **`port_concept`** | Trust Center — strict chain / allowed SANs **as diagnostics**, not bypass |
| Upstream DoH usage (`proxy/doh.go`) | **`docs_only`** | Compare with `tunnel_doh`, DNS bypass lists — unify docs vocabulary |
| System proxy toggle + persistence (`sysproxy/`) | **`port_concept`** | Desktop should state restore semantics explicitly |
| GFWList-driven defaults (`proxy/gfwlist.go`) | **`reject`** default import | Do not ship Nova lists as product defaults |

### `config/settings.json` (demo) structural notes

The checked-in demo uses top-level **`listen_port`** as a **string**, nested **`tun`**, **`cloudflare_config`** (preferred IPs, DoH URL, Warp toggles), and **`auto_routing`**. **`tools/report-nova-proxy-config.py --demo`** compares names against **`docs/config-registry.json`** and warns that **`listen_port`** collides by name with mhrv but differs by type (mhrv expects a JSON number).

### Explicit non-goals

- No **`go.sum`** / Nova modules in product build graph.
- No resurrection of **`proxy/usque.exe`** or megabyte **`rules/config.json`** snapshots (removed under hygiene policy).

---

## `youtube-domain-fronting-patch-main` — Cronet SNI patch stack

### Role

Upstream documents **GPLv3** patches targeting **`libcronet`** inside official YouTube / YouTube Music APKs (arm64-focused patch tables in donor `README.md`).

Canonical user-facing write-up: [`youtube-external-patching.md`](youtube-external-patching.md). Mirror pointer: [`youtube-domain-fronting-patch-main/DONOR_REFERENCE.md`](../youtube-domain-fronting-patch-main/DONOR_REFERENCE.md).

### Artifact and license matrix

| Artifact class | Status | Notes |
|----------------|--------|-------|
| Patch binaries / patched `libcronet` | **`reject`** (product) | GPL derivative **binary** redistribution decisions are explicit legal scope |
| Patch recipes / Morphe metadata URLs (donor README) | **`docs_only`** | Explain externally; do not automate fetch into repo |
| Concept “force SNI, keep Host/URL” | **`docs_only`** | Contrasts with browser MITM path under `mhrv-f` |

### Risk matrix (why default is docs-only)

| Risk dimension | Severity | Mitigation in `mhrv-f` |
|----------------|----------|----------------------|
| GPL obligations on combined works | **High** if vendored | Do **not** copy patch code; link externally |
| APK signing continuity | **High** | Sideload forks break Play update expectations — document only |
| Account / ToS | **Medium–High** | User responsibility — no endorsement |
| Cronet version drift per APK | **High maintenance** | Unsupported combination matrix |

### Supported alternatives (inside product scope)

Document in relay guides — details in [`relay-modes.md`](relay-modes.md), [`advanced-options.md`](advanced-options.md), [`youtube-external-patching.md`](youtube-external-patching.md):

| Need | Direction |
|------|-----------|
| YouTube in **browser** via relay | `apps_script` / `vercel_edge` / `direct` + CA |
| Reduced restricted-mode quirks | `youtube_via_relay` where applicable |
| Heavy video throughput | Prefer **`full`** tunnel versus per-chunk Apps Script |
| Official **app** carve-out | External GPL patch research only — not bundled here |

---

## Cross-cutting matrices

### Binary and large-artifact policy

| Artifact type | Policy |
|---------------|--------|
| `.exe`, `.dll`, patched `.so`, Cronet | **`quarantine`** / **`reject`** for shipping |
| Large rules / GeoIP blobs | **`reject`** in-tree |
| Donor `node_modules`/`.gradle` inside donor trees | Skipped by hygiene walk — still **do not commit** junk under donor roots |

### Split-brain prevention (canonical pointers)

| Concern | Canonical location |
|---------|---------------------|
| Config field meanings | [`config-registry.json`](config-registry.json) → generated [`config-registry.md`](config-registry.md) |
| Desktop save roots | `ConfigWire` + `check-config-wire-vs-registry.py` |
| Android preservation roots | `ownedKeys` + `check-android-owned-keys-list.py` + [`android-config-preservation.md`](android-config-preservation.md) |
| Worker relay security model | [`cloudflare-worker-json-relay.md`](cloudflare-worker-json-relay.md) |

### Maintainer checklist — touching any donor tree

1. Update **this matrix** if classification changes.
2. Add **`docs/changelog/`** maintainer entry + **`elevation_audit_roadmap_source.md`** Progress Log row.
3. Re-run **`python tools/run-repo-sanity.py`** when docs/tooling links change.
4. Never add donor paths to **`Cargo.toml`**, **`build.gradle.kts`**, or release packaging without explicit review.

---

## Next implementation hooks (project-native)

| Idea bucket | Target surface | Typical donor hint |
|-------------|----------------|---------------------|
| Route explanation | Route Advisor | Nova auto-routing tables |
| Trust / CA / chain clarity | Trust Center + Doctor | Hub doc [`trust-center.md`](trust-center.md); Nova cert verification UX; CFW NSS notes |
| Backend / edge health over time | Observatory + Backend Registry | Hub [`backend-registry.md`](backend-registry.md); Nova CF pool probing concept |
| OS proxy lifecycle | Desktop UX | Nova `sysproxy/` |
| YouTube honesty | Docs + future wizard (Batch 9) | GPL Cronet patch README |

**Batch 2 tooling scope:** absorption matrix + **`report-nova-proxy-config.py`** + **`youtube-external-patching.md`** + **`DONOR_REFERENCE.md`** stubs — **not** automatic imports.

---

## Appendix A — Hygiene skip roots

These directory names match **`DONOR_REFERENCE_ROOTS`** in `tools/check-repo-cleanliness.py` (verify when editing script):

- `mhr-cfw-main`
- `Nova-Proxy-App-main`
- `youtube-domain-fronting-patch-main`

---

## Appendix B — Related documentation index

| Doc | Topic |
|-----|-------|
| [`cfw-reference-audit.md`](cfw-reference-audit.md) | CFW donor audit detail |
| [`cloudflare-worker-json-relay.md`](cloudflare-worker-json-relay.md) | Maintained Worker bridge |
| [`youtube-external-patching.md`](youtube-external-patching.md) | GPL Cronet / official apps |
| [`relay-modes.md`](relay-modes.md) | Supported `mhrv-f` modes |
| [`android-config-preservation.md`](android-config-preservation.md) | Unknown-root JSON preservation |

---

## Changelog discipline

When classification changes: update **this file**, add **`docs/changelog/`** entry, add **`elevation_audit_roadmap_source.md`** Progress Log row.

Last reviewed: 2026-05-03 (matrix deepening — inventories, risk tables, Nova capability map, appendices)
