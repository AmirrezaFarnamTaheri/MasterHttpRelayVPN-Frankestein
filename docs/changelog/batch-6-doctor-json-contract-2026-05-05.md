# Batch 6 - Doctor JSON Contract - 2026-05-05

<!-- see docs/changelog/v1.1.0.md for the file format: Persian, then `---`, then English. -->
• قرارداد JSON برای Doctor اضافه شد: `doctor::doctor_report_json_value` و `doctor::doctor_item_json_value` حالا منبع واحد shape خروجی structured diagnostics هستند.
• `support-bundle` دیگر `doctor.json` را با لیست دستی فیلدها نمی‌سازد؛ به renderer مشترک Doctor وصل شد. این کار جلوی split-brain بین CLI/Doctor/support bundle و کارت‌های آیندهٔ Desktop/Android diagnostics را می‌گیرد.
• سند جدید `docs/doctor-json-contract.md` shape فعلی را مستند می‌کند: top-level `ok` و `items`، و برای هر item فیلدهای `id`، `level`، `title`، `detail`، و `fix`. سطح‌ها هم به صورت ثابت `ok` / `warn` / `fail` تعریف شدند.
• guard جدید `tools/check-doctor-json-contract.py` اضافه و وارد `tools/run-repo-sanity.py` و parity gate شد. این guard بررسی می‌کند support bundle دوباره JSON را دستی نسازد، سند و docs hub لینک داشته باشند، و همهٔ فیلدهای قرارداد مستند باشند.
• تست Rust برای renderer جدید اضافه شد تا shape فعلی support-bundle-compatible باقی بماند.
---
• Added a Doctor JSON contract: `doctor::doctor_report_json_value` and `doctor::doctor_item_json_value` are now the single structured diagnostics output shape.
• `support-bundle` no longer hand-builds `doctor.json`; it calls the shared Doctor renderer. This prevents split-brain between CLI/Doctor/support bundle and future Desktop/Android diagnostics cards.
• New `docs/doctor-json-contract.md` documents the current shape: top-level `ok` and `items`, and per-item `id`, `level`, `title`, `detail`, and `fix`. Levels are the stable strings `ok` / `warn` / `fail`.
• Added `tools/check-doctor-json-contract.py` and wired it into `tools/run-repo-sanity.py` plus CI/local parity. The guard checks that support bundle does not re-hand-build JSON, docs hub/tools links exist, and all contract fields are documented.
• Added a Rust renderer regression test so the support-bundle-compatible shape remains locked.
