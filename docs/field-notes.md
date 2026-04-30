# Field Notes And Edge Candidates

This page keeps distilled field findings that are useful in practice without
carrying raw chat exports, social usernames, message timestamps, duplicate
links, or platform button text. Treat every candidate here as a starting point:
test it on your own network, keep only what passes, and prefer the smallest
working setup.

## Google SNI Candidates

The default Google SNI pool includes DNS-valid names gathered from field
reports plus the existing Google service set used by the app. These are used
when `front_domain` is a Google-owned hostname and `sni_hosts` is empty.

Useful candidates now included in the built-in pool:

```text
www.google.com
google.com
mail.google.com
gmail.com
www.gmail.com
workspace.google.com
www.gstatic.com
ssl.gstatic.com
gstatic.com
g1.gstatic.com
g3.gstatic.com
fonts.gstatic.com
csi.gstatic.com
connectivitycheck.gstatic.com
checkin.gstatic.com
clientservices.googleapis.com
beacons.gvt2.com
beacons.gcp.gvt2.com
```

Two reported names were deliberately not added because a local DNS sanity
check returned NXDOMAIN:

```text
www.workspace.google.com
www.mail.google.com
```

How to use this safely:

- Keep `front_domain = "www.google.com"` unless you are deliberately testing.
- Use the desktop **SNI pool tester** or `mhrv-f test-sni`.
- A failed ping is not automatically meaningful. Some HTTPS endpoints do not
  answer ICMP, and SNI success is a TLS handshake question.
- Keep a small custom `sni_hosts` list only after the candidates pass on your
  own ISP.

## Vercel XHTTP Candidates

For the external Vercel XHTTP helper, field reports show that the client
`Address` and `SNI` can sometimes use Vercel or framework edge names while
`Host` remains your actual Vercel project domain.

Candidates to test:

```text
community.vercel.com
analytics.vercel.com
botid.vercel.com
blog.vercel.com
app.vercel.com
api.vercel.com
ai.vercel.com
cursor.com
nextjs.org
react.dev
```

Keep normal certificate validation when possible. Setting `allowInsecure` can
help isolate whether a failure is certificate-related, but it should not become
the saved everyday profile unless you knowingly accept that trust downgrade.

Candidate smell test before you spend time testing:

- The name should be hosted by the same public edge/provider family you are
  trying to front. For Vercel, likely candidates are Vercel product subdomains,
  Vercel-hosted framework/community sites, or well-known projects served from
  Vercel's edge.
- DNS should resolve to the same broad edge ranges or CNAME family as a known
  working candidate. Compare `Resolve-DnsName nextjs.org -Type A` with the new
  name.
- TLS should complete with that same hostname as SNI. Test the candidate as
  both Address and SNI, then keep `Host` on your own deployed Vercel project.
- Discard names that are only visually related, marketing-only, private/login
  surfaces, or unrelated CDNs. A ping response alone is not enough; many valid
  TLS candidates block ICMP, and many pingable hosts are not useful XHTTP
  fronts.

## Netlify, Fastly, And CloudFront Notes

These notes apply to external Xray/V2Ray fronting configurations, not to native
desktop modes inside `mhrv-f`.

- Netlify XHTTP helper: use a Netlify site hostname or attached custom domain
  as the client `Address`/`SNI`/`Host`, with `Host` matching the deployed
  Netlify site. The helper forwards streamed XHTTP to your own Xray backend.
- Netlify XHTTP reachable front candidates reported as working for
  `Address`/`SNI` testing:

  ```text
  kubernetes.io
  helm.sh
  letsencrypt.org
  docs.helm.sh
  kubectl.docs.kubernetes.io
  blog.helm.sh
  kind.sigs.k8s.io
  cluster-api.sigs.k8s.io
  krew.sigs.k8s.io
  gateway-api.sigs.k8s.io
  scheduler-plugins.sigs.k8s.io
  kustomize.sigs.k8s.io
  image-builder.sigs.k8s.io
  ```

  Keep `Host` set to your own Netlify site/custom domain unless you have a
  deliberate external-client reason to change it. Use `allowInsecure = true`
  only as a diagnostic or when your external Xray profile intentionally uses
  mismatched Address/SNI/Host values and you accept the certificate-validation
  trade-off.
- How to suspect a new Netlify candidate: look for stable public documentation,
  foundation, or project sites that sit behind AWS CloudFront or the same edge
  family as the working Netlify fronts. Prefer domains with boring public
  content, a clean TLS certificate, and DNS that resembles the known candidates.
  Test with DNS, TLS/SNI, then a real XHTTP profile. Keep only candidates that
  connect on your own ISP.
- Netlify JSON helper: use `tools/netlify-json-relay` with native serverless
  JSON mode when you want a no-VPS fetch relay and do not have an Xray backend.
- Netlify via external MITM/fronting configs: a reported Xray rule routes
  `geosite:netlify` through a CloudFront-like TLS repack path using
  `letsencrypt.org` and AWS/CloudFront certificate names. This is documented
  as an external client idea only; it is not a native `mhrv-f` mode.
- Fastly field reports: some GitHub, Reddit, Fastly, and asset hosts may work
  through Fastly-oriented front names such as `www.python.org` in external
  Xray configs. Keep this separate from the Netlify XHTTP helper.
- For v2rayNG MITM-DomainFronting style configs, enable the Hev TUN feature
  and keep the default local SOCKS port unchanged if the imported routing rules
  reference that port.

## Full Tunnel Notes

Goose-style relay instructions map to `mhrv-f` full mode as follows:

- VPS server: use `tunnel-node`.
- Server secret: set `TUNNEL_AUTH_KEY`.
- Apps Script bridge: deploy `assets/apps_script/CodeFull.gs`.
- Client mode: choose `full`.
- Local apps: point them at the desktop HTTP/SOCKS listeners, or use the
  Android VPN mode for phone-wide routing.

For capacity, add multiple Apps Script account groups instead of treating one
long list of deployment IDs as a single identity. Each group should represent a
separate account/quota pool with its own secret. For Firefox SOCKS5, enable
remote DNS in the browser or use `socks5h` where the client supports it.

## Direct Mode And YouTube Setup

`direct` is a bootstrap helper. It can help reach Google-owned setup
surfaces such as Apps Script, Google Drive, Chrome Web Store, or YouTube page
chrome, but it is not a general media/video path.

YouTube video payloads are quota-heavy. Use `youtube_via_relay` only when you
understand the trade-off: it routes YouTube through Apps Script, changes the
effective user agent to the Apps Script fetcher, and spends daily UrlFetch
quota quickly. Account groups, relay rate limits, and range tuning matter more
than raw SNI lists for this path.

## Discarded Field Items

The following raw items were intentionally not merged into the product:

- Public web proxy lists, because they are third-party browsing surfaces with
  unknown logging, malware, and credential risk.
- Cookie-export workflows for download automation, because they encourage
  storing browser session cookies in external automation secrets.
- Third-party APK links, because they are outside this project's release and
  signing chain.
- Advice to disguise copied deployment code to avoid platform enforcement.
  Instead, keep deployments private, authenticated, and compliant with the
  platform you use.
