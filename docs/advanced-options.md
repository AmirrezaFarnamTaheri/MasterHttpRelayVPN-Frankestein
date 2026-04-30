# Advanced Options

This page explains the config knobs exposed by the desktop UI, Android app, and
`config.json`. Defaults are intentionally conservative. Change one thing at a
time and use Doctor/Test after each meaningful change.

## Mode and Relay Selection

### `mode`

Allowed values:

- `apps_script`: local proxy + local MITM + Google Apps Script fetch relay.
- `vercel_edge`: local proxy + local MITM + Vercel or Netlify Edge JSON fetch
  relay.
- `direct`: no-relay SNI-rewrite mode for Google and verified fronting groups.
- `full`: full tunnel through Apps Script plus tunnel-node.

See [`docs/relay-modes.md`](relay-modes.md) for the decision guide.

### `account_groups[]`

Used by Apps Script and full-tunnel paths. Each enabled group is normally one
Google account/quota pool.

Important fields:

- `label`: human-readable name for logs/UI.
- `enabled`: disables a group without deleting it.
- `weight`: relative selection weight.
- `auth_key`: must match that account's Apps Script `AUTH_KEY`.
- `script_ids`: deployment IDs or deployment URLs.

More accounts and deployment IDs increase resilience and quota capacity.

Use this mental model:

- One group normally means one Google account, one `AUTH_KEY`, and one quota
  pool.
- Multiple `script_ids` inside that group are deployment endpoints for the same
  identity. They help rotation, fallback, and transient deployment failures, but
  they do not magically create a second account quota pool.
- Multiple enabled groups mean multiple identities or backup pools. Use
  `weight` to let a stronger or less-limited group carry more requests.
- If one group fails with `401`/auth errors, check only that group's key and
  deployments first. If every group fails, check the selected mode, Google edge
  reachability, CA trust, or a global network problem.
- Add capacity before raising aggressive speed knobs. Higher concurrency on one
  weak group usually turns a quota problem into a faster quota problem.

### `vercel`

Used only when `mode = "vercel_edge"`.

- `base_url`: Vercel or Netlify app origin, for example
  `https://your-project.vercel.app` or `https://your-site.netlify.app`.
- `relay_path`: usually `/api/api`.
- `auth_key`: must match the platform environment variable `AUTH_KEY`.
- `verify_tls`: keep `true`; disabling accepts TLS interception on the
  serverless relay hop.
- `max_body_bytes`: max request body size sent through JSON/base64.
  The UI default is 4 MiB (`4194304` bytes). Raise it only for real uploads or
  large POST bodies that hit this guardrail.
- `enable_batching`: reserved for relay batching; keep `false` unless a future
  release documents it.

Vercel Deployment Protection must be disabled for the relay domain. For
Netlify, `/api/api` must route to the Edge Function and return JSON.

## Runtime and Performance

Tuning order matters more than any single number:

1. Make sure `google_ip`, `front_domain`, and SNI tests are healthy.
2. Make sure the chosen backend returns the expected response shape
   (`/exec` for Apps Script, JSON from `/api/api` for serverless JSON, or
   tunnel-node logs for full mode).
3. Add deployment/account capacity if the bottleneck is quota or `504` storms.
4. Then adjust runtime profile, fan-out, ranges, and rate limits one at a time.

### `runtime_auto_tune` + `runtime_profile`

When enabled, the engine chooses defaults for several hot-path knobs.

- `eco`: quota-friendly, fewer concurrent relay calls, slower large downloads.
- `balanced`: default middle ground.
- `max_speed`: more aggressive concurrency; use only with enough relay capacity.

### `parallel_relay`

Apps Script fan-out per logical request.

- `0` or `1`: normal, quota-friendly.
- `2`: mild latency improvement on flaky deployments.
- `3+`: only when you have spare quota and understand the call multiplier.

In `vercel_edge`, range/fan-out behavior is intentionally conservative; one
client request maps to one serverless JSON relay request.

### `relay_rate_limit_qps` + `relay_rate_limit_burst`

Client-side token bucket for smoothing relay bursts.

- Lower QPS can reduce quota stampedes and `504` waves.
- Too low makes page loading feel queued.
- Burst allows short page-load spikes without permanently raising QPS.

### `range_parallelism` + `range_chunk_bytes`

Large GET downloads may be fetched in byte ranges and stitched locally.

- Higher parallelism improves throughput but increases in-flight relay calls.
- Larger chunks reduce relay call count but each call takes longer.
- For quota pressure, prefer larger chunks and lower parallelism.

### `relay_request_timeout_secs`

Timeout for one ordinary Apps Script relay call in `apps_script` mode.

This is the budget for one HTTP request/response through your deployed
`Code.gs` relay. It is not the Full-mode tunnel batch timeout; that is
`request_timeout_secs`.

- **Raise it** when the relay usually works but your logs show late successes
  or occasional false `504`/timeout failures on slow mobile or filtered
  networks.
