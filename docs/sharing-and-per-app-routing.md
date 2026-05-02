# Sharing And Per-App Routing

`mhrv-f` can be used by one app, one device, or several trusted devices on the
same LAN. The safe setup depends on the platform.

## Desktop: Per-App Proxy

Desktop builds expose local HTTP and SOCKS5 proxy listeners:

- HTTP proxy: `listen_host:listen_port` (default `127.0.0.1:8085`)
- SOCKS5 proxy: `listen_host:socks5_port` when `socks5_port` is set
  (the UI default is `127.0.0.1:8086`)

Per-app routing on desktop is explicit opt-in:

1. Keep `listen_host = "127.0.0.1"` for local-only use.
2. Set `socks5_port = 8086` if the app needs SOCKS5.
3. In the app you want to route, set its proxy to HTTP `127.0.0.1:8085` or
   SOCKS5 `127.0.0.1:8086`.
4. Leave other apps without a proxy so they continue using the normal network.

The desktop UI's **Sharing and per-app routing** section displays copy buttons
for the current HTTP and SOCKS endpoints. Use those strings as the source of
truth after changing ports.

Good desktop per-app targets:

- A dedicated browser profile.
- Telegram Desktop or another app with built-in SOCKS/HTTP proxy settings.
- Xray, sing-box, v2rayN, or another local client chained to the SOCKS5 port.
- Command-line tools with `HTTP_PROXY`, `HTTPS_PROXY`, or `ALL_PROXY`.

Examples:

```powershell
$env:HTTP_PROXY = "http://127.0.0.1:8085"
$env:HTTPS_PROXY = "http://127.0.0.1:8085"
curl https://example.com
```

```bash
ALL_PROXY=socks5h://127.0.0.1:8086 curl https://example.com
```

Use `socks5h://` when the client supports it; the `h` means DNS is resolved
through the SOCKS proxy instead of leaking through the local resolver.

True transparent desktop per-app capture is not a simple application setting.
It requires OS-specific packet filtering, a TUN/TAP driver, or a platform VPN
provider. The desktop UI does not expose a fake toggle for that because it would
not be reliable across Windows, macOS, and Linux.

## Desktop: Share To LAN

Use LAN sharing only on a private network you control.

The readiness cards surface LAN sharing as warnings, not ordinary setup
blockers:

- `lan.exposure` appears when `listen_host` is `0.0.0.0` or `::`.
- `lan.token` appears when LAN sharing has no HTTP/CONNECT token and no
  allowlist.
- `lan.allowlist` appears when SOCKS5 is exposed on LAN without an allowlist.

These warnings do not prevent the proxy from starting, because LAN sharing can
be intentional. They are there so an exposed listener is never quiet or
surprising.

1. Set `listen_host = "0.0.0.0"` in the UI's **Sharing and per-app routing**
   section.
2. Keep `listen_port` and `socks5_port` on known values, for example `8085`
   and `8086`.
3. Add `lan_allowlist` entries for the devices that may connect.
4. On the other device, set its HTTP proxy to `<desktop-lan-ip>:8085` or SOCKS5
   proxy to `<desktop-lan-ip>:8086`.

When the UI says `this-device-LAN-IP`, replace it with the desktop's actual
address on that Wi-Fi/Ethernet network. On Windows, `ipconfig` usually shows it
as an IPv4 address under the active adapter.

Recommended config:

```json
{
  "listen_host": "0.0.0.0",
  "listen_port": 8085,
  "socks5_port": 8086,
  "lan_token": null,
  "lan_allowlist": ["192.168.1.42", "192.168.1.0/24"]
}
```

`lan_allowlist` accepts exact IPs and CIDR ranges. Use exact IPs when your
router gives stable DHCP leases. Use a CIDR range only when every device in
that range is trusted.

`lan_token` protects HTTP proxy clients that can send the
`X-MHRV-F-Token` header. SOCKS5 has no header preface, so it cannot use this
token. If you expose SOCKS5 on LAN, set `lan_allowlist`; otherwise SOCKS5 fails
closed when a token is configured without an allowlist.

Recommended rollout:

1. Test local HTTP proxy on the desktop first.
2. Switch to **Share on LAN**.
3. Add the client device IP to **Allowed IPs**.
4. Test HTTP from the second device.
5. Test SOCKS5 from the second device.
6. Watch the desktop **Recent log** while connecting; a rejected LAN client
   usually means the allowlist does not match the client IP.

## Android: App Splitting

Android VPN mode uses Android's native `VpnService` app filters:

- **All apps**: route every eligible app through the tunnel.
- **Only selected apps**: allow-list mode; only picked apps use the tunnel.
- **All except selected**: deny-list mode; selected apps stay direct.

The app always excludes itself from the VPN path so its own management traffic
does not loop back into the tunnel.

## Android: Proxy-Only Per-App Opt-In

Proxy-only mode starts the local HTTP/SOCKS listeners but does not create a
system VPN.

Use it when:

- Another VPN app already owns the Android VPN slot.
- You want one app or one Wi-Fi profile to opt in manually.
- A specific app supports proxy settings but does not tolerate user CA MITM in
  VPN mode.

After connecting in Proxy-only mode, configure apps or Wi-Fi settings with:

- HTTP: `127.0.0.1:<http-port>`
- SOCKS5: `127.0.0.1:<socks5-port>`

Only apps that honor those settings use `mhrv-f`.

## Android: Share Phone Proxy To LAN

The Android advanced **Share proxy on LAN** switch binds listeners to
`0.0.0.0`, allowing trusted devices on the same Wi-Fi or hotspot network to
use the phone as a proxy.

This is proxy sharing, not guaranteed OS-level VPN hotspot sharing. Android
vendors differ on whether VPN traffic is forwarded over hotspot. The reliable
path is to configure the other device to use the phone's HTTP or SOCKS proxy
address directly.

Use only on trusted networks. Any device that can reach the exposed ports can
spend your Apps Script quota.

Practical phone-to-device recipe:

1. Connect both devices to the same Wi-Fi or phone hotspot.
2. Start `mhrv-f` on the phone.
3. Enable **Share proxy on LAN**.
4. Find the phone's LAN/hotspot IP in Android network settings.
5. On the other device, set HTTP proxy to `<phone-ip>:<http-port>` or SOCKS5
   proxy to `<phone-ip>:<socks5-port>`.
6. Open a browser on the other device and test a simple HTTPS page.

Common hotspot defaults:

- Android hotspot gateway is often `192.168.43.1`, but vendors differ. Use the
  IP Android shows for the hotspot when available.
- iOS Wi-Fi HTTP proxy: Settings -> Wi-Fi -> tap the hotspot -> Configure Proxy
  -> Manual -> Server `<phone-ip>`, Port `<http-port>`.
- iOS full-device proxy apps such as Shadowrocket or Potatso can point at the
  phone's SOCKS5 endpoint, for example `<phone-ip>:1081`, and create a local VPN
  on the iOS device.
- macOS/Windows can use system HTTP proxy settings for HTTP(S) browsing, or a
  per-app SOCKS5 setting for apps that support it.

## Troubleshooting

- Other device cannot connect: check firewall rules, same Wi-Fi/VLAN, and the
  actual LAN IP of the desktop or phone.
- HTTP works but SOCKS5 fails: add `lan_allowlist`. SOCKS5 cannot use
  `lan_token`.
- Browser connects but HTTPS fails: install/trust the local CA on that client
  for MITM modes, or use `full` mode where local MITM is not required.
- Android app not routed: check App splitting mode and whether the app trusts
  user-installed CAs on Android 7+.
