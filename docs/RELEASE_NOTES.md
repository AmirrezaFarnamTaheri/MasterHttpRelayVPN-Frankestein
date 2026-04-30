# Release Notes

- `direct` is the public no-relay SNI-rewrite mode name across UI, Android,
  docs, and examples. Legacy `google_only` configs still load.
- `fronting_groups` adds SNI rewrite for Vercel, Fastly, and
  Netlify/CloudFront-style edges. Start from
  `config.fronting-groups.example.json`.
- Netlify XHTTP now has first-class docs and an in-app/static VLESS generator.
  Tested Address/SNI candidates include `kubernetes.io`, `helm.sh`,
  `letsencrypt.org`, and the documented Helm/Kubernetes/SIG subdomains.
- Vercel XHTTP now shares the same generator workflow, with candidates such as
  `react.dev`, `nextjs.org`, `cursor.com`, and Vercel-owned subdomains.
- Full mode `CodeFull.gs` includes Apps Script edge DNS caching to reduce
  tunnel-node DNS round-trips.
- The safer `tunnel_doh` default keeps browser DoH inside the tunnel.
- Hotspot/LAN sharing docs cover iOS, macOS, and Windows setup.
