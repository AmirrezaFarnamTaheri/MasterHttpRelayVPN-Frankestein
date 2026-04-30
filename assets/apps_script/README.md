# Apps Script Relays

This directory contains deploy-ready Google Apps Script files used by `mhrv-f`.
Use the file that matches the mode you selected in the client.

## Files

- `Code.gs`: normal `apps_script` relay.
- `CodeFull.gs`: full-tunnel relay channel for `tunnel-node`.
- `CodeCloudflareWorker.gs`: Apps Script entry that sends the final fetch
  through a private Cloudflare Worker.

## Deployment Checklist

1. Open <https://script.google.com> and create a project.
2. Delete the placeholder code.
3. Paste the full contents of the script you need.
4. Set the required secret in the script:
   - `AUTH_KEY` for `Code.gs`
   - `TUNNEL_AUTH_KEY` and `TUNNEL_URL` for `CodeFull.gs`
   - `AUTH_KEY`, `WORKER_URL`, and `WORKER_AUTH_KEY` for
     `CodeCloudflareWorker.gs`
5. Deploy as a Web app:
   - Execute as: **Me**
   - Who has access: **Anyone**
6. Copy the Deployment ID into the matching `account_groups[].script_ids`
   entry in `mhrv-f`.

## Safety Notes

- Use long random secrets. Do not reuse public examples.
- Redeploy as a new version after changing a script constant.
- Do not publish a relay with a blank or example secret.
- Keep `DIAGNOSTIC_MODE=false` except while debugging setup problems.
- Leave `CACHE_SPREADSHEET_ID` blank unless you intentionally want small public
  GET responses cached in a Google Sheet you control.

For the Cloudflare Worker exit path, see
[`docs/cloudflare-worker-json-relay.md`](../../docs/cloudflare-worker-json-relay.md).
