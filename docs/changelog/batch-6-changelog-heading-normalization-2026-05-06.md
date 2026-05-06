# Batch 6 - Changelog Heading Normalization - 2026-05-06

## Summary

Normalized the changelog folder so every source changelog file has an explicit
H1 title, then made that title contract part of repo sanity.

## Changed

- Added missing H1 headings to 10 historical changelog files.
- Changed `tools/generate-changelog-index.py` to fail when a source changelog
  is missing an H1 instead of falling back to the filename.
- Added `tools/check-changelog-headings.py`.
- Wired the heading guard into `tools/run-repo-sanity.py`.
- Added the heading guard to `tools/check-ci-local-sanity-parity.py`.
- Updated `docs/changelog/README.md` and `tools/README.md`.
- Regenerated `docs/changelog/index.md`.

## Cleanup

The generated changelog index no longer needs `(no H1)` placeholders. Older
batch notes now read like intentional audit entries rather than loose scratch
files.

## Verification

- `python tools/check-changelog-headings.py`
- `python tools/generate-changelog-index.py`
- `python tools/generate-changelog-index.py -Check`
- `python tools/check-ci-local-sanity-parity.py`
