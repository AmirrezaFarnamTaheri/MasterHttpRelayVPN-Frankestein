# Changelog - Strategic Batch 3 support-bundle preview (2026-05-03)

Maintainer-facing record for the Trust Center support-bundle preview increment.

## Summary

| Field | Detail |
|-------|--------|
| What changed | Added a machine-readable support-bundle manifest, wrote it as `manifest.json` during export, added `mhrv-f support-bundle --preview`, added bounded/redacted `recent-logs.txt`, and documented bundle contents/redaction in README, Doctor docs, and Trust Center docs. |
| Why | Trust Center required a preview before support export. The preview now comes from the same manifest used by the real bundle, reducing docs/CLI/export drift. |
| Files changed | `src/support_bundle.rs`, `src/main.rs`, `README.md`, `docs/doctor.md`, `docs/trust-center.md`, `elevation_audit_roadmap_source.md`, this changelog. |
| Desktop impact | No visible UI yet; Desktop can reuse `support_bundle::preview_manifest()` for a future preview panel. |
| Android impact | No Kotlin/JNI change yet; Android still needs a mobile support summary/share path. |
| Backend impact | None. |
| Docs impact | Support-bundle docs now list `manifest.json` and `trust.json`; preview command is documented. |
| Config/schema impact | None. |

## Behavior

- `mhrv-f support-bundle --preview` prints JSON and exits without writing a
  bundle.
- Real support bundles now include:
  - `manifest.json`
  - `meta.json`
  - `config.redacted.json`
  - `doctor.json`
  - `status.json`
  - `trust.json`
  - `recent-logs.txt`
- The manifest records the redaction policy:
  - auth keys: redacted;
  - LAN tokens: removed;
  - deployment IDs: masked prefix/suffix;
  - private keys: not included.
- `recent-logs.txt` reads the newest direct app-data or `logs/*.log` file when
  present, caps the excerpt at 64 KiB, and applies config-aware redaction. If no
  persistent log file exists, it records that live Desktop/Android panels may
  still hold transient logs.

## Cleanup

- No legacy export path was kept: the manifest is now part of the current bundle
  shape.
- No Gradle command was run.

## Split-brain / race assessment

- Split-brain reduced: CLI preview and actual bundle contents use the same
  `preview_manifest()` source.
- Race risk low: preview is pure data and does not read runtime state or write
  files.

## Verification

- `cargo fmt --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all-targets --features ui --quiet` (172 root tests + 5 UI/config tests)
- `python tools/run-repo-sanity.py`
- `cargo run --quiet --bin mhrv-f -- support-bundle --preview`
- `python tools/check-doc-links.py`
- `python tools/check-repo-cleanliness.py`

## Remaining risk

- Desktop Help now shows Trust/support-bundle manifest counts, but a dedicated
  export preview UI is still pending.
- Android UI preview surfaces are not implemented yet.
