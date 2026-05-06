# Versioning Policy

`mhrv-f` uses pragmatic release versioning around user impact and compatibility.
The project has multiple moving pieces: Rust desktop/CLI, Android JNI/UI,
Apps Script helpers, serverless JSON relays, tunnel-node, config examples, and
docs. A release number should communicate what changed across that whole stack,
not only the Rust crate version.

## Version Bump Guidance

Patch-style release:

- bug fixes;
- documentation fixes;
- guard/tooling changes;
- UI wording or non-breaking layout improvements;
- backend helper hardening that keeps the same deploy/config contract;
- new tests or parity gates.

Minor-style release:

- new user-visible feature;
- new config field with safe default;
- new backend mode or helper variant;
- new Android/Desktop workflow;
- expanded support-bundle or Doctor contract that remains additive;
- significant UI/UX restructuring.

Major-style release:

- config migration that removes or renames canonical fields;
- backend helper protocol break;
- tunnel-node API break;
- removal of a shipped mode;
- signing/install compatibility break;
- deliberate non-backward-compatible cleanup that users must act on.

## Config And Helper Compatibility

- Canonical config output should stay clean; deprecated internal shapes may be
  removed when the repo has tests/docs for the new shape.
- If a change affects Apps Script, Cloudflare Worker, Vercel/Netlify, or
  tunnel-node compatibility, update helper docs, compatibility markers, example
  config, Doctor/readiness checks, and release notes together.
- If Android and Desktop temporarily differ, document the split and add a
  roadmap item. Do not let silent platform drift pass as compatibility.

## Release Notes

Every tagged release should have either:

- `docs/changelog/v<version>.md`; or
- an explicitly generated GitHub Release body from the release workflow.

`docs/RELEASE_NOTES.md` is the rolling staging area for user-facing changes.
Maintainer batch logs remain an audit trail and should not be pasted directly as
the public release body without editing for users.
