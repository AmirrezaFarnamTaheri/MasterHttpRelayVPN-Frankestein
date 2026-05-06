# Batch 6 - Android Doctor JNI Bridge - 2026-05-06

<!-- see docs/changelog/v1.1.0.md for the file format: Persian, then `---`, then English. -->
• پل Android برای Doctor اضافه شد: `Native.doctorJson(configJson)` حالا از Kotlin قابل صدا زدن است و همان قرارداد JSON مشترک Doctor را برمی‌گرداند.
• سمت Rust/JNI، config از مسیر canonical `Config::from_json_str` parse می‌شود، Doctor واقعی Rust اجرا می‌شود، و خروجی با `doctor::doctor_report_json_value(&report)` serialize می‌شود. بنابراین Android shape جداگانه‌ای برای diagnostics نمی‌سازد.
• خطاهای invalid config و خطای ساخت runtime هم با همان envelope قراردادی برمی‌گردند: `ok=false` و `items[]` با فیلدهای `id`، `level`، `title`، `detail`، و `fix`.
• guard جدید `tools/check-android-doctor-jni-bridge.py` اضافه و وارد `tools/run-repo-sanity.py` و CI/local parity شد.
• سند `docs/doctor-json-contract.md` و `tools/README.md` به‌روزرسانی شدند. این batch هنوز Android UI card نمی‌سازد؛ فقط backend/bridge parity را تمیز و قابل تست می‌کند تا UI بعدی split-brain نشود.
---
• Added the Android Doctor bridge: `Native.doctorJson(configJson)` is now callable from Kotlin and returns the shared Doctor JSON contract.
• On the Rust/JNI side, config parsing goes through canonical `Config::from_json_str`, Rust Doctor runs normally, and output serializes through `doctor::doctor_report_json_value(&report)`. Android does not gain a separate diagnostics shape.
• Invalid config and runtime-init failures also return the same contract envelope: `ok=false` plus `items[]` with `id`, `level`, `title`, `detail`, and `fix`.
• Added `tools/check-android-doctor-jni-bridge.py` and wired it into `tools/run-repo-sanity.py` plus CI/local parity.
• Updated `docs/doctor-json-contract.md` and `tools/README.md`. This batch does not add the Android UI card yet; it makes the backend bridge clean and testable first so the future UI does not split-brain.
