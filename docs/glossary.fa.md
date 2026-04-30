<div dir="rtl">

# واژه‌نامه

- **Apps Script relay**: دیپلوی Google Apps Script شما (آدرس `/exec`) که به‌جای شما سایت‌ها را fetch می‌کند.
- **AUTH_KEY / auth_key**: راز مشترک بین `Code.gs` و کانفیگ کلاینت. اگر اشتباه باشد، رله فوراً fail می‌شود.
- **Deployment ID**: شناسهٔ بلند داخل URL دیپلوی Apps Script. می‌توانید URL کامل `/exec` یا ID خام را وارد کنید.
- **Domain fronting**: تکنیکی که اتصال TLS بیرونی شبیه دامنهٔ مجاز (SNI) است، ولی `Host` داخلی مقصد واقعی را مشخص می‌کند.
- **SNI (Server Name Indication)**: نام میزبان در handshake TLS. این پروژه یک pool از SNIها را برای دوام در برابر فیلتر می‌چرخاند.
- **google_ip**: IPv4 لبهٔ گوگل که به آن وصل می‌شوید. اشتباه بودنش از علت‌های رایج timeout است.
- **front_domain**: نام میزبان SNI برای TLS بیرونی (اغلب `www.google.com`).
- **MITM / CA**: در حالت Apps Script، کلاینت HTTPS را محلی باز می‌کند، بایت‌ها را رله می‌کند و دوباره رمز می‌کند. برای این کار باید CA محلی روی دستگاه مورد اعتماد باشد.
- **Full tunnel**: حالتی که ترافیک end-to-end از Apps Script + tunnel-node شما عبور می‌کند، بدون نیاز به نصب CA محلی روی دستگاه کلاینت.
- **tunnel-node**: سرور راه‌دور که در Full Tunnel اجرا می‌کنید.
- **UDP (Full Tunnel)**: در Full Tunnel، UDP می‌تواند سرتاسری عبور کند (برای QUIC/HTTP3، DNS، STUN).
- **Doctor**: `mhrv-f doctor` (و دکمهٔ Doctor در UI) — تشخیص مرحله‌ای و پیشنهاد fixهای عملی.
- **Range-parallel downloads**: ویژگی کارایی برای GETهای بزرگ: یک `Range` کوچک probe می‌شود، سپس بقیهٔ داده‌ها به شکل chunkهای موازی fetch و در نهایت به یک `200 OK` کامل stitch می‌شود.
- **Quota (Apps Script)**: Apps Script سقف روزانهٔ درخواست دارد؛ صفحات سنگین و ویدیو می‌تواند آن را تمام کند. راهکارها: چند deployment، چند account group، و در صورت نیاز rate limit یا fan-out کمتر.

</div>

