# Platform Alternatives And Migration Notes

Last checked: 2026-04-30. Free plans and quotas change often; verify the linked
official pricing/docs before promising a workflow to users.

This page answers three questions:

1. Can this platform replace Vercel/Netlify for one of our helpers?
2. How do we migrate and test it?
3. What would we append to the app/docs if it proves useful?

## Fit Matrix

| Platform | Best fit for this project | Free/free-ish status | Recommendation |
|---|---|---|---|
| EdgeOne Pages | Static docs/generator, serverless functions, possible XHTTP/JSON experiments | Official page says Pages is free and supports serverless functions | Worth a prototype after checking function streaming and request body limits |
| Render | JSON relay or tunnel-node-style web service; static docs | Free web services and static sites exist, but free web services spin down after idle | Good for experiments; cold starts hurt XHTTP/full tunnel reliability |
| Railway | tunnel-node, JSON relay, XHTTP Node relay | Pricing lists a 30-day free trial with credits, then paid minimum | Good developer experience, not a long-term free default |
| Fly.io | tunnel-node near users; possible JSON relay | Current pricing has trial credit; legacy free allowances are only for older orgs | Strong for paid/controlled VPS-like deployments |
| Northflank | tunnel-node/container relay | Sandbox lists two free services and one free database | Worth testing for always-on small services |
| DigitalOcean App Platform | static docs/generator for free; paid web services for relays | Free tier is static-site-only; containers/web services are paid | Good docs/generator host, not free for relays |
| Heroku | tunnel-node/Node relay on paid dynos | Eco dynos are paid and sleep; no general free dyno tier | Usable paid fallback, not a free recommendation |
| Dokploy | self-host manager for tunnel-node and Node relays | Free open-source self-hosted control panel; needs your own server | Good VPS automation path, not a hosting provider |
| Coolify | self-host manager for tunnel-node and Node relays | Self-hosted plan is free forever; needs your own server | Good VPS automation path, similar role to Dokploy |
| Zeabur | self-host/server management and possible service deployment | Free plan manages one own server; hosted runtime details need plan review | Useful if the user already has a server |
| uniCloud | China-oriented serverless/static hosting | DCloud docs show free service spaces for Alibaba/Alipay cloud, not Tencent | Interesting China-route experiment; requires China ecosystem testing |
| 21YunBox / 21CloudBox | China deployment/acceleration, Netlify-style China mirror | Commercial/compliance oriented; not a generic free relay host | Useful for China-accessible static/docs or site acceleration, not native relay default |
| Maoziyun | Static frontend platform | Official FAQ says currently free, later business/heavy-traffic charging | Static/generator only until function/proxy support is proven |
| PinMe | Static frontend/IPFS hosting | CLI deploys static files; not a normal serverless runtime | Good for static docs/generator only; not for JSON/XHTTP relays |

## Migration Patterns

### Pattern A: Static Generator / Docs Host

Use this when the platform only serves static files.

Good candidates: DigitalOcean free static sites, PinMe, Maoziyun, EdgeOne Pages
static, Render static, 21YunBox static, uniCloud frontend hosting.

Steps:

1. Publish `tools/netlify-xhttp-relay/public/` or a dedicated docs bundle.
2. Confirm `index.html` and `vless-generator.html` load over HTTPS.
3. Do not expect native relay behavior. Static hosts cannot replace
   `tools/vercel-json-relay`, `tools/netlify-json-relay`, XHTTP Edge Functions,
   or `tunnel-node`.
4. Append to the app only as a docs/tool link, not as a backend mode.

Test:

```sh
curl -I https://your-static-host/vless-generator.html
```

### Pattern B: Serverless JSON Relay

Use this when the platform can run a Fetch/HTTP function and make outbound HTTP
requests.

Good candidates to prototype: EdgeOne Functions/Pages Functions, Render web
service, Railway/Heroku/Fly/Northflank container service, uniCloud HTTP
function if body/response limits fit.

Migration from Vercel/Netlify:

1. Port the same JSON contract used by `tools/vercel-json-relay`:
   `POST /api/api` with `{ "k": AUTH_KEY, "u": url, "m": method, ... }`.
2. Store `AUTH_KEY` as a platform environment variable.
3. Set app mode to `vercel_edge` because the native client protocol is generic.
4. Set `vercel.base_url` to the new platform origin and keep
   `vercel.relay_path = "/api/api"`.
5. Run Doctor and Test relay.

Append to the app:

1. Add `tools/<platform>-json-relay/`.
2. Add docs under `docs/<platform>-json-relay.md`.
3. Add it to **Backend tools** in `src/bin/ui.rs` only after a working
   endpoint returns JSON, not HTML/login/protection pages.
