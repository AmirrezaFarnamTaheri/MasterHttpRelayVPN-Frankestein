<div dir="rtl">

# راهنمای سریع UI دسکتاپ

این صفحه مفاهیم UI دسکتاپ (`mhrv-f-ui`) را به کلیدهای کانفیگ و دستورهای CLI نگاشت می‌کند.

## اعمال اصلی

- **Start / Stop**: روشن/خاموش کردن موتور پروکسی محلی (معادل `mhrv-f serve`).
- **Test relay**: تست انتها به انتها (`mhrv-f test`).
- **Doctor / Doctor + Fix**: عیب‌یابی مرحله‌ای (`mhrv-f doctor` / `mhrv-f doctor-fix`).

## فیلدهای کلیدی (معنی هرکدام)

- **Mode** → `mode`
  - `apps_script`: حالت کلاسیک رله + MITM CA محلی (HTTPS فقط وقتی CA مورد اعتماد باشد)
  - `direct`: حالت bootstrap برای رسیدن به `script.google.com`
  - `full`: حالت Full Tunnel (نیازمند tunnel-node؛ روی کلاینت CA محلی لازم نیست)
- **Google IP** → `google_ip`
- **Front domain (SNI)** → `front_domain`
- **HTTP port** → `listen_port`
- **SOCKS5 port** → `socks5_port`

## اطلاعات رلهٔ Apps Script

در کانفیگ‌های جدید، اطلاعات رله داخل گروه‌های اکانت نگه‌داری می‌شود:

- **Account groups** → `account_groups[]`
  - `auth_key`
  - `script_ids` (یک یا چند deployment ID/URL)

## ابزارهای تشخیصی

- **Scan IPs** → `mhrv-f scan-ips`
- **Test SNI pool** → `mhrv-f test-sni`
- **Scan SNI** → `mhrv-f scan-sni`

## گزینه‌های کارایی/پایداری (پیشرفته)

- **Range parallelism** → `range_parallelism`
  - دانلودهای بزرگ با probe `Range` شروع می‌شوند و سپس چانک‌ها موازی fetch می‌شوند.
  - همزمانی بیشتر معمولاً سریع‌تر است، ولی quota/ریسک خطا را هم بالا می‌برد.
- **Range chunk bytes** → `range_chunk_bytes`
  - چانک بزرگ‌تر تعداد فراخوانی‌های Apps Script را کم می‌کند (quota-friendly) ولی هر تماس طولانی‌تر می‌شود.
- **YouTube via relay** → `youtube_via_relay`
  - یوتیوب را از Apps Script عبور می‌دهد (گاهی برای Restricted Mode مفید است)
  - هزینه: quota مصرف می‌شود و User-Agent ثابت Apps Script اعمال می‌شود
- **Relay QPS limiter** → `relay_rate_limit_qps` / `relay_rate_limit_burst`
  - برای صاف کردن burst و کاهش جهش‌های quota/504
- **Auto-tune** → `runtime_auto_tune` + `runtime_profile`
  - انتخاب خودکار مقادیر پیشنهادی برای چند گزینهٔ مهم

</div>

