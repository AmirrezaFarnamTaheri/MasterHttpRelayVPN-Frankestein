# Batch 6 - Readiness UI Contract Gate - 2026-05-05

<!-- Batch changelog: Persian first, then `---`, then English. -->
• گیت جدید `tools/check-readiness-ui-contract.py` اضافه شد تا readiness IDها و repair actionها بین Rust، Desktop، Android و docs دوشاخه نشوند. منبع اصلی همچنان `src/readiness.rs` است و گیت چک می‌کند همهٔ IDها در `android/app/src/main/java/com/farnam/mhrvf/ReadinessIds.kt` و `docs/readiness-matrix.md` حضور دارند.
• Desktop اکنون با گیت static محافظت می‌شود: `ModeReadinessItem` باید از `readiness::ReadinessId` استفاده کند و repair actionها باید از `readiness::repair_for_id` و `readiness::repair_anchor_for_target` بیایند، نه stringهای جداگانه.
• Android Home هم محافظت شد: repair cardها باید از `ReadinessIds`، `ReadinessRepairTargets` و `ReadinessRepairAnchors` generated استفاده کنند. گیت markerهای پرریسک مثل Apps Script credentials، Serverless JSON، Direct fronting، CA trust، LAN sharing و Full tunnel را چک می‌کند.
• این گیت در `tools/run-repo-sanity.py` و `tools/check-ci-local-sanity-parity.py` اضافه شد و در `tools/README.md` مستند شد. ردیف `CS.3` در roadmap کامل شد. ردیف `CS.4` عمداً کامل نشده؛ تبدیل Doctor itemها به cardهای richer هنوز کار جداگانه است.
• cleanup/parity: هیچ خروجی build تولید نشد، Gradle اجرا نشد، و تغییر runtime انجام نشد؛ این batch فقط قرارداد static و مستندسازی است.
---
• Added `tools/check-readiness-ui-contract.py` so readiness IDs and repair actions cannot split across Rust, Desktop, Android, and docs. `src/readiness.rs` remains the source of truth, and the guard checks that every ID appears in generated `android/app/src/main/java/com/farnam/mhrvf/ReadinessIds.kt` and `docs/readiness-matrix.md`.
• Desktop is now statically protected: `ModeReadinessItem` must use `readiness::ReadinessId`, and repair actions must come from `readiness::repair_for_id` plus `readiness::repair_anchor_for_target`, not duplicate strings.
• Android Home is protected too: repair cards must consume generated `ReadinessIds`, `ReadinessRepairTargets`, and `ReadinessRepairAnchors`. The guard checks high-risk families such as Apps Script credentials, Serverless JSON, Direct fronting, CA trust, LAN sharing, and Full tunnel.
• Wired the guard into `tools/run-repo-sanity.py` and `tools/check-ci-local-sanity-parity.py`, documented it in `tools/README.md`, and completed roadmap row `CS.3`. Roadmap row `CS.4` is intentionally not complete; richer Doctor item cards remain separate work.
• Cleanup/parity: no build output was produced, no Gradle command was run, and runtime behavior did not change; this batch is a static contract/documentation batch only.
