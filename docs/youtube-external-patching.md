# Official YouTube apps and external Cronet patching

`mhrv-f` is built around **browser or proxy-aware apps** that honor your local HTTP/SOCKS MITM path (desktop) or VPN/TUN routing (Android). **Official Google YouTube and YouTube Music apps** often use **Cronet** and native stacks that **do not** behave like Chrome — TLS fingerprinting, pinning, and HTTP/3 paths differ.

This page documents **external** research tooling only. **No Cronet patch code from donors is vendored into `mhrv-f`** (GPL, binary patching, update/signing risks).

## Why this is separate from `mhrv-f`

| Topic | Browser via `mhrv-f` | Official YouTube / YT Music app |
|------|----------------------|----------------------------------|
| TLS stack | Typical browser MITM visibility | Cronet inside the APK |
| Updates | Browser updates independently | App updates replace native libs |
| Policy | Document relay modes + CA trust | Third-party patches void vendor assumptions |

For relay modes and quotas affecting **`googlevideo.com`** in **browser** tabs, see [`relay-modes.md`](relay-modes.md) and [`advanced-options.md`](advanced-options.md) (`youtube_via_relay`, full tunnel, etc.).

## External reference (donor mirror in this repo)

The folder **`youtube-domain-fronting-patch-main/`** is a **read-only mirror** for classification — see [`donor-absorption-matrix.md`](donor-absorption-matrix.md).

Upstream projects (for example tooling that applies **GPL-licensed** patches to **`libcronet`**) describe forcing TLS **SNI** while leaving URL / HTTP Host unchanged so DPI sees an allowed name. That is **not** integrated here because:

1. **License**: GPLv3 obligations apply to derivative works — incompatible with absorbing patch blobs into this codebase without a deliberate legal/design decision.
2. **Safety**: Patched APKs break signature continuity with Play updates and may affect account/device trust policies users rely on.
3. **Maintenance**: Each YouTube APK version needs matching patch binaries; `mhrv-f` cannot ship or endorse that treadmill.

Users researching that route should treat it as **personal experimentation**, read upstream licenses, and accept sideload/update/signing trade-offs themselves.

## Supported paths inside `mhrv-f`

- **Browser YouTube** through local proxy/VPN modes documented in relay guides.
- **`youtube_via_relay`** and related knobs where applicable (browser/App Script paths — see registry and Android UI copy).
- **`full` tunnel** when you want device-wide egress without relying on per-app Cronet behavior.

## Related docs

- Donor classification: [`donor-absorption-matrix.md`](donor-absorption-matrix.md)
- Android preservation of advanced JSON (not Cronet patches): [`android-config-preservation.md`](android-config-preservation.md)

Last reviewed: 2026-05-03
