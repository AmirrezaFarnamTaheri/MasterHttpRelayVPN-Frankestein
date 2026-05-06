# Batch 6 - Android String Resource Parity Gate - 2026-05-05

<!-- Batch changelog: Persian first, then `---`, then English. -->
• گیت جدید `tools/check-android-string-resource-parity.py` اضافه شد تا `android/app/src/main/res/values/strings.xml` و `android/app/src/main/res/values-fa/strings.xml` از نظر کلیدهای localization دوشاخه نشوند.
• گیت جدید هر دو فایل XML را parse می‌کند، duplicate key را fail می‌کند، blank string را fail می‌کند، و تضمین می‌کند هیچ کلیدی فقط در انگلیسی یا فقط در فارسی وجود ندارد. وضعیت فعلی: ۲۱۳ کلید در هر دو locale و صفر mismatch.
• چند کلید پرریسک هم به‌صورت explicit محافظت شدند: نام برنامه، Mode، بخش‌های Apps Script / Serverless JSON، دکمه‌های Connect/Disconnect/Install MITM، و labelهای repair برای Apps Script، Serverless، Direct، CA trust، LAN و Full tunnel.
• گیت در `tools/run-repo-sanity.py` و `tools/check-ci-local-sanity-parity.py` اضافه شد و در `tools/README.md` مستند شد. ردیف‌های `B0.4` و `B4.4` در roadmap کامل شدند.
• cleanup/parity: Gradle اجرا نشد، خروجی build تولید نشد، و تغییر runtime انجام نشد. مهاجرت کامل متن‌های hard-coded Compose هنوز جداگانه در `B0.3`، `B4.1`، `B4.2` و `B4.3` دنبال می‌شود.
---
• Added `tools/check-android-string-resource-parity.py` so `android/app/src/main/res/values/strings.xml` and `android/app/src/main/res/values-fa/strings.xml` cannot drift in localization keys.
• The guard parses both XML files, fails duplicate keys, fails blank values, and guarantees there are no keys that exist only in English or only in Persian. Current state: 213 keys in each locale and zero mismatches.
• Several high-risk keys are explicitly protected too: app name, Mode, Apps Script / Serverless JSON sections, Connect/Disconnect/Install MITM buttons, and repair labels for Apps Script, Serverless, Direct, CA trust, LAN, and Full tunnel.
• Wired the guard into `tools/run-repo-sanity.py` and `tools/check-ci-local-sanity-parity.py`, documented it in `tools/README.md`, and completed roadmap rows `B0.4` and `B4.4`.
• Cleanup/parity: no Gradle command was run, no build output was produced, and runtime behavior did not change. Full migration of hard-coded Compose copy remains tracked separately by `B0.3`, `B4.1`, `B4.2`, and `B4.3`.