- **Lower it** only when you have multiple healthy deployment IDs and want a
  dead relay call to fail over sooner.
- **Do not raise it to hide quota problems.** If Apps Script quota is exhausted,
  a larger timeout only makes the browser wait longer. Add deployments/accounts
  or reduce fan-out instead.

### `request_timeout_secs`

Timeout for one Full-mode batch round trip through `CodeFull.gs` and
`tunnel-node`.

- Default: `30`.
- Valid range is clamped to `5..300`.

How to reason about it:

- Full mode groups many socket operations into **batches**. One timed-out batch
  can affect several active app connections, so this knob changes perceived
  reliability more dramatically than the ordinary relay timeout.
- A **higher timeout** gives a slow Apps Script or tunnel-node more time to
  answer. This avoids false failures on congested or filtered networks, but
  when a deployment is truly stuck the browser waits longer before retrying.
- A **lower timeout** makes failures surface sooner, which is useful only when
  you have other healthy deployments to retry through. With one deployment it
  usually just turns slow-but-working traffic into repeated errors.
- If logs show batches succeed after 25-35 seconds, raise to `45` or `60`.
  If logs show dead deployments and you have several alternatives, lower
  gradually and watch for false timeouts.

Symptom map:

- **"Sometimes slow, then works"**: raise timeout a little.
- **"Always waits, then fails"**: do not keep raising; test tunnel-node URL,
  `TUNNEL_AUTH_KEY`, and backend reachability.
- **"One ID is bad, others are good"**: keep timeout moderate and tune
  `auto_blacklist_*` so the bad ID leaves rotation.

### `auto_blacklist_*`

Full-mode batch timeouts use strikes before a deployment is cooled down,
because one timeout can be a cold start or a transient network stall.

- `auto_blacklist_strikes`: default `3`.
- `auto_blacklist_window_secs`: default `30`.
- `auto_blacklist_cooldown_secs`: default `120`.

How to reason about it:

- A **strike** means one Full-mode batch timed out for one Apps Script
  deployment. The deployment is not blacklisted immediately because a single
  timeout can be normal cold start, mobile-network loss, or temporary ISP
  shaping.
- `auto_blacklist_strikes` controls how much evidence is needed. Higher values
  are forgiving and reduce false lockouts. Lower values fail over faster but
  may punish a deployment that would have recovered on the next request.
- `auto_blacklist_window_secs` controls how close together those failures must
  be. A short window catches "this deployment is failing right now"; a long
  window treats scattered failures as related.
- `auto_blacklist_cooldown_secs` controls how long the deployment stays out of
  rotation after the threshold trips. Longer cooldown protects users with many
  alternatives from repeatedly touching a bad deployment. Shorter cooldown is
  safer when that deployment is your only path.

The knobs work together:

- `strikes` answers: **how much proof do we need?**
- `window` answers: **must that proof happen close together?**
- `cooldown` answers: **after we believe it is bad, how long do we avoid it?**

Think in terms of cost:

- In a **single-deployment setup**, a false blacklist is expensive because
  there is nowhere else to send traffic. Be patient before blacklisting, and
  retry soon.
- In a **multi-deployment setup**, keeping a bad ID in rotation is expensive
  because every retry wastes latency and quota while good IDs are available.
  Be quicker to blacklist, and keep it out longer.

Decision guide:

- **One deployment / one Google account**: avoid locking yourself out. Try
  `auto_blacklist_strikes: 5`, `auto_blacklist_window_secs: 60`,
  `auto_blacklist_cooldown_secs: 30`. This says "wait for repeated evidence,
  and if it trips, retry soon because there is no alternative."
- **Many deployments / healthy rotation**: fail away from bad IDs quickly. Try
  `auto_blacklist_strikes: 2`, `auto_blacklist_window_secs: 30`,
  `auto_blacklist_cooldown_secs: 300`. This says "two close failures are
  enough; keep the bad ID away while other IDs carry traffic."
- **Unclear case**: change one knob at a time. If false lockouts appear, raise
  strikes or shorten cooldown. If users feel long stalls before recovery, lower
  strikes or extend cooldown after a trip.

Reading logs:

- `timeout strike 1/3`: the ID is still in rotation; this is only evidence.
- `blacklisted script ... timeouts`: the threshold tripped; that ID is cooling
  down and traffic should move to other IDs if any exist.
- Repeated blacklisting of every ID usually means a shared dependency is wrong
  (`TUNNEL_AUTH_KEY`, tunnel-node down, Apps Script deployment stale), not that
  every ID independently became bad.

### `coalesce_step_ms` + `coalesce_max_ms`

Full-mode batch coalescing window for the client tunnel multiplexer.

- `coalesce_step_ms` is the soft wait added after a new tunnel op arrives.
- `coalesce_max_ms` is the hard cap for one batch.
- Defaults are `40` and `1000`. Lower values reduce input latency; higher
  values can reduce Apps Script batch count when many sessions wake together.

## Network Reachability

### `google_ip`

