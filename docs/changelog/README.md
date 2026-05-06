# Changelog directory

- **Per-version release notes** (when used by the release workflow): `v<semver>.md`
  (for example `v1.2.13.md`) — consumed by `.github/workflows/release.yml` when
  present.
- **Maintainer / batch logs:** `batch-*.md` and similar — audit trail for repo
  trust, parity, and governance work; not a substitute for GitHub Release text.
- **Generated index:** [`index.md`](index.md) lists every changelog file by date,
  filename, and first heading. Regenerate it with
  `python tools/generate-changelog-index.py`; repo sanity checks it with
  `python tools/generate-changelog-index.py -Check`.
- **Heading contract:** every changelog file except this README and the
  generated index must have a first-level `#` heading. Repo sanity enforces this
  with `python tools/check-changelog-headings.py`.
- **Template:** start new maintainer batch notes from [`TEMPLATE.md`](TEMPLATE.md)
  so UI/UX, config/schema, backend, security/trust, docs, breaking cleanup,
  parity, race/split-brain, verification, and cleanup sections are considered
  every time.

Canonical release artifacts and hashes always come from **CI**
(`.github/workflows/release.yml`), not from this folder.
