# Changelog - Strategic Batch 3 Trust Center CLI (2026-05-03)

Maintainer-facing record for the larger Trust Center CLI/docs iteration.

## Summary

| Field | Detail |
|-------|--------|
| What changed | Added `mhrv-f trust-center` and `mhrv-f trust-center --json`, and documented the command in README, Doctor docs, Trust Center docs, and docs index. |
| Why | Desktop Help and support bundles now share `trust_center::snapshot()`. The CLI needed the same source of truth so maintainers and headless users can inspect trust/signing/support state without exporting a bundle or opening Desktop. |
| Files changed | `src/main.rs`, `README.md`, `docs/doctor.md`, `docs/trust-center.md`, `docs/index.md`, `elevation_audit_roadmap_source.md`, this changelog. |
| Desktop impact | No new Desktop code in this slice; Desktop already consumes the same snapshot. |
| Android impact | None in code; Android mobile projection remains pending. |
| Backend impact | None. |
| Docs impact | CLI diagnostics flow now points users from Doctor/support-bundle preview to the Trust Center snapshot. |

## Behavior

- `mhrv-f trust-center` loads the active config and prints:
  - mode and platform;
  - whether local CA is required;
  - CA cert/key presence and platform trust probe;
  - CA next action from the shared snapshot;
  - browser trust probe facts;
  - Android user-CA caveat;
  - Android release signing policy;
  - support-bundle manifest file/sensitive counts and redaction policy.
- `mhrv-f trust-center --json` prints the raw `TrustSnapshot` JSON.
- `--json` is rejected for other commands to keep CLI semantics precise.

## Cleanup

- No separate CLI trust model was added.
- No trust-store mutation was added.
- No new dependency was introduced.

## Split-brain / race assessment

- Split-brain reduced: Desktop Help, CLI, and support bundles now all read the
  same Rust Trust Center snapshot.
- Race risk low: the command is synchronous and read-only.
- The command requires a valid config so mode-specific CA requirements are not
  guessed from defaults.

## Verification

- `cargo fmt --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all-targets --features ui --quiet` (179 root tests + 5 UI/config tests)
- `cargo run --quiet --bin mhrv-f -- trust-center` (clean human output)
- `cargo run --quiet --bin mhrv-f -- trust-center --json` (clean JSON output, no log preamble)
- `python tools/run-repo-sanity.py`
- `python tools/check-doc-links.py`
- `python tools/check-repo-cleanliness.py`
- `cargo clippy --all-targets -- -D warnings` in `tunnel-node`
- `cargo test --all-targets --quiet` in `tunnel-node` (34 tests)
- Removed `tools/__pycache__` after Python checks regenerated it.

## Remaining risk

- Android does not yet expose the Trust Center projection.
- The CLI command is local-trust focused; deployed backend live probes remain
  Doctor/Test responsibilities.
