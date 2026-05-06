# ADR-0008: No Stale Leftovers Cleanup Policy

## Status

Accepted

## Context

The project has absorbed value from multiple variants and donor folders. That
history is useful, but it increases the risk of stale docs, deprecated config
paths, duplicate examples, generated files, donor binaries, and old UI language
surviving after the active implementation has moved on.

The user explicitly approved a clean-forward policy: do not preserve old
internal shapes or unshipped paths when a cleaner, fully updated shape exists.

## Decision

Every completed batch must remove stale/deprecated leftovers unless they are an
explicit, documented, tested compatibility surface. Cleanup is part of the
definition of done, not a separate someday task.

## Consequences

- Changelog, roadmap, docs, examples, tests, and guards must be updated in the
  same batch as implementation work.
- Donor code and binaries remain quarantined or rejected unless deliberately
  absorbed.
- Backward compatibility is not kept for internal shapes by default.
- Supported compatibility surfaces, such as legacy config imports, need docs and
  tests.
- Generated files should be regenerated deterministically, and local caches
  should be removed after verification.

## Follow-Up

Keep `tools/check-repo-cleanliness.py`, changelog index generation,
release/repo governance guards, and roadmap closeout notes green after every
batch.
