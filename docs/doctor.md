# Doctor (guided diagnostics)

`mhrv-f doctor` is a first-run assistant that checks the most common failure modes and prints actionable fixes.

Run it any time you see:

- `504 Relay timeout`
- certificate errors (`NET::ERR_CERT_AUTHORITY_INVALID`)
- “it worked yesterday, now it times out”
- uncertainty about whether the relay is actually up

## How to run

```bash
./mhrv-f doctor
```

Or in the desktop UI:

- Click **Doctor** (next to **Test relay**) and read the **Recent log** panel.

## What it checks

- **Config warnings**: LAN-exposure guardrails, weak auth keys, `verify_ssl=false`, etc.
- **Mode sanity**: `apps_script` / `google_only` / `full`.
- **Apps Script pools**: checks that at least one enabled `account_groups` exists (when required).
- **MITM CA readiness** (non-`full` modes): verifies the CA exists and appears trusted.
- **End-to-end relay probe**: runs the same probe as `mhrv-f test` and classifies common failures.

## Common fixes the doctor will suggest

- **CA not trusted**: run `mhrv-f --install-cert` (as admin) or import `ca/ca.crt` into OS trust store.
- **Relay probe fails**:
  - verify `AUTH_KEY` matches between `config.json` and `Code.gs`
  - replace dead deployment IDs (re-deploy a “New version” in Apps Script)
  - scan a different `google_ip` / test the SNI pool
  - add backup accounts/IDs when quota exhaustion is the cause