IPv4 address of a Google frontend used for SNI-fronted connections. If it is
poisoned or stale, connections time out quickly.

Useful commands:

```bash
./mhrv-f scan-ips
./mhrv-f test-sni
```

### `front_domain`

SNI hostname for the Google edge hop. The safest default is `www.google.com`.
Do not replace it with an IP address.

### `upstream_socks5`

Optional SOCKS5 chain for raw TCP fallback traffic, usually a local xray,
v2ray, or sing-box inbound such as `127.0.0.1:50529`.

Important boundaries:

- It requires an already working SOCKS5 proxy. `mhrv-f` does not create that
  upstream tunnel for you.
- It is mainly useful in `apps_script` proxy mode for non-HTTP protocols that
  the Apps Script fetch relay cannot carry.
- It does not apply to normal HTTP/HTTPS requests that are handled by local MITM
  and the JSON relay path.
- It is not a substitute for `full` mode or tunnel-node.

Use it when an application can only speak SOCKS5/raw TCP and you already have a
separate tunnel client available.

### `sni_hosts`

Optional SNI rotation pool. Use it when the SNI tester shows some Google names
work better than others from your network.

### `fetch_ips_from_api`, `max_ips_to_scan`, `scan_batch_size`, `google_ip_validation`

Control IP discovery and validation. Larger scans find more candidates but take
longer. Validation helps avoid using non-Google or malformed endpoints.

## Routing Behavior

### `passthrough_hosts`

Bypass the JSON relay for matching hosts. Matching supports exact names,
leading-dot suffixes, and `*.` aliases:

- `example.com`
- `.example.com`
- `*.example.com`

Use this for hosts that work directly, should avoid MITM, or should route via
`upstream_socks5`.

### `domain_overrides`

Per-host rules:

- `force_route = "direct"`
- `force_route = "sni_rewrite"`
- `force_route = "relay"`
- `force_route = "full_tunnel"`
- `never_chunk = true`

Use this to fix one site without changing global behavior.

### `hosts`

Maps specific hostnames to routing categories. Useful for Google-owned hosts
that can use direct SNI rewrite instead of Apps Script relay.

### `tunnel_doh` + `bypass_doh_hosts`

By default, known browser DNS-over-HTTPS endpoints stay inside the tunnel
(`tunnel_doh = true`). This is slower on networks where direct DoH works, but it
is safer on filtered networks where direct connections to `dns.google`,
`chrome.cloudflare-dns.com`, or similar pinned DoH hosts are blocked.

- Set `tunnel_doh = false` only if direct DoH works on your network and you want
  the latency win from bypassing Apps Script/tunnel-node for DNS.
- Add exact hosts or leading-dot suffixes to `bypass_doh_hosts` for extra DoH
  endpoints in your browser or region.

### `block_quic`

When enabled, the SOCKS5 UDP relay drops UDP/443 datagrams before they enter
the full tunnel. Browsers and many apps then fall back to TCP/HTTPS, avoiding
QUIC-over-tunnel stalls and wasted Apps Script batches.

Leave it off if you intentionally want full-mode SOCKS5 UDP/443 to pass
through.

### `normalize_x_graphql`

Trims noisy X/Twitter GraphQL query params to increase cache hit rate. Disable
only if X endpoints visibly break.

### `youtube_via_relay`

Routes YouTube HTML/API surfaces through the relay instead of direct
Google-host handling.

Trade-off:

- May help with SNI-policy/restricted-mode problems.
- Costs relay quota and may be slower for video.
- `ytimg.com` thumbnails/assets stay on SNI rewrite so they do not burn relay
  quota.
- `googlevideo.com` video chunks are not SNI-rewritten by default. They use
  separate Google video edges, so pointing them at the normal `google_ip` can
  cause TLS/wrong-edge failures.
- YouTube SABR streams (`sabr=1` in `googlevideo.com` URLs) can hit the
  Apps Script buffering/timeout ceiling around one minute. Use Full mode for
  reliable YouTube relay playback; otherwise keep video chunks off the relay
  when possible.

## Safety and LAN Exposure

### `listen_host`

Default `127.0.0.1` binds only to the local machine. LAN binding is convenient
but risky unless paired with guardrails.

### `lan_token`

Requires a token header for HTTP/CONNECT clients when exposing the proxy.

### `lan_allowlist`

Restricts which client IPs may connect. Especially important for SOCKS5 because
SOCKS5 has no native HTTP header token.

## TLS Verification

### `verify_ssl`

Controls certificate verification for the Apps Script/Google-edge hop. Keep it
enabled unless debugging a known interception issue.

### `vercel.verify_tls`

Controls certificate verification for the serverless JSON relay hop. Keep it
enabled. Disabling it makes the relay transport easier to intercept.

## Self-Heal

### `outage_reset_*`

When repeated eligible relay failures happen in a short window, the engine can
reset keep-alive pools and reconnect.

- Too strict: may not recover quickly.
- Too aggressive: may churn connections unnecessarily.

Defaults are suitable for most users.
