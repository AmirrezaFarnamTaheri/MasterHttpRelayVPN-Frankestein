# Glossary

- **Apps Script relay**: Your Google Apps Script deployment (the `/exec` URL) that fetches real sites on your behalf.
- **AUTH_KEY / auth_key**: Shared secret between `Code.gs` and your client config. If it’s wrong, the relay fails immediately.
- **Deployment ID**: The long ID in the Apps Script deployment URL. You can paste either the full `/exec` URL or the bare ID.
- **Domain fronting**: A technique where the outer TLS connection looks like it targets a permitted domain (SNI), but the inner HTTP `Host` carries the real destination.
- **SNI (Server Name Indication)**: The hostname sent in the TLS handshake. This project rotates a pool of SNI names to survive filtering.
- **google_ip**: The Google edge IPv4 address you dial. Wrong values are a top cause of timeouts.
- **front_domain**: The hostname used as TLS SNI on the outer connection (often `www.google.com`).
- **MITM / CA (certificate authority)**: In Apps Script (MITM) mode, the client decrypts HTTPS locally, relays bytes, then encrypts again. This requires trusting a local CA on your device.
- **Full tunnel**: A mode where traffic is tunneled end-to-end via Apps Script + your tunnel-node, without installing a local MITM CA on the client device.
- **tunnel-node**: The remote server component you operate for Full Tunnel mode.
- **UDP (Full Tunnel)**: In Full Tunnel mode, UDP can traverse the tunnel end-to-end on newer releases (important for QUIC/HTTP3, DNS, STUN).
- **Doctor**: `mhrv-f doctor` (and the UI Doctor button) — guided diagnostics that checks common failure modes and prints actionable fixes.
- **Range-parallel downloads**: A performance feature for large GETs: probe with a small `Range`, then fetch the rest in parallel chunks and stitch back into a full `200 OK` for clients that never requested a range.
- **Quota (Apps Script)**: Google Apps Script enforces daily request limits; heavy pages and video chunking can exhaust it. Mitigations: multiple deployment IDs, multiple account groups, and optionally rate limiting / lower fan-out.

