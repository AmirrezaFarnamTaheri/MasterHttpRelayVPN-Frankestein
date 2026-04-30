<div dir="rtl">

# رلهٔ Vercel برای XHTTP (اختیاری)

در این ریپو یک ابزار اختیاری داخل `tools/vercel-xhttp-relay/` قرار داده شده: یک **Vercel Edge Function** که ترافیک **XHTTP** را به سرور Xray شما رله می‌کند.

این ابزار **جزو پروتکل و معماری خود mhrv-f نیست**. فقط برای کاربرانی است که سرور Xray با ترنسپورت **XHTTP** دارند و می‌خواهند جلوی آن را با Vercel فرانت کنند.

## کی به درد می‌خورد؟

- هاست اصلی Xray شما روی بعضی شبکه‌ها بلاک است
- `vercel.com` یا `*.vercel.app` قابل دسترسی است
- ترنسپورت شما **XHTTP** است (نه WS/gRPC/TCP)

## ایدهٔ کلی

- کلاینت به دامنهٔ Vercel وصل می‌شود (TLS از دید فیلتر شبیه Vercel است)
- Edge Function بدنهٔ درخواست را به `TARGET_DOMAIN` شما stream می‌کند و پاسخ را stream برمی‌گرداند
- هندلر عمداً کوچک و مخصوص stream است: بدنهٔ درخواست را مستقیم با `duplex: "half"` به backend می‌دهد، redirect را دستی نگه می‌دارد، و headerهای hop-by-hop / Vercel را قبل از forward حذف می‌کند.

## دیپلوی روی Vercel

1. پوشهٔ `tools/vercel-xhttp-relay/` را به عنوان پروژه روی Vercel deploy کنید.
2. در تنظیمات پروژه، Environment Variable بسازید:
   - `TARGET_DOMAIN`: مثل `https://xray.example.com:2096`
3. یک redeploy انجام دهید.

## نکتهٔ تنظیم کلاینت (VLESS + XHTTP)

الگوی معمول:

- **Address / SNI**: `vercel.com`
- **Host**: دامنهٔ دیپلوی شما (مثل `your-app.vercel.app`)
- **Transport**: `xhttp`
- **Path**: باید دقیقاً با inbound XHTTP سرور Xray شما یکی باشد

## نکات امنیتی/قانونی

- این یک رلهٔ عمومی HTTP است؛ با آن مثل زیرساخت حساس رفتار کنید.
- قوانین و سیاست‌های Vercel و قوانین محلی به عهدهٔ شماست.
- Edge Runtime به WebSocket upgrade، سوکت TCP خام، یا UDP دسترسی نمی‌دهد؛ برای ترنسپورت‌های غیر XHTTP باید معماری دیگری داشته باشید.

</div>
