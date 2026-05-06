# Contributing

Thanks for helping with `mhrv-f`. This project spans Rust desktop/CLI, Android,
Apps Script helpers, serverless relays, tunnel-node, and a large docs surface.
Small changes are welcome, but every change should keep parity and security in
view.

## Before You Change Code

1. Read [`docs/index.md`](docs/index.md) for the docs map.
2. Read [`CHANGELOG.md`](CHANGELOG.md) for release/changelog expectations.
3. For security-sensitive changes, read [`SECURITY.md`](SECURITY.md) and
   [`docs/safety-security.md`](docs/safety-security.md).
4. For Android signing or release work, read
   [`docs/android-signing.md`](docs/android-signing.md) and
   [`docs/release-checklist.md`](docs/release-checklist.md).
5. For architecture decisions, source-of-truth choices, compatibility
   boundaries, or security/release tradeoffs, read
   [`docs/adr/README.md`](docs/adr/README.md).

## Local Verification

Start with the profile that matches the change type:
[`docs/verification-profiles.md`](docs/verification-profiles.md). Profiles are
additive, so a config change with Android UI impact should run both profiles.
Use [`docs/change-impact-checklist.md`](docs/change-impact-checklist.md) to map
touched surfaces to parity checks and cleanup expectations.
Use [`docs/tooling-source-map.md`](docs/tooling-source-map.md) when editing
generated or guarded contract docs.

Broad local sanity:

```powershell
python tools\run-repo-sanity.py
```

If Node.js or PowerShell is not available:

```powershell
python tools\run-repo-sanity.py --skip-node
python tools\run-repo-sanity.py --skip-readiness
```

Rust checks:

```powershell
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --features ui
```

tunnel-node checks:

```powershell
Push-Location tunnel-node
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
Pop-Location
```

Android JVM tests are CI/pre-provisioned. Local repo sanity intentionally uses
static Android guards and does not require Gradle.

## Parity Expectations

When changing config, runtime behavior, diagnostics, release process, or UI,
check the affected surfaces:

- Desktop UI
- Android UI
- CLI
- backend helpers
- tunnel-node
- examples
- generated docs
- English/Persian docs where user-facing
- changelog and roadmap

If a surface intentionally differs, document the reason and add a roadmap item.

## Changelog And Roadmap

For implementation batches:

1. Start from [`docs/changelog/TEMPLATE.md`](docs/changelog/TEMPLATE.md).
2. Add a batch note under `docs/changelog/`.
3. Update `elevation_audit_roadmap_source.md`.
4. Regenerate the changelog index:

   ```powershell
   python tools\generate-changelog-index.py
   ```

5. Run:

   ```powershell
   python tools\check-changelog-headings.py
   python tools\generate-changelog-index.py -Check
   python tools\check-release-governance.py
   python tools\check-verification-profiles.py
   python tools\check-change-impact-checklist.py
   python tools\check-tooling-source-map.py
   ```

## Architecture Decisions

Use [`docs/adr/README.md`](docs/adr/README.md) for decisions that affect more
than one surface or create a lasting tradeoff. Examples include signing
material, release artifact authority, config migration boundaries, status/Doctor
schemas, platform-default differences, donor absorption/rejection, and cleanup
policy.

When adding or changing an ADR:

1. Start from [`docs/adr/TEMPLATE.md`](docs/adr/TEMPLATE.md).
2. Link the ADR from [`docs/adr/README.md`](docs/adr/README.md).
3. Update changelog and roadmap in the same batch.
4. Run:

   ```powershell
   python tools\check-adr-governance.py
   ```

## Cleanup Rule

After each completed change:

- remove stale/deprecated code and docs;
- refresh generated files;
- remove temporary caches;
- avoid preserving backward compatibility for internal shapes unless explicitly
  documented and tested;
- verify no split-brain source of truth was introduced.
