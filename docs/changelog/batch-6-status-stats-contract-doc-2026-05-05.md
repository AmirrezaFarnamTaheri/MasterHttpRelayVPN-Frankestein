# Batch 6 - Status/Stats Contract Doc - 2026-05-05

<!-- see docs/changelog/v1.1.0.md for the file format: Persian, then `---`, then English. -->
• قرارداد زندهٔ `status` / `stats` مستند شد: فایل جدید `docs/status-stats-json-contract.md` مشخص می‌کند منبع حقیقت فیلدهای آمار `status_api::stats_snapshot_json_value` است، `/status` و `status.json` داخل support bundle همان شیء `stats` را envelope می‌کنند، و Android فعلاً همان شیء raw را از `Native.statsJson(handle)` می‌گیرد.
• قرارداد Android هم صریح شد: Usage Today باید هر دو شکل raw stats و envelope شبیه `/status` را با `root.optJSONObject("stats") ?: root` قبول کند تا مهاجرت‌های بعدی UI یا status DTO باعث شکست card نشود.
• `tools/check-status-stats-json-contract.py` سخت‌گیرتر شد: علاوه بر Rust/JNI/Kotlin، حالا وجود لینک در `docs/index.md` و `tools/README.md`، مستند شدن همهٔ کلیدهای required، و توضیح raw/enveloped shapeها را هم بررسی می‌کند.
• `tools/README.md` و hub اصلی docs به قرارداد جدید لینک شدند، تا این schema فقط در کد پنهان نباشد.
• تست/بررسی: guard قرارداد stats، doc links، sanity runner بدون Node، CI/local parity، و repo cleanliness pass شدند. هیچ Gradle/Java/Kotlin محلی اجرا نشد.
---
• Documented the live `status` / `stats` contract in the new `docs/status-stats-json-contract.md`: `status_api::stats_snapshot_json_value` is the field source of truth, `/status` and support-bundle `status.json` envelope that same stats object, and Android currently consumes the raw object through `Native.statsJson(handle)`.
• Made the Android parser contract explicit: Usage Today must accept both raw stats and a `/status`-style envelope through `root.optJSONObject("stats") ?: root`, so future status DTO migration does not break the existing card.
• Tightened `tools/check-status-stats-json-contract.py`: it now validates Rust/JNI/Kotlin ownership plus docs/index links, tools README links, every required documented key, and the raw/enveloped shape explanation.
• Linked the new contract from `tools/README.md` and the main docs hub so the schema is not hidden only in code.
• Verification: stats contract guard, doc links, no-Node sanity runner, CI/local parity, and repo cleanliness passed. No local Gradle/Java/Kotlin work was run.
