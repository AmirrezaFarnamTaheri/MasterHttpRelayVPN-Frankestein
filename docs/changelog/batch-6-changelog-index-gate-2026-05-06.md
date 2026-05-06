# Batch 6 - Changelog Index Gate - 2026-05-06

## Summary

Added a generated maintainer changelog index so the growing
`docs/changelog/` audit trail remains discoverable and mechanically checked.

## Changed

- Added `tools/generate-changelog-index.py`.
- Generated `docs/changelog/index.md`.
- Linked the index from `docs/index.md`.
- Documented the index in `docs/changelog/README.md`.
- Documented the generator in `tools/README.md`.
- Wired `python tools/generate-changelog-index.py -Check` into
  `tools/run-repo-sanity.py`.
- Added the generator to `tools/check-ci-local-sanity-parity.py`.

## Notes

The generator uses each changelog file's first `#` heading as the title. Older
entries without an H1 are marked with `(no H1)` in the generated table instead
of being silently disguised as polished records.

## Verification

- `python tools/generate-changelog-index.py`
- `python tools/generate-changelog-index.py -Check`
- `python tools/check-ci-local-sanity-parity.py`
