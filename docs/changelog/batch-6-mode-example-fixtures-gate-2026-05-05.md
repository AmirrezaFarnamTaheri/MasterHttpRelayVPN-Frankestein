# Batch 6 - Mode Example Fixtures Gate - 2026-05-05

<!-- Batch changelog: Persian first, then `---`, then English. -->
• گیت جدید `tools/check-mode-example-fixtures.py` اضافه شد تا مثال‌های کانفیگ هر mode از هم drift نکنند. این گیت بررسی می‌کند که `config.example.json`، `config.vercel-edge.example.json`، `config.direct.example.json`، `config.fronting-groups.example.json`، `config.google-only.example.json` و `config.full.example.json` وجود دارند، مقدار `mode` درست دارند، و در تست Rust `bundled_example_configs_load_and_validate` با `include_str!` واقعاً validate می‌شوند.
• `docs/parity-matrix.json` هم داخل همین گیت کنترل می‌شود: هر mode باید حداقل یک example داشته باشد و هیچ modeای نباید به example اشتباه اشاره کند. این یعنی docs، Rust tests و exampleهای واقعی یک قرارداد مشترک دارند.
• Android هم به‌صورت static در همین گیت محافظت شد: markerهای `preservedUnknownRootJson` و `preservedAccountGroupsJson` و مسیر copy/remove/merge برای کلیدهای ناشناخته چک می‌شوند تا import/edit/export موبایل، تنظیمات پیشرفته‌ای را که Desktop یا کاربر دستی نوشته، بی‌صدا حذف نکند.
• گیت در `tools/run-repo-sanity.py` و `tools/check-ci-local-sanity-parity.py` اضافه شد و در `tools/README.md` مستند شد. ردیف `CM.4` در roadmap کامل شد.
• cleanup/parity: هیچ خروجی build تولید نشد، Gradle اجرا نشد، و تغییر runtime انجام نشد؛ این batch فقط guard و مستندسازی است.
---
• Added `tools/check-mode-example-fixtures.py` so config examples for each mode cannot drift apart. The guard checks that `config.example.json`, `config.vercel-edge.example.json`, `config.direct.example.json`, `config.fronting-groups.example.json`, `config.google-only.example.json`, and `config.full.example.json` exist, declare the expected `mode`, and are actually validated by Rust through `bundled_example_configs_load_and_validate` with `include_str!`.
• The same guard checks `docs/parity-matrix.json`: every mode must list at least one example, and no mode may point at an example whose JSON declares a different mode. Docs, Rust tests, and real examples now share one static contract.
• Android static parity is covered too: the guard checks `preservedUnknownRootJson`, `preservedAccountGroupsJson`, and the copy/remove/merge path for unknown root keys so mobile import/edit/export cannot silently discard advanced settings written by Desktop or by hand.
• Wired the guard into `tools/run-repo-sanity.py` and `tools/check-ci-local-sanity-parity.py`, documented it in `tools/README.md`, and completed roadmap row `CM.4`.
• Cleanup/parity: no build output was produced, no Gradle command was run, and runtime behavior did not change; this batch is a guard/documentation batch only.
