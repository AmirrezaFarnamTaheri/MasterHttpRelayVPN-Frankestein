# Changelog

This file is the human-facing release-notes hub for `mhrv-f`.

## Source Of Truth

- **Canonical public release:** GitHub Releases created by
  `.github/workflows/release.yml`, including artifacts and `SHA256SUMS.txt`.
- **Per-version release notes:** `docs/changelog/v<version>.md` when present.
  These are the preferred hand-written notes for a tagged release and can be
  projected into GitHub Release text and optional Telegram announcements.
- **Rolling unreleased notes:** `docs/RELEASE_NOTES.md` for user-facing changes
  that are not yet organized into a tagged `v*.md` note.
- **Maintainer audit trail:** `docs/changelog/batch-*.md` plus the generated
  `docs/changelog/index.md`. These explain implementation batches and
  verification, but they are not the public release body by themselves.

## Maintainer Rules

When a change affects users, config shape, backend helpers, security/trust,
Android/Desktop parity, release artifacts, or docs:

1. Update the relevant public notes surface:
   - `docs/changelog/v<version>.md` for an imminent tagged release; or
   - `docs/RELEASE_NOTES.md` for rolling unreleased notes.
2. Add or update the batch log under `docs/changelog/`.
3. Regenerate the changelog index:

   ```powershell
   python tools\generate-changelog-index.py
   ```

4. Run the release-governance and changelog drift gates:

   ```powershell
   python tools\check-release-governance.py
   python tools\check-changelog-headings.py
   python tools\generate-changelog-index.py -Check
   ```

## Links

- Rolling notes: [`docs/RELEASE_NOTES.md`](docs/RELEASE_NOTES.md)
- Maintainer changelog index: [`docs/changelog/index.md`](docs/changelog/index.md)
- Changelog folder policy: [`docs/changelog/README.md`](docs/changelog/README.md)
- Batch template: [`docs/changelog/TEMPLATE.md`](docs/changelog/TEMPLATE.md)
- Release checklist: [`docs/release-checklist.md`](docs/release-checklist.md)
