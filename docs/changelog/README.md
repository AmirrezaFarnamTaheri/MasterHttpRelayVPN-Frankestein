# Changelog directory

- **Per-version release notes** (when used by the release workflow): `v<semver>.md`
  (for example `v1.2.13.md`) — consumed by `.github/workflows/release.yml` when
  present.
- **Maintainer / batch logs:** `batch-*.md` and similar — audit trail for repo
  trust, parity, and governance work; not a substitute for GitHub Release text.

Canonical release artifacts and hashes always come from **CI**
(`.github/workflows/release.yml`), not from this folder.
