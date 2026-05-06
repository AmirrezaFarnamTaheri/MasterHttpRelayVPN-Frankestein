# Batch 6 - Upstream v1.9.8/v1.9.9 Stability And Tuning Absorption

Date: 2026-05-04

## Summary

Absorbed the useful, non-duplicative parts of the upstream v1.9.8/v1.9.9
commits into this fork: Android disconnect stability, full-mode batch latency
tuning, tunnel-node drain correctness, Desktop test-button mode gating, and
richer Direct-mode Fastly examples.

## Changes

- Android `MainActivity` now sends only `ACTION_STOP` when the user disconnects.
  It no longer immediately follows with `stopService()`, avoiding an OS
  lifecycle race between `stopSelf()` and `stopService()`.
- Android `MhrvVpnService.teardown()` now stops the Rust proxy before signalling
  and joining tun2proxy. Closing the SOCKS5 listener first wakes the native
  worker's blocking read before the runtime memory is released.
- Full-mode coalescing defaults moved to the upstream low-latency profile:
  `coalesce_step_ms = 10` and `coalesce_max_ms = 1000`.
- Android config defaults, platform-defaults docs, Rust proxy fallback defaults,
  and tunnel-node straggler settle constants now agree.
- Desktop **Test Relay** now explains why `full` and `direct` modes are skipped
  instead of showing a misleading red failure for healthy non-relay modes.
- tunnel-node batch handling now:
  - releases the global sessions map before awaiting TCP writes or drains;
  - wakes mixed TCP/UDP polls with `select!` instead of waiting for both sides;
  - aborts watcher tasks on every cancellation path;
  - cleans up EOF sessions only when the drain operation actually returned EOF,
    preserving over-cap tail bytes.
- Added tunnel-node regression tests for:
  - over-cap buffered TCP sessions surviving until tail bytes are drained;
  - TCP-ready / UDP-idle pure polls returning promptly.
- Expanded `config.fronting-groups.example.json` Fastly starter domains with
  Pinterest, CNN, and BuzzFeed families, keeping them documented as examples to
  verify per network.

## Cross-Platform / Backend / Docs Parity

- Desktop: Test Relay mode guard added and documented in `docs/relay-modes.md`.
- Android: disconnect lifecycle and teardown order updated; English and Persian
  troubleshooting rows updated.
- Backend/runtime: tunnel-node correctness and coalescing behavior updated with
  regression coverage.
- Config/examples: Android defaults, Rust defaults, generated platform-defaults
  docs, advanced-options docs, and Direct-mode fronting example updated.
- Donor/upstream review: exit-node host expansion was not ported because this
  fork has no active `config.exit-node.example.json` product surface; the
  donor-only entry remains quarantined.

## Verification

- `cargo fmt --check` in the root crate.
- `cargo fmt --check` in `tunnel-node/`.
- `cargo test --all-targets --features ui` in the root crate: 184 core tests
  plus 5 UI/config tests passed.
- `cargo clippy --all-targets --all-features -- -D warnings` in the root
  crate.
- `cargo test --all-targets` in `tunnel-node/`: 36 tests passed.
- `cargo clippy --all-targets -- -D warnings` in `tunnel-node/`.
- `python tools/check-platform-defaults.py`.
- `python tools/generate-platform-defaults-doc.py -Check`.
- `python tools/check-android-config-sharing.py`.
- `python tools/check-android-support-redaction.py`.
- `python tools/check-doc-links.py`.
- `python tools/run-repo-sanity.py`.
- `python tools/check-repo-cleanliness.py`.
- No Gradle command was run by this batch. A pre-existing Gradle daemon process
  was visible during process inspection; it was not started or touched by this
  work.

## Cleanup

- Removed generated `tools/__pycache__`.
- Confirmed no remaining `__pycache__` directories under the workspace.
- Did not copy exit-node donor artifacts or create a new exit-node example
  surface.
- Did not touch Gradle wrapper/cache/APK/build outputs.
