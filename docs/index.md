# Start here — MasterHttpRelayVPN-Frankestein (`mhrv-f`)

This is the docs hub for **MasterHttpRelayVPN-Frankestein**. In commands, file paths, and binaries, the project uses the short name `mhrv-f`.

## Choose your goal

### 1) Desktop: browse through a local proxy (most common)
- Use the desktop UI: `mhrv-f-ui` (recommended first-run) or the CLI `mhrv-f`.
- Follow the main setup guide in [`README.md`](../README.md#setup-guide).
- If something fails, jump to [Troubleshooting](#troubleshooting).

### 2) Android: system-wide routing (VPN/TUN) or proxy-only
- Full Android guide: [`docs/android.md`](android.md) (English) / [`docs/android.fa.md`](android.fa.md) (فارسی)
- The Android app runs the same Rust engine and wraps it with a VPN (TUN) bridge.

### 3) Full tunnel (advanced): end-to-end tunnel with a tunnel-node
- Use this when you operate a tunnel-node and want full device traffic to traverse the tunnel.
- This path supports **end-to-end UDP** in Full Tunnel mode (QUIC/HTTP3, DNS, STUN) on recent releases.
- See: [`tunnel-node/README.md`](../tunnel-node/README.md)

## Verify it works (fast checks)
- Desktop UI: click **Test relay** and then **Doctor**.
- CLI:

```bash
./mhrv-f test
./mhrv-f doctor
```

## Troubleshooting
- Guided diagnostics: [`docs/doctor.md`](doctor.md) / [`docs/doctor.fa.md`](doctor.fa.md)
- Symptom-driven decision tree: [`docs/troubleshooting.md`](troubleshooting.md)

## Safety & security (plain language)
- What the CA does, what is decrypted, and how to uninstall safely: [`docs/safety-security.md`](safety-security.md)

## Glossary
- Terms used across the UI and docs: [`docs/glossary.md`](glossary.md)

