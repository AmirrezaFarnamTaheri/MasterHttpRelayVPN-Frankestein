# Start Here - MasterHttpRelayVPN-Frankestein (`mhrv-f`)

This is the documentation hub for **MasterHttpRelayVPN-Frankestein**. The
project name is long; commands, binary names, config paths, and logs use the
short name `mhrv-f`.

The safest way to approach the project is:

1. Pick the platform.
2. Pick the relay mode.
3. Deploy only the backend required by that mode.
4. Run diagnostics before changing advanced knobs.
5. Use per-app or LAN sharing only after the local path works.

## Fast Path

### Desktop

Use `mhrv-f-ui` for first setup. It includes a first-run wizard, a persistent
top control panel, and task-focused tabs for Setup, Network, Advanced, Monitor,
Trust, and Help & docs. The UI exposes mode summary cards, backend-tool open
buttons, CA install/removal, relay testing, Doctor diagnostics, live logs,
profiles, update checks, LAN/per-app controls with copyable proxy endpoints,
and Trust Center/support-bundle preview state.

- Main setup guide: [`README.md`](../README.md#setup-guide)
- Desktop UI reference: [`docs/ui-desktop.md`](ui-desktop.md)
- Windows installer package: [`docs/desktop-installer.md`](desktop-installer.md)
- Symptom troubleshooting: [`docs/troubleshooting.md`](troubleshooting.md)

### Android

The Android app runs the same Rust engine, adds a Compose UI, and uses
`VpnService` + `tun2proxy` for VPN mode. Its main screen now mirrors the
desktop/CLI trust vocabulary with a compact Trust Center card for CA status,
Android user-CA limits, release-signing continuity, and support-data sharing
discipline.

- Android guide: [`docs/android.md`](android.md)
- Android unknown-root preservation (`ownedKeys`, drift gates): [`docs/android-config-preservation.md`](android-config-preservation.md)
- Persian Android guide: [`docs/android.fa.md`](android.fa.md)
- Android hard-coded copy inventory:
  [`docs/android-hardcoded-copy-inventory.md`](android-hardcoded-copy-inventory.md)
- Android redacted support snapshot schema:
  [`docs/android-support-snapshot.md`](android-support-snapshot.md)
- Release APK signing policy (committed keystore, CI authority):
  [`docs/android-signing.md`](android-signing.md)
- Per-app routing and LAN sharing:
  [`docs/sharing-and-per-app-routing.md`](sharing-and-per-app-routing.md)

## Mode Decision

| Need | Choose | Backend to deploy | CA needed |
|---|---|---|---|
| Free classic browser proxy | `apps_script` | `assets/apps_script/Code.gs` | Yes |
| Apps Script with Cloudflare egress | `apps_script` | `CodeCloudflareWorker.gs` + Worker | Yes |
| No-VPS serverless alternative | `vercel_edge` | Vercel or Netlify JSON relay | Yes |
| Reach setup pages or tested CDN-fronted targets first | `direct` | optional `fronting_groups` | Yes for HTTPS browsing |
| Full tunnel without local MITM | `full` | `CodeFull.gs` + `tunnel-node` VPS | No local MITM |
| Xray/V2Ray XHTTP front | external tool | Netlify/Vercel XHTTP helper + XHTTP backend | handled by Xray/V2Ray |

Read the detailed comparison before committing to a path:
[`docs/relay-modes.md`](relay-modes.md).

## Backend Guides

- Backend registry (deploy map, health probes, compat `kind`s):
  [`docs/backend-registry.md`](backend-registry.md)
- Apps Script setup:
  [`README.md`](../README.md#step-1--deploy-the-apps-script-relay-one-time)
- Vercel Edge JSON setup:
  [`docs/vercel-json-relay.md`](vercel-json-relay.md)
- Netlify Edge JSON setup:
  [`docs/netlify-json-relay.md`](netlify-json-relay.md)
- Direct fronting groups:
  [`docs/fronting-groups.md`](fronting-groups.md)
- Cloudflare Worker JSON exit:
  [`docs/cloudflare-worker-json-relay.md`](cloudflare-worker-json-relay.md)
- CFW donor audit:
  [`docs/cfw-reference-audit.md`](cfw-reference-audit.md)
- Full tunnel server:
  [`tunnel-node/README.md`](../tunnel-node/README.md)
- UDP/udpgw in full mode:
  [`docs/udpgw.md`](udpgw.md)
- Vercel XHTTP helper:
  [`docs/vercel-xhttp-relay.md`](vercel-xhttp-relay.md)
- Netlify XHTTP helper:
  [`docs/netlify-xhttp-relay.md`](netlify-xhttp-relay.md)
- Platform alternatives and migration:
  [`docs/platform-alternatives.md`](platform-alternatives.md)

## Verify It Works

Desktop UI:

1. Save config.
2. Click **Doctor**.
3. Click **Test relay** for `apps_script` or `vercel_edge`.
4. Start the proxy and browse through the configured HTTP/SOCKS port.
5. For `full` mode, verify with a public IP-check page and tunnel-node logs.

CLI:

```bash
./mhrv-f doctor
./mhrv-f trust-center
./mhrv-f test
./mhrv-f test-sni
./mhrv-f scan-ips
```

`mhrv-f test` is a JSON relay probe. It intentionally refuses `direct` and
`full` because those paths are verified differently.

## Local Routing

- Desktop per-app routing is explicit app proxy opt-in.
- Android VPN mode has native app splitting.
- Android Proxy-only mode is manual per-app proxy opt-in.
- Desktop/phone LAN sharing exposes HTTP/SOCKS listeners to trusted devices.

Use the full guide before enabling LAN exposure:
[`docs/sharing-and-per-app-routing.md`](sharing-and-per-app-routing.md).

## Troubleshooting And Reference

- Guided diagnostics: [`docs/doctor.md`](doctor.md)
- Shared Doctor JSON contract:
  [`docs/doctor-json-contract.md`](doctor-json-contract.md)
- Trust Center CLI/UI/bundle snapshot: [`docs/trust-center.md`](trust-center.md)
- Shared live status/stats JSON contract:
  [`docs/status-stats-json-contract.md`](status-stats-json-contract.md)
- Shared readiness IDs and repair targets: [`docs/readiness-matrix.md`](readiness-matrix.md)
- Canonical config registry (field metadata): [`docs/config-registry.md`](config-registry.md)
- Config parity matrix (field × surface): [`docs/config-parity-matrix.md`](config-parity-matrix.md)
- Mode/backend parity matrix (surface support): [`docs/parity-matrix.md`](parity-matrix.md)
- Platform defaults (Rust vs Android, canonical JSON + generated table):
  [`docs/platform-defaults.md`](platform-defaults.md)
- Symptom decision tree: [`docs/troubleshooting.md`](troubleshooting.md)
- Safety and CA lifecycle: [`docs/safety-security.md`](safety-security.md)
- Advanced knobs: [`docs/advanced-options.md`](advanced-options.md)
- Field notes and edge candidates: [`docs/field-notes.md`](field-notes.md)
- Glossary: [`docs/glossary.md`](glossary.md)
- Maintainer release checklist:
  [`docs/release-checklist.md`](release-checklist.md)
- Maintainer drift tools (Python generators and CI parity gates):
  [`tools/README.md`](../tools/README.md) — one-command mirror of CI:
  `python tools/run-repo-sanity.py`
- Verification profiles by change type:
  [`docs/verification-profiles.md`](verification-profiles.md)
- Change-impact checklist by touched surface:
  [`docs/change-impact-checklist.md`](change-impact-checklist.md)
- Tooling source map for guarded/generated docs:
  [`docs/tooling-source-map.md`](tooling-source-map.md)
- Local workspace cleanup (build caches and regenerable dirs):
  [`docs/workspace-cleanup.md`](workspace-cleanup.md)
- Donor reference trees — absorption matrix (what to port vs quarantine):
  [`docs/donor-absorption-matrix.md`](donor-absorption-matrix.md)
- Official YouTube apps vs browser / external Cronet patch research (docs-only):
  [`docs/youtube-external-patching.md`](youtube-external-patching.md)
- Maintainer batch / audit changelogs:
  [`docs/changelog/index.md`](changelog/index.md)
- Release/changelog hub: [`CHANGELOG.md`](../CHANGELOG.md)
- Contributing guide: [`CONTRIBUTING.md`](../CONTRIBUTING.md)
- Security policy: [`SECURITY.md`](../SECURITY.md)
- Ownership notes: [`docs/ownership.md`](ownership.md)
- Architecture decision records: [`docs/adr/README.md`](adr/README.md)
- Versioning policy: [`docs/versioning-policy.md`](versioning-policy.md)
- Rollback policy: [`docs/rollback-policy.md`](rollback-policy.md)

## Offline Or Blocked GitHub Releases

If the GitHub Releases page is blocked on your network, use
[`releases/README.md`](../releases/README.md) for ZIP/clone fallback and hash
verification guidance.
