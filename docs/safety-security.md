# Safety & security (plain language)

## What this tool does (and why a CA is involved)
In Apps Script (MITM) mode, **MasterHttpRelayVPN-Frankestein** decrypts HTTPS *locally on your device*, relays the bytes through your Apps Script deployment, then encrypts again toward the real destination.

That is why it needs a **local certificate authority (CA)** that your browser can trust.

## What is decrypted, and where
- **Decrypted on your device**: in MITM mode, HTTPS plaintext is visible to the local process (like any debugging proxy).
- **Not decrypted by your ISP**: your ISP sees an outer TLS connection that looks like ordinary Google traffic.

## Your CA files
In your config directory you will see:
- `ca/ca.crt` — public certificate (safe to share only if you understand the risk of others trusting it)
- `ca/ca.key` — private key (**never share, never commit, never upload**)

If someone gets `ca.key` and you (or your devices) trust the corresponding CA, they can impersonate sites for you.

## Install / repair / remove
- **Install**: use the launcher (`run.bat` / `run.command` / `run.sh`), click **Install CA** in the desktop UI, or run `mhrv-f --install-cert`.
- **Repair**: if you see cert errors, reinstall and re-trust the CA; then re-run Doctor.
- **Remove (uninstall)**: click **Remove CA** in the desktop UI or run `mhrv-f --remove-cert`.

Removal is deliberately conservative:

- It attempts to remove the CA from the OS trust store first.
- It attempts Firefox/NSS cleanup by deleting the same CA nickname from discovered browser profiles.
- It deletes the local `ca/` directory only after OS trust no longer appears active.
- It does not delete your `config.json`, Apps Script deployment IDs, Vercel settings, tunnel-node settings, or logs.

If removal fails, rerun the command from an elevated shell (`sudo` / Administrator). If it still fails, remove the `mhrv-f` / `MasterHttpRelayVPN` CA manually from the OS certificate manager, then delete the app config folder's `ca/` directory.

## Android-specific note (Android 7+)
Android allows each app to opt out of user-installed CAs. Browsers usually opt in; banking/chat apps often do not. This is normal Android behavior. Use proxy-only mode or split tunneling for those apps.
