<!--
Release notes used by the GitHub Actions `telegram` job:
`.github/workflows/release.yml` calls `.github/scripts/telegram_release_notify.py`
which reads THIS file when `TELEGRAM_INCLUDE_CHANGELOG=true`.

Format rules:
- Persian notes first (will be wrapped in an HTML <blockquote>).
- A separator line that is EXACTLY: ---
- English notes second (also wrapped in <blockquote>).

Keep it short and scannable. Use plain bullet lines.
-->

• (این بخش را برای هر انتشار پر کنید)
• نکات مهم: تغییرات UI/Android/پایداری/امنیت
• اگر چیزی نیاز به اقدام کاربر دارد (مثلاً تغییر config)، همین‌جا بنویسید

---

• (Fill this section for each release)
• Key highlights: UI/Android/stability/security
• If users must take any action (config change, reinstall CA, etc.), say it here
