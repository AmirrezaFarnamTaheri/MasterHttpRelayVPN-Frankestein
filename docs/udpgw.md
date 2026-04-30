# `udpgw` (native UDP gateway) — Full Tunnel mode

`udpgw` is a **native UDP gateway** that runs inside your `tunnel-node`. It exists to make **UDP-heavy apps** (especially VoIP) work reliably in **Full Tunnel** mode.

## Why `udpgw` exists (even if you have SOCKS5 UDP ASSOCIATE)

There are two different UDP paths in the project, and they serve different kinds of traffic:

- **SOCKS5 UDP ASSOCIATE**
  - Used by apps that *explicitly* negotiate SOCKS5 UDP relay.
  - The relay opens **one tunnel session per UDP destination** and drains/polls each independently.
  - On high-latency or shaky networks, dozens of concurrent UDP flows can degrade (each flow pays its own polling overhead).

- **`udpgw`**
  - Used for **TUN-captured UDP** in **Full Tunnel** mode.
  - Multiplexes **all UDP over one persistent “TCP-like” session** using `conn_id` framing.
  - Keeps a **stable source port per flow** by maintaining persistent sockets per `(conn_id, destination)` — important for protocols like **STUN** and **Telegram VoIP**.

## How it’s addressed (magic destination)

The Android full-tunnel client enables `udpgw` by passing the special destination:

- **`198.19.255.254:7300`**

This is a *magic* address. The `tunnel-node` intercepts it and creates a virtual in-process session wired to the `udpgw` server task.

Notes:
- `198.18.0.0/15` is reserved for benchmarking (RFC 2544) and is not a real Internet destination.
- The address is intentionally near the end of the reserved range to avoid colliding with tun2proxy virtual-DNS fake IPs, which commonly start near `198.18.0.1`.

## QUIC and DNS are blocked in `udpgw` (by design)

The `tunnel-node` deliberately blocks:

- **UDP 443 (QUIC / HTTP3)**: going through `udpgw` is typically slower than falling back to TCP/HTTP2 over the batch pipeline. Blocking QUIC forces browsers to use TCP, improving YouTube and general browsing speed.
- **UDP 53 (DNS)**: DNS is better handled by tun2proxy’s virtual DNS / SOCKS5 UDP path (more reliable for small request-response exchanges).

VoIP and other UDP (STUN/RTP/etc.) still flow through `udpgw` normally.

## Deployment requirement (important)

Enabling `udpgw` requires a **new `tunnel-node` deployment** that includes the `udpgw` module and the magic-address interception.

If your client is updated but your server is not, Full Tunnel UDP behavior will not improve (and you may see errors/timeouts for the magic destination).

## Troubleshooting quick hints

- **VoIP breaks / STUN failures**: make sure your Full Tunnel mode is using a `tunnel-node` build that includes `udpgw`, and that the client is passing `198.19.255.254:7300` when in Full mode.
- **Browsing slow**: this is *exactly why* UDP 443 is blocked in `udpgw` — ensure your browser is not forcing QUIC, and confirm you’re on a recent tunnel-node build.
