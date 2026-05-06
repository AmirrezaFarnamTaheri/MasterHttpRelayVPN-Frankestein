# Batch 6 - Desktop Doctor Summary Card - 2026-05-06

<!-- see docs/changelog/v1.1.0.md for the file format: Persian, then `---`, then English. -->
• کارت خلاصهٔ Doctor به تب Monitor دسکتاپ اضافه شد. خروجی Doctor دیگر فقط در log گم نمی‌شود؛ UI آخرین `DoctorReport` ساخت‌یافته، زمان اجرای آن، وضعیت کلی، و شمارش `ok` / `warn` / `fail` را نگه می‌دارد و نشان می‌دهد.
• اجرای معمولی Doctor حالا report همان اجرا را در state دسکتاپ ذخیره می‌کند. اجرای Doctor+Fix هم report بعد از fix را ذخیره می‌کند، نه وضعیت قدیمی قبل از repair؛ بنابراین کارت Monitor با وضعیت واقعی بعد از تعمیر هماهنگ می‌ماند.
• کارت جدید تا ۵ مورد مهم را نشان می‌دهد: اگر warning/failure وجود داشته باشد همان‌ها اولویت دارند، و اگر همه‌چیز OK باشد چند item سالم اول نمایش داده می‌شود. متن fix هم در همان کارت نمایش داده می‌شود، ولی log کامل همچنان باقی می‌ماند.
• guard جدید `tools/check-desktop-doctor-summary.py` اضافه شد و وارد `tools/run-repo-sanity.py` و CI/local parity شد تا Desktop دوباره به Doctor log-only برنگردد.
• سند `docs/doctor-json-contract.md` و `tools/README.md` به‌روزرسانی شدند. Android هنوز full Doctor bridge ندارد؛ parity gap عمداً در roadmap باقی ماند تا در فاز JNI/UI بعدی بسته شود.
---
• Added a Desktop Monitor Doctor summary card. Doctor output no longer disappears into logs only; the UI now keeps the latest structured `DoctorReport`, run timestamp, overall state, and `ok` / `warn` / `fail` counts.
• Plain Doctor runs store the report they just produced. Doctor+Fix stores the post-fix report, not the stale pre-fix state, so the Monitor card reflects the repaired state.
• The card shows up to 5 important items: warnings/failures first when present, otherwise the first healthy items. Fix text is surfaced inline while the full log remains available.
• Added `tools/check-desktop-doctor-summary.py` and wired it into `tools/run-repo-sanity.py` plus CI/local parity so Desktop cannot silently regress to Doctor log-only output.
• Updated `docs/doctor-json-contract.md` and `tools/README.md`. Android still has no full Doctor bridge; that parity gap remains intentionally tracked for the later JNI/UI phase.
