# Changelog - Strategic Batch 3 trust snapshot (2026-05-03)

Maintainer-facing record for the Trust Center implementation increment after
the Batch 0-4 audit.

## Summary

| Field | Detail |
|-------|--------|
| What changed | Added a shared Rust Trust Center snapshot and exported it in support bundles as `trust.json`; added read-only Firefox/NSS browser trust probe details; removed a duplicate `#[cfg(feature = "ui")]` gate in `src/lib.rs`; updated Trust Center docs to describe the current snapshot and remaining gaps. |
| Why | The Trust Center had a documentation hub, but no shared data surface. Support bundles, future CLI/desktop UI cards, and Android projections need one non-mutating trust view instead of duplicated ad hoc checks. |
| Files changed | `src/trust_center.rs`, `src/lib.rs`, `src/support_bundle.rs`, `docs/trust-center.md`, `elevation_audit_roadmap_source.md`, this changelog. |
| Desktop impact | No visible UI change yet; Desktop can now consume `trust_center::snapshot()` in a later Trust Center panel. |
| Android impact | No JNI/Kotlin change yet; snapshot is Rust-side and can later back an Android-safe projection. |
| Backend impact | None. The snapshot only reads local trust/signing state. |
| Docs impact | Trust Center docs now state that `trust.json` exists and that live Firefox/NSS probe plus bundle preview UI remain pending. |
| Config/schema impact | None. |

## Behavior

- `trust_center::snapshot(&Config)` returns a serializable, non-mutating view of:
  - platform and architecture;
  - current mode;
  - whether the local MITM CA is required for that mode;
  - CA cert/key file presence;
  - best-effort platform trust probe when `ca/ca.crt` exists;
  - Firefox/NSS live-probe availability status;
  - read-only Firefox profile counts, enterprise-roots marker counts, user-owned
    enterprise-roots counts, `certutil` availability, and Linux Chrome NSS DB
    presence;
  - Android user-CA limitation note;
  - Android release signing policy pointer.
- `support-bundle` now writes `trust.json` next to `meta.json`,
  `config.redacted.json`, `doctor.json`, and `status.json`.
- Full mode is explicitly represented as `ca_required = false` /
  `status = not_required`.

## Cleanup

- Removed duplicated `#[cfg(feature = "ui")]` in `src/lib.rs`.
- No deprecated compatibility path added.
- No generated Gradle output was touched by code changes.

## Split-brain / race assessment

- Split-brain reduced: trust bundle data now has a Rust source instead of being
  manually reconstructed in support-bundle code.
- Race risk low: snapshot is synchronous and read-only. It does not create,
  install, remove, or mutate CA files.
- Remaining split-brain: Doctor CA item and future UI cards still need to call
  or map this snapshot directly.

## Verification

- `cargo fmt --check`
- `cargo test trust_center --features ui --quiet`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all-targets --features ui --quiet` (173 root tests + 5 UI/config tests after read-only browser probe tests)
- `python tools/run-repo-sanity.py`
- `python tools/check-doc-links.py`
- `python tools/check-repo-cleanliness.py`
- tunnel-node clippy/tests were re-run during the preceding audit pass and remained green.

## Remaining risk

- Deeper NSS certificate-presence checks remain pending; current browser probe
  is read-only and does not inspect every NSS DB for the exact CA nickname.
- Bundle preview UI remains pending.
- Android does not yet expose/share the trust snapshot.
- Recent logs were added in the later support-bundle preview increment; this
  trust snapshot remains read-only and does not collect logs itself.
