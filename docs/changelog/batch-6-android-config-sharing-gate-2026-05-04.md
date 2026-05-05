<!-- Batch changelog: Persian first, then English. -->
• مسیر QR/deep-link اندروید محکم‌تر شد:
  - `ConfigStore.encode()` حالا export را از `preservedUnknownRootJson` شروع می‌کند و بعد keyهای owned اندروید را overwrite می‌کند؛ بنابراین root keyهای advanced که از Desktop یا config دستی آمده‌اند در QR/share از بین نمی‌روند.
  - تست‌های contract در `ConfigStoreTest.kt` اضافه شد برای scheme فعلی `mhrvf://`، import legacy `mhrv-rs://`، حفظ unknown rootها، و reject شدن payload نامعتبر.
  - `tools/check-android-config-sharing.py` اضافه شد تا همین قراردادها را به صورت استاتیک و بدون Gradle در local/CI نگه دارد.
  - `tools/run-repo-sanity.py` این gate را اجرا می‌کند.
• docs به‌روزرسانی شد: `tools/README.md`، `docs/android.md` و `docs/android.fa.md`.
• verification:
  - `python tools/check-android-config-sharing.py`
  - `python tools/check-android-support-redaction.py`
  - `python tools/run-repo-sanity.py`
  - `python tools/check-doc-links.py`
  - `python tools/check-repo-cleanliness.py`
  - cleanup: حذف `tools/__pycache__`
• Gradle اجرا/دانلود/نصب نشد.

---
• Hardened Android QR/deep-link config sharing:
  - `ConfigStore.encode()` now seeds export JSON from `preservedUnknownRootJson` before overwriting Android-owned keys, so advanced Desktop/manual root keys survive QR/share export.
  - Added `ConfigStoreTest.kt` contract coverage markers for current `mhrvf://` export, legacy `mhrv-rs://` import, unknown-root preservation, and invalid-payload rejection.
  - Added `tools/check-android-config-sharing.py` as a static no-Gradle local/CI gate for the same contract.
  - Wired the gate into `tools/run-repo-sanity.py`.
• Updated `tools/README.md`, `docs/android.md`, and `docs/android.fa.md`.
• Verification:
  - `python tools/check-android-config-sharing.py`
  - `python tools/check-android-support-redaction.py`
  - `python tools/run-repo-sanity.py`
  - `python tools/check-doc-links.py`
  - `python tools/check-repo-cleanliness.py`
  - cleanup: removed `tools/__pycache__`
• No Gradle run/download/install was performed.
