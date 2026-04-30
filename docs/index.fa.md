<div dir="rtl">

# شروع از اینجا — MasterHttpRelayVPN-Frankestein (`mhrv-f`)

این صفحه مرکز مستندات **MasterHttpRelayVPN-Frankestein** است. در نام فایل‌ها، دستورها و باینری‌ها از نام کوتاه `mhrv-f` استفاده می‌شود.

## هدف خودتان را انتخاب کنید

### ۱) دسکتاپ: مرور وب با پروکسی محلی (رایج‌ترین)

- UI دسکتاپ: `mhrv-f-ui` (پیشنهادی برای اولین اجرا) یا CLI: `mhrv-f`
- راهنمای نصب اصلی: [`README.md`](../README.md#راهنمای-فارسی)
- اگر جایی شکست خورد: [رفع اشکال](#رفع-اشکال)

### ۲) اندروید: مسیریابی سراسری (VPN/TUN) یا فقط پروکسی

- راهنمای کامل اندروید: [`docs/android.fa.md`](android.fa.md) (فارسی) / [`docs/android.md`](android.md) (English)
- برنامهٔ اندروید همین موتور Rust را اجرا می‌کند و با VPN (TUN) می‌پیچد.

### ۳) فول‌تونل (پیشرفته): تونل سرتاسری با tunnel-node

- وقتی استفاده کنید که `tunnel-node` را خودتان اجرا می‌کنید و می‌خواهید ترافیک کامل دستگاه از آن عبور کند.
- در نسخه‌های جدید، در حالت Full Tunnel، **UDP** سرتاسری پشتیبانی می‌شود (QUIC/HTTP3، DNS، STUN).
- راهنما: [`tunnel-node/README.md`](../tunnel-node/README.md)

## سریع چک کنید که کار می‌کند

- در UI دسکتاپ: **Test relay** سپس **Doctor**
- در CLI:

```bash
./mhrv-f test
./mhrv-f doctor
```

## رفع اشکال

- دکتر (تشخیصی مرحله‌ای): [`docs/doctor.fa.md`](doctor.fa.md) / [`docs/doctor.md`](doctor.md)
- عیب‌یابی بر اساس علائم: [`docs/troubleshooting.fa.md`](troubleshooting.fa.md) / [`docs/troubleshooting.md`](troubleshooting.md)

## گزینه‌های پیشرفته

- توضیح کامل همهٔ گزینه‌های پیشرفته و اثرشان روی سرعت/پایداری/quota:
  - English: [`docs/advanced-options.md`](advanced-options.md)
  - فارسی: [`docs/advanced-options.fa.md`](advanced-options.fa.md)

## امنیت و اعتماد (به زبان ساده)

- CA چیست، چه چیزی کجا رمزگشایی می‌شود، و روش حذف امن: [`docs/safety-security.fa.md`](safety-security.fa.md) / [`docs/safety-security.md`](safety-security.md)

## واژه‌نامه

- اصطلاحات پرکاربرد: [`docs/glossary.fa.md`](glossary.fa.md) / [`docs/glossary.md`](glossary.md)

## UI دسکتاپ (راهنمای سریع)

- فارسی: [`docs/ui-desktop.fa.md`](ui-desktop.fa.md)
- English: [`docs/ui-desktop.md`](ui-desktop.md)

## رلهٔ Vercel برای XHTTP (اختیاری)

- فارسی: [`docs/vercel-xhttp-relay.fa.md`](vercel-xhttp-relay.fa.md)
- English: [`docs/vercel-xhttp-relay.md`](vercel-xhttp-relay.md)

## `udpgw` (UDP در Full Tunnel، پایداری VoIP)

- فارسی: [`docs/udpgw.fa.md`](udpgw.fa.md)
- English: [`docs/udpgw.md`](udpgw.md)

</div>

