# Direct Fronting Groups

`mode = "direct"` runs without Apps Script, Vercel/Netlify JSON, or a VPS.
It only uses the local SNI-rewrite path: the browser connects to mhrv-f, mhrv-f
terminates local TLS with your trusted CA, then opens a new TLS connection to a
friendly edge SNI while keeping the real HTTP `Host` inside the encrypted stream.

Built-in direct mode already knows the Google edge through:

```json
{
  "google_ip": "216.239.38.120",
  "front_domain": "www.google.com"
}
```

`fronting_groups` extends the same idea to other multi-tenant edges such as
Vercel, Fastly, and Netlify's CloudFront-backed sites.

## Example

Start from `config.fronting-groups.example.json`:

```json
{
  "mode": "direct",
  "fronting_groups": [
    {
      "name": "netlify-cloudfront",
      "ip": "35.157.26.135",
      "sni": "letsencrypt.org",
      "domains": ["netlify.com", "netlify.app"]
    }
  ]
}
```

For Netlify-XHTTP style clients, tested SNI / Address candidates include:

- `kubernetes.io`
- `helm.sh`
- `letsencrypt.org`
- `docs.helm.sh`
- `kubectl.docs.kubernetes.io`
- `blog.helm.sh`
- `kind.sigs.k8s.io`
- `cluster-api.sigs.k8s.io`
- `krew.sigs.k8s.io`
- `gateway-api.sigs.k8s.io`
- `scheduler-plugins.sigs.k8s.io`
- `kustomize.sigs.k8s.io`
- `image-builder.sigs.k8s.io`

Use the same host for Address and SNI in the VLESS/Xray profile and set
`allowInsecure = true` only for that external client workflow. In native mhrv-f
direct mode, keep `verify_ssl = true`; mhrv-f validates the upstream certificate
against the configured `sni`.

For Vercel-XHTTP style clients, tested Address/SNI candidates include
`react.dev`, `nextjs.org`, `cursor.com`, and Vercel-owned subdomains such as
`app.vercel.com`, `api.vercel.com`, and `community.vercel.com`. These are
external-client profiles, not `fronting_groups` entries by themselves.

## What Each Field Means

- `name`: label used in logs. Keep it unique so troubleshooting is readable.
- `ip`: one current edge IP for the chosen CDN. If it stops working, resolve the
  SNI again and update it.
- `sni`: the harmless-looking TLS SNI sent to the edge, for example `react.dev`
  for Vercel, `www.python.org` for Fastly, or `letsencrypt.org` for Netlify's
  CloudFront path.
- `domains`: target hostnames that should route through this group. Entries
  match exact hosts and subdomains, so `netlify.app` covers `site.netlify.app`.

## Routing Order

For a single HTTPS CONNECT, mhrv-f chooses in this order:

1. `passthrough_hosts` and explicit direct `domain_overrides`.
2. DoH handling (`tunnel_doh` and `bypass_doh_hosts`).
3. `mode = full`, which bypasses local MITM and sends everything to tunnel-node.
4. Matching `fronting_groups`.
5. Built-in Google SNI rewrite.
6. `mode = direct` fallback: raw/direct TCP.
7. `mode = apps_script` fallback: Apps Script relay.

This means fronting groups win over the built-in Google path for their listed
domains, but they do not override explicit user passthrough choices.

## Safety Checklist

Only list domains that really live on the same edge as the chosen SNI. If you put
an unrelated domain into a group, the CDN receives an encrypted request whose
inner `Host` names a tenant it does not serve. That can create two problems:

- Privacy leak: the CDN can log the unexpected hostname, your IP, and timing.
- Broken UX: the CDN returns a default 404 or wrong-tenant response.

Quick validation:

```powershell
Resolve-DnsName letsencrypt.org -Type A
Resolve-DnsName netlify.app -Type A
```

The IPs do not have to be identical every time, but they should clearly belong to
the same CDN family. When in doubt, keep the group out of your active config.

How to look for new candidates:

- Start from a known-good edge and collect names that are visibly served by that
  same provider family: provider product subdomains, official docs, public
  framework sites, or foundation sites hosted on the same edge.
- Check DNS first. A candidate that resolves through the same CNAME chain or
  address family as a known-good name is worth testing; an unrelated CDN is not.
- Check TLS with the candidate as SNI. If the handshake fails or the certificate
  is unrelated, discard it.
- Test the real client profile last. ICMP ping is only a weak signal; many
  useful fronts do not answer ping, and many pingable hosts cannot carry XHTTP.

## UI Status

The desktop UI preserves `fronting_groups` on Save, but it does not edit the
group list yet. Edit `config.json` directly, then reopen the app. This is
intentional: a wrong group can leak Host headers, so the first UI editor should
include validation and warnings instead of a tiny free-form table.
