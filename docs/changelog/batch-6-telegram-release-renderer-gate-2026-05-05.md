# Batch 6 - Telegram Release Renderer Gate

Date: 2026-05-05

## Summary

Hardened the optional Telegram release notification path so changelog text is a
safe projection of `docs/changelog/v*.md`. The script now escapes raw HTML-like
text, converts the project's small Markdown subset to Telegram HTML, and bounds
long changelog replies before posting.

## Changes

- Added `md_to_tg_html`, `html_escape`, and `build_changelog_reply` to
  `.github/scripts/telegram_release_notify.py`.
- Updated the APK caption note path to use the same safe Markdown-to-Telegram
  renderer.
- Added `tools/check-telegram-release-notify.py`.
- Wired the new gate into `tools/run-repo-sanity.py`.
- Documented the gate in `tools/README.md` and `docs/release-checklist.md`.

## Guarded Contract

- Leading changelog editor comments must be stripped.
- `**bold**`, inline-code spans, and Markdown links must render as Telegram
  HTML.
- Raw `<`, `>`, and `&` text must be escaped before posting.
- Changelog replies must stay under a bounded Telegram message budget.
- Telegram remains an optional convenience mirror, not the canonical release
  source of truth.

## Parity / Split-Brain Notes

- The release workflow still reads `docs/changelog/v*.md` when present.
- The GitHub Release remains authoritative for artifacts, checksums, and release
  notes.
- No duplicate donor Telegram publisher was introduced.

## Verification

- `python tools/check-telegram-release-notify.py`
- `python tools/check-doc-links.py`
- `python tools/run-repo-sanity.py --skip-node`
- `python tools/run-repo-sanity.py`
- `python tools/check-repo-cleanliness.py`

All checks passed on 2026-05-05. The first aggregate run caught a literal
Markdown-link example in this changelog that looked like a real local link; the
text was corrected and the docs/sanity checks passed afterward.

## Cleanup

- Removed generated `.github/scripts/__pycache__` and `tools/__pycache__`
  directories after verification.
- No Gradle command was run and no Android build output was generated.
- Process inspection found no active `gradle`, `java`, or `kotlinc` processes
  during closeout.
