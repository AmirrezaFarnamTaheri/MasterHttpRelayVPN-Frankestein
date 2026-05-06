# Batch 6 - Android Doctor Summary Card - 2026-05-06

<!-- see docs/changelog/v1.1.0.md for the file format: Persian, then `---`, then English. -->
• کارت خلاصهٔ Doctor به صفحهٔ اصلی اندروید اضافه شد. این کارت از `Native.doctorJson(configJson)` استفاده می‌کند، همان قرارداد مشترک `ok/items/id/level/title/detail/fix` را parse می‌کند، و وضعیت کلی + شمارش سالم/هشدار/خطا + چند مورد مهم را نشان می‌دهد.
• اجرای Doctor با snapshot از `cfg.toJson()` شروع می‌شود. اگر کاربر هنگام اجرای Doctor کانفیگ را تغییر بدهد، نتیجهٔ قدیمی نادیده گرفته می‌شود تا UI وضعیت stale نشان ندهد.
• همهٔ متن‌های قابل‌مشاهدهٔ کارت به منابع string انگلیسی و فارسی منتقل شدند؛ شمارندهٔ منابع EN/FA با هم به‌روز شد.
• guard جدید `tools/check-android-doctor-summary-ui.py` اضافه و وارد `tools/run-repo-sanity.py` و CI/local parity شد. این guard بررسی می‌کند Android UI از bridge مشترک استفاده کند، parser قرارداد را داشته باشد، stale-result guard باقی بماند، و کلیدهای string هر دو زبان موجود باشند.
• سند `docs/doctor-json-contract.md` و `tools/README.md` به‌روزرسانی شدند. این کار parity عملی بین Desktop summary و Android summary را کامل‌تر می‌کند، بدون اینکه Doctor JSON shape دوم ساخته شود.
---
• Added an Android home-screen Doctor summary card. The card calls `Native.doctorJson(configJson)`, parses the shared `ok/items/id/level/title/detail/fix` contract, and shows overall status, healthy/warn/fail counts, and the most important items.
• Doctor runs start from a `cfg.toJson()` snapshot. If the user changes config while Doctor is running, the stale result is ignored so the UI does not display diagnostics for a config that is no longer current.
• All visible Doctor-card copy moved into English and Persian string resources; EN/FA resource parity was updated together.
• Added `tools/check-android-doctor-summary-ui.py` and wired it into `tools/run-repo-sanity.py` plus CI/local parity. The guard checks that Android UI uses the shared bridge, keeps the contract parser, preserves stale-result protection, and has both language string keys.
• Updated `docs/doctor-json-contract.md` and `tools/README.md`. This completes more practical Desktop/Android Doctor-summary parity without creating a second Doctor JSON shape.
