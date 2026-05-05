<!-- Batch changelog: Persian first, then English. -->
• gate استاتیک **Android support redaction owner** اضافه شد:
  - `tools/check-android-support-redaction.py` وجود `SupportRedaction.kt`، استفادهٔ Compose از `androidSupportSnapshot(...)`، و پوشش source تست `SupportRedactionTest.kt` را بررسی می‌کند.
  - اگر `HomeScreen.kt` دوباره helperهای محلی `maskedDeploymentId`، `androidSupportSnapshot` یا `yesNo` را برگرداند، check fail می‌شود.
  - check الزام می‌کند assertionهای حذف `auth_key`، کلید Serverless، توکن LAN، upstream SOCKS5، JSON خام ناشناخته و Deployment ID کامل در source تست باقی بمانند.
• `tools/run-repo-sanity.py` حالا این gate را اجرا می‌کند؛ بنابراین CI `repo-sanity` و اجرای local یک منبع حقیقت مشترک دارند.
• `tools/README.md`، `docs/android.md`، `docs/android.fa.md` و `docs/trust-center.md` توضیح دادند که این gate بدون Gradle اجرا می‌شود و JVM test عمیق‌تر همچنان در CI/pre-provisioned environment است.
• verification:
  - `python tools/check-android-support-redaction.py`
  - `python tools/run-repo-sanity.py`
  - `python tools/check-doc-links.py`
  - `python tools/check-repo-cleanliness.py`
  - cleanup: حذف `tools/__pycache__`
• Gradle اجرا/دانلود/نصب نشد.

---
• Added a static **Android support redaction owner** gate:
  - `tools/check-android-support-redaction.py` verifies `SupportRedaction.kt`, the Compose call to `androidSupportSnapshot(...)`, and the static `SupportRedactionTest.kt` contract markers.
  - The check fails if `HomeScreen.kt` reintroduces local `maskedDeploymentId`, `androidSupportSnapshot`, or `yesNo` helper ownership.
  - The check requires source assertions for omitted `auth_key`, serverless auth key, LAN token, upstream SOCKS5 credentials, raw unknown JSON, and full deployment IDs.
• `tools/run-repo-sanity.py` now runs the gate, so CI `repo-sanity` and local validation share the same source of truth.
• `tools/README.md`, `docs/android.md`, `docs/android.fa.md`, and `docs/trust-center.md` document that this is a no-Gradle static gate; deeper JVM execution remains CI/pre-provisioned.
• Verification:
  - `python tools/check-android-support-redaction.py`
  - `python tools/run-repo-sanity.py`
  - `python tools/check-doc-links.py`
  - `python tools/check-repo-cleanliness.py`
  - cleanup: removed `tools/__pycache__`
• No Gradle run/download/install was performed.
