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
and Help & docs. The UI exposes mode summary cards, backend-tool open buttons,
CA install/removal, relay testing, Doctor diagnostics, live logs, profiles,
update checks, and LAN/per-app controls with copyable proxy endpoints.

- Main setup guide: [`README.md`](../README.md#setup-guide)
- Desktop UI reference: [`docs/ui-desktop.md`](ui-desktop.md)
- Windows installer package: [`docs/desktop-installer.md`](desktop-installer.md)
- Symptom troubleshooting: [`docs/troubleshooting.md`](troubleshooting.md)

### Android

The Android app runs the same Rust engine, adds a Compose UI, and uses
`VpnService` + `tun2proxy` for VPN mode.

- Android guide: [`docs/android.md`](android.md)
- Persian Android guide: [`docs/android.fa.md`](android.fa.md)
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
- Symptom decision tree: [`docs/troubleshooting.md`](troubleshooting.md)
- Safety and CA lifecycle: [`docs/safety-security.md`](safety-security.md)
- Advanced knobs: [`docs/advanced-options.md`](advanced-options.md)
- Field notes and edge candidates: [`docs/field-notes.md`](field-notes.md)
- Glossary: [`docs/glossary.md`](glossary.md)

## Offline Or Blocked GitHub Releases

If the GitHub Releases page is blocked on your network, use
[`releases/README.md`](../releases/README.md) for ZIP/clone fallback and hash
verification guidance.
