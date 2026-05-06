# Batch 6 - tunnel-node Drain / Concurrency Gate

Date: 2026-05-05

## Summary

Added a repo-sanity static gate for the tunnel-node v1.9.9 drain correctness and
concurrency fixes. This protects the highest-risk parts of the upstream
absorption: cancellation-safe watcher tasks, no global sessions-map lock across
per-session awaits, mixed TCP/UDP polls that wake on either side, and EOF cleanup
that does not drop over-cap tail bytes.

## Changes

- Added `tools/check-tunnel-node-drain-concurrency.py`.
- Wired the new gate into `tools/run-repo-sanity.py`.
- Documented the gate in `tools/README.md`.

## Guarded Contract

- `wait_for_any_drainable` and `wait_for_any_udp_drainable` must use
  `AbortOnDrop` around watcher tasks.
- Batch TCP/UDP drain lists must carry cloned `Arc<SessionInner>` /
  `Arc<UdpSessionInner>` values instead of re-locking the global sessions maps
  during drains.
- The batch `data` path and single-op `data` path must clone the session inner
  under the global map lock and release that map before awaiting writer/drain
  work.
- Mixed TCP+UDP waits must use empty-aware `tokio::select!`, not
  `tokio::join!`.
- TCP and UDP EOF cleanup must be driven by `drain_now` / `drain_udp_now`
  returned `eof` values, not raw EOF atomics.
- The regression tests for over-cap tail preservation and TCP-ready/UDP-idle
  mixed polling must remain present.
- The tunnel-node settle tuning must remain at `10 ms` step and `1000 ms` max
  unless the platform defaults and docs are deliberately updated together.

## Parity / Split-Brain Notes

- Backend/runtime behavior is guarded directly in the tunnel-node source.
- Desktop and Android behavior is unchanged in this batch.
- The gate complements the executable tunnel-node tests and gives CI/local
  repo-sanity a fast failure mode if the high-risk code shape is undone.

## Verification

- `python tools/check-tunnel-node-drain-concurrency.py`
- `python tools/run-repo-sanity.py --skip-node`
- `python tools/run-repo-sanity.py`
- `python tools/check-doc-links.py`
- `python tools/check-repo-cleanliness.py`

All checks passed on 2026-05-05. The full repo sanity route also re-ran backend
JavaScript syntax checks, Apps Script syntax/tests, Android static parity gates,
generated config/readiness/parity gates, and the stale JSON/XML/Android scan.

## Cleanup

- No generated build output was created.
- No Gradle command was run and no Android build output was generated.
- Removed the generated `tools/__pycache__` directory after verification.
- Process inspection found no active `gradle`, `java`, or `kotlinc` processes
  during this batch's closeout.
