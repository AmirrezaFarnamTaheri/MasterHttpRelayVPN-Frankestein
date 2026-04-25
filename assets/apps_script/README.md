# Apps Script source (mirrored)

The file `Code.gs` next to this README matches the upstream Apps Script relay maintained by the original project. Upstream home: <https://github.com/masterking32/MasterHttpRelayVPN>.

This copy lives in our repo for two reasons:

1. **Survives upstream outages**: if the user is on a network where raw.githubusercontent.com is temporarily unreachable but they can clone or ZIP this repo, they still have the deploy-ready file.
2. **Pins what we tested against**: the relay protocol between `mhrv-f` and the script is informal; upstream changes can silently break us. Keeping a snapshot here lets us diff and see if a spec drift is responsible for any reported breakage.

All credit for the original `Code.gs` concept and protocol goes to [@masterking32](https://github.com/masterking32).

This repository’s `assets/apps_script/Code.gs` / `CodeFull.gs` are **maintained copies** with a few pragmatic hardening changes based on real-world reports:

- **Header privacy hardening**: we strip forwarded-IP and proxy chain headers, and we use a small **allowlist** of forwarded headers to reduce accidental identity leakage.
- **No obfuscated/minified variants**: obfuscated Apps Script in a public repo is a bad idea (hard to audit, easy to smuggle secrets). We intentionally keep the script readable.
- **Optional Telegram quota notifications (off by default)**: a tiny opt-in helper to warn when Apps Script usage approaches the daily limit.

If you're using mhrv-f, follow the deploy instructions in the script header. The only required edit is still `AUTH_KEY` — set it to a strong secret and reuse that exact string in your `mhrv-f` config.