4. Add a syntax/build check to CI.

Test:

```sh
curl -i https://your-platform.example/api/api
mhrv-f test
```

### Pattern C: XHTTP Helper

Use this when the platform supports request and response streaming well enough
to proxy Xray XHTTP to your own backend.

Good candidates to test: EdgeOne Functions, Render/Railway/Fly/Northflank
containers, Heroku paid dyno. Static-only platforms are not candidates.

Migration from Vercel/Netlify:

1. Port the streaming relay from `tools/vercel-xhttp-relay` or
   `tools/netlify-xhttp-relay`.
2. Keep `TARGET_DOMAIN=https://your-xray-backend:port`.
3. Pick one public path, for example `/p4r34m`, and make sure only that path is
   routed to the relay when static helper pages must stay visible.
4. Generate client links with the desktop **Backend tools -> XHTTP VLESS
   generator**. Use the nearest preset first, then edit Address/SNI candidates.
5. Test with a real Xray client. A plain browser request is not enough to prove
   XHTTP streaming.

Append to the app:

1. Add a tool folder only after stream tests pass.
2. Add candidate presets only after multiple networks confirm them.
3. Keep it documented as an external Xray/V2Ray helper, not a native `mhrv-f`
   mode.

### Pattern D: tunnel-node / Full Mode Server

Use this when the platform runs a long-lived container or VM.

Good candidates: Fly.io, Railway, Render, Northflank, Heroku paid, Dokploy on
your VPS, Coolify on your VPS, Zeabur managing your own server.

Migration:

1. Build `tunnel-node` for Linux.
2. Set `TUNNEL_AUTH_KEY` and expose the service port.
3. Put the public tunnel-node URL in `assets/apps_script/CodeFull.gs`.
4. Use client mode `full`.
5. Verify with tunnel-node logs and an IP-check page.

## Address/SNI Candidate Discovery

Do not guess from vibes. A good candidate has a reason to be on the same edge as
the helper:

- Provider-owned names: `app.vercel.com`, `api.vercel.com`,
  `community.vercel.com`.
- Framework/project names hosted on that provider: `nextjs.org`, `react.dev`,
  `cursor.com` for Vercel-style reports.
- Public documentation/foundation names that resolve through the same CDN family
  as known working Netlify/CloudFront reports: `kubernetes.io`, `helm.sh`,
  `letsencrypt.org`, and related Helm/Kubernetes/SIG documentation subdomains.
- Boring, stable, public sites are better than login-only dashboards or
  short-lived campaign domains.

Testing order:

1. DNS: compare `Resolve-DnsName candidate -Type A` with a known-good candidate.
2. TLS: confirm the candidate completes a handshake when used as SNI.
3. Client profile: set Address and SNI to the candidate, keep Host on your own
   deployed relay, then test the real XHTTP client.
4. Keep only candidates that pass on the target network. Ping is optional and
   weak; it can fail for good hosts and succeed for bad ones.

## Source Notes

- [Render free docs](https://render.com/docs/free) document free web services,
  static sites, and idle spin-down behavior.
- [Railway pricing](https://railway.com/pricing) lists a free trial with
  credits, then a paid minimum.
- [Fly.io pricing](https://fly.io/docs/about/pricing/) describes no new free
  plans and legacy-only free allowances.
- [Northflank pricing](https://northflank.com/pricing) lists a Sandbox with
  free services.
- [Heroku pricing](https://www.heroku.com/pricing) lists paid Eco dynos that
  sleep.
- [Dokploy docs](https://docs.dokploy.com/) and
  [Coolify pricing](https://coolify.io/pricing) describe self-hosted deployment
  managers, not free hosting providers.
- [DigitalOcean App Platform pricing](https://www.digitalocean.com/pricing/app-platform)
  shows a free static-site tier; containers/web services are paid.
- [EdgeOne Pages pricing](https://pages.edgeone.ai/pricing) and
  [EdgeOne Pages Functions docs](https://pages.edgeone.ai/document/pages-functions-overview)
  advertise free Pages plus edge/cloud functions.
- [DCloud uniCloud pricing](https://doc.dcloud.net.cn/uniCloud/price.html)
  lists free service-space/frontend-hosting allowances for some providers and no
  free Tencent Cloud service space.
- [PinMe](https://github.com/glitternetwork/pinme) is a static frontend/IPFS
  deployment CLI.
- [21YunBox/21CloudBox](https://www.21cloudbox.com/) is China
  deployment/acceleration focused.
- [Maoziyun](https://www.maoziyun.com/) presents itself as a currently-free
  static frontend platform, with future business/heavy-traffic charging.
