# Doctor - Guided Diagnostics

`mhrv-f doctor` checks the common first-run and daily-use failure modes and
prints actionable fixes. It is available in the CLI and in the desktop UI.

## Run It

```bash
./mhrv-f doctor
```

In the desktop UI, click **Doctor** and read the **Recent log** panel. Use
**Doctor + Fix** for safe local fixes such as CA generation/install attempts.

## When to Use It

- `504 Relay timeout`
- certificate errors such as `NET::ERR_CERT_AUTHORITY_INVALID`
- a relay worked yesterday but now times out
- `vercel_edge` returns HTML/protection pages instead of JSON
- you are not sure whether the relay credentials are correct
- you changed modes and want a sanity check

## What It Checks

- **Config warnings**: weak auth keys, LAN exposure guardrails,
  `verify_ssl=false`, serverless relay TLS verification warnings, and similar risky
  settings.
- **Mode sanity**: `apps_script`, `vercel_edge`, `direct`, or `full`.
- **Apps Script pools**: enabled account groups and deployment counts when
  Apps Script credentials are required.
- **Serverless JSON config**: required `vercel.base_url`, `vercel.auth_key`,
  and endpoint shape in `vercel_edge` mode.
- **MITM CA readiness**: CA file generation and OS trust status for modes that
  decrypt HTTPS locally.
- **Relay probe**: the same JSON HTTP probe as `mhrv-f test` for `apps_script`
  and `vercel_edge`.

## Common Fixes

### CA Not Trusted

Run:

```bash
./mhrv-f --install-cert
```

On Windows, run from an Administrator shell if the user store install is not
enough. Firefox may need restart or NSS/enterprise-roots handling; the installer
attempts this automatically where possible.

### Apps Script Probe Fails

Check:

- `AUTH_KEY` in config matches `Code.gs`
- deployment access is **Anyone**
- deployment ID is current
- your account has quota left
- `google_ip` and `front_domain` still work from your network

Useful commands:

```bash
./mhrv-f test-sni
./mhrv-f scan-ips
```

### Serverless JSON Probe Fails

Check:

- `vercel.base_url` is the deployed app origin, for example
  `https://your-project.vercel.app` or `https://your-site.netlify.app`
- `vercel.relay_path` is usually `/api/api`
- `vercel.auth_key` matches platform environment variable `AUTH_KEY`
- the Vercel/Netlify project was redeployed after changing environment variables
- Vercel Deployment Protection is disabled, or Netlify `/api/api` routes to the
  Edge Function

HTML responses usually mean platform auth/protection or a routing page is in
front of the function. The native client expects JSON.

### Full Mode

Doctor does not use `mhrv-f test` as proof for `full` mode. Verify full mode by
starting the tunnel, browsing through it, and checking that an IP-check page
shows the tunnel-node public IP.
