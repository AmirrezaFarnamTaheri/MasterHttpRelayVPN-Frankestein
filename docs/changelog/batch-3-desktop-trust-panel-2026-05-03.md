# Changelog - Strategic Batch 3 desktop Trust Center panel (2026-05-03)

Maintainer-facing record for the first visible Desktop Trust Center panel.

## Summary

| Field | Detail |
|-------|--------|
| What changed | Added a read-only Trust Center status panel to Desktop **Help & docs**. It renders the shared Rust trust snapshot and support-bundle manifest instead of duplicating ad hoc UI checks. |
| Why | Earlier Batch 3 work created `trust_center::snapshot()` and `support_bundle::preview_manifest()`, but Desktop users could only read a docs link. This makes certificate/signing/support-bundle state visible in-app without starting mutable repair flows. |
| Files changed | `src/bin/ui.rs`, `docs/trust-center.md`, `docs/changelog/batch-3-support-bundle-preview-2026-05-03.md`, `elevation_audit_roadmap_source.md`, this changelog. |
| Desktop impact | Help & docs now shows mode, CA status, CA cert/key presence, platform trust probe result, Firefox profile/marker counts, `certutil` availability, Android signing policy, support-bundle file count, and sensitive-file count. |
| Android impact | None in code. Android still needs a mobile Trust summary/share path, tracked separately. |
| Backend impact | None. The panel is local/read-only. |
| Docs impact | Trust Center docs now say Desktop has a read-only Help panel. |

## Behavior

- The panel validates the current form and then snapshots trust state for that
  config.
- Invalid/incomplete forms show a clear **Trust snapshot unavailable** callout
  instead of panicking or guessing.
- CA repair text is taken from `trust_center::snapshot().ca.next_action`.
- Support-bundle file counts come from `support_bundle::preview_manifest()`.

## Cleanup

- No new UI-only trust model was added.
- No CA install/remove action was added to the panel; existing serialized
  buttons remain the only mutating flows.
- Removed stale duplicated "Desktop/Android UI preview surfaces are not
  implemented yet" wording from the previous support-bundle changelog.

## Split-brain / race assessment

- Split-brain reduced: Desktop now consumes the shared Rust snapshot and bundle
  manifest.
- Race risk low: panel reads synchronously from the current form and does not
  spawn background jobs or mutate trust stores.
- Remaining stale-result work belongs to future live/probing panels that perform
  asynchronous checks.

## Verification

- `cargo fmt --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all-targets --features ui --quiet` (179 root tests + 5 UI/config tests)
- `python tools/run-repo-sanity.py`
- `python tools/check-doc-links.py`
- `python tools/check-repo-cleanliness.py`
- `cargo clippy --all-targets -- -D warnings` in `tunnel-node`
- `cargo test --all-targets --quiet` in `tunnel-node` (34 tests)

## Remaining risk

- The panel is in Help & docs, not a dedicated top-level Trust Center tab.
- Android does not yet expose the same snapshot.
- Deeper NSS certificate-presence checks remain future work.
