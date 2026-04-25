# Optional future enhancements (MasterHttpRelayVPN-Frankestein)

The `mhrv-f` product is feature-complete for its documented scope. The items below are **optional** follow-ups for contributors or fork maintainers, not blockers for daily use.

## Shipped in the current product

- Per-domain policy (`domain_overrides`: route choice and `never_chunk`).
- Adaptive profiles, degradation, quota-aware cooldowns, and client-side `relay_rate_limit_qps` throttling.
- `doctor` / `doctor-fix`, `rollback-config`, and `support-bundle` for diagnostics.
- Local status API and desktop dashboard widgets.

## Possible extensions (not required)

- **Richer circuit isolation**: per-host open/half-open/closed with explicit recovery probes, beyond script cooldowns and global degradation.
- **Persistent health memory**: store recent failure and blacklist state across process restarts (in addition to config snapshots).
- **Rule pack files**: import/export of domain override lists with optional checksum metadata.
- **Stricter resource limits**: hard caps on buffer sizes and download sizes in addition to the soft rate limiter.
- **Config import wizard**: assist users migrating from ad-hoc JSON layouts with a field mapping report.
- **Deeper UI instrumentation**: structured failure list with in-app “apply fix” where safe and deterministic.

For troubleshooting, use [doctor.md](doctor.md) and the main [README](../README.md).
