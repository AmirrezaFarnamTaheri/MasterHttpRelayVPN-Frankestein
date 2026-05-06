# Batch 46 - Fronting Groups Example Drift Gate

Date: 2026-05-05

## Summary

- Added `tools/check-fronting-groups-example.py`.
- Wired the new example guard into `tools/run-repo-sanity.py`.
- Added the new guard to `tools/check-ci-local-sanity-parity.py`.
- Documented the guard in `tools/README.md`.

## Contract Now Guarded

- `config.fronting-groups.example.json` must remain a `direct` mode starter.
- The starter must remain loopback-only by default.
- Vercel, Fastly, and Netlify/CloudFront groups must stay present.
- The Vercel group must keep key starter domains such as `vercel.com`,
  `vercel.app`, `nextjs.org`, `cursor.com`, and `ai-sdk.dev`.
- The Fastly group must keep the curated Reddit, Pinterest, CNN, BuzzFeed,
  GitHub asset, PyPI, and Fastly domain families.
- The Fastly group must keep the `151.101.x.x` anycast example and
  `www.python.org` SNI.
- The Netlify/CloudFront group must keep `netlify.com` and `netlify.app`.
- `docs/fronting-groups.md`, `docs/relay-modes.md`, and the parity matrix must
  keep pointing users to the example.

## Parity Notes

- This is an example/docs/product-surface guard, not a runtime default change.
- It preserves donor/upstream Fastly example value without turning those routes
  into guaranteed behavior.

## Concurrency / Split-Brain Notes

- No runtime code changed.
- The gate prevents the example JSON, fronting-group docs, relay-mode docs, and
  parity matrix from drifting apart.

## Cleanup

- No generated build output was intentionally created.
- Removed Python `__pycache__` directories created by local verification.
- No Gradle command was run.

## Verification

- `python tools/check-fronting-groups-example.py`
- `python tools/check-ci-local-sanity-parity.py`
- `python tools/check-doc-links.py`
- `python tools/run-repo-sanity.py --skip-node`
- `python tools/check-repo-cleanliness.py`
- `python tools/run-repo-sanity.py`

All checks passed on 2026-05-05.
