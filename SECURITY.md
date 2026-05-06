# Security Policy

`mhrv-f` is a networking tool that touches local proxying, MITM certificates,
Android VPN routing, backend helper scripts, support bundles, and release
artifacts. Treat security reports and trust-model changes carefully.

## Reporting Vulnerabilities

Please do not publish exploitable details in a public issue before maintainers
have had a chance to respond.

Use a private channel when available on the repository host. If private
reporting is not configured, open a minimal public issue that says a security
report is available and avoids proof-of-concept details, secrets, endpoints, or
user-identifying logs.

Include:

- affected platform: Desktop, Android, CLI, Apps Script, Cloudflare Worker,
  serverless relay, tunnel-node, release workflow, docs;
- affected version or commit;
- impact summary;
- reproduction steps with secrets removed;
- whether user config, CA material, auth keys, LAN tokens, signing material, or
  support bundles are involved.

## Supported Versions

The latest GitHub Release is the supported public build. Older releases may be
referenced for rollback, but fixes should normally ship as a new release rather
than by replacing artifacts in-place.

## Secret Handling

Never post raw:

- Apps Script deployment IDs when avoidable;
- `auth_key`;
- serverless `AUTH_KEY`;
- LAN tokens;
- upstream SOCKS5 credentials;
- Android signing passwords;
- private backend URLs with credentials;
- raw support bundles without review.

Use the redacted support-bundle and Android support snapshot paths where
possible, then review before sharing.

## Trust Model Caveats

- Local MITM modes require a local CA and can expose decrypted traffic on the
  local machine.
- Android apps may reject user-installed CAs even when browsers work.
- LAN sharing exposes local listeners beyond the current device.
- Full tunnel depends on tunnel-node and backend helper auth.
- GitHub Releases and `SHA256SUMS.txt` are the canonical release source.
- Telegram posts are optional mirrors, not authority.

## Security-Sensitive Changes

Changes in these areas need extra review and explicit changelog/roadmap notes:

- certificate generation, install, removal, or trust detection;
- support-bundle or log redaction;
- Android VPN lifecycle and permissions;
- exposed listeners, LAN tokens, split tunneling, and per-app routing;
- Apps Script / Cloudflare / serverless / tunnel-node auth;
- release workflow, signing, checksums, and rollback policy.

Run:

```powershell
python tools\run-repo-sanity.py
python tools\check-release-governance.py
python tools\check-repo-governance.py
```
