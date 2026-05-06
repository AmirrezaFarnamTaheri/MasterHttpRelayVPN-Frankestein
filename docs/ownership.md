# Ownership Notes

This repo is still using lightweight governance rather than formal CODEOWNERS.
The goal is clear review expectations without pretending there is a large
maintainer team.

## Areas

| Area | Paths | Review focus |
|---|---|---|
| Rust core | `src/`, `Cargo.toml` | config loading, proxy behavior, Doctor/status contracts, redaction |
| Desktop UI | `src/bin/ui.rs`, UI docs | parity, state ownership, diagnostics, layout clarity |
| Android | `android/` | config preservation, JNI contracts, lifecycle, strings EN/FA |
| Backend helpers | `assets/apps_script/`, Cloudflare/serverless docs | helper compatibility markers, auth, redeploy instructions |
| tunnel-node | `tunnel-node/` | concurrency, drain correctness, auth, full-mode compatibility |
| Docs | `README.md`, `docs/`, `tools/README.md` | source-of-truth links, Persian parity, stale names |
| Release | `.github/workflows/`, `.github/scripts/`, `CHANGELOG.md` | CI authority, checksums, release notes, Telegram as mirror |
| Security/trust | `SECURITY.md`, trust docs, signing docs, redaction code | secrets, CA behavior, support sharing, signing continuity |

## Review Rule

When a change crosses areas, review it as one change. Do not land a config or
backend change with only one platform updated unless the difference is
documented as intentionally platform-specific.

## Future CODEOWNERS

Add `.github/CODEOWNERS` only when maintainers are available for the listed
areas. Until then, use this document plus the PR template checklist.
