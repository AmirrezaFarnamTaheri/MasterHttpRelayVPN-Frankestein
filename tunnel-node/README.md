# Tunnel Node

HTTP tunnel bridge server for MasterHttpRelayVPN "full" mode. Bridges Apps Script tunnel requests to real TCP and UDP destinations.

## Architecture

```
Phone → mhrv-f → [domain-fronted TLS] → Apps Script → [HTTP] → Tunnel Node → [real TCP/UDP] → Internet
```

The tunnel node manages persistent TCP and UDP sessions. TCP sessions are real TCP connections; UDP sessions are connected UDP sockets pinned to one destination host:port. Data flows through a JSON protocol:

- **connect** — open TCP to host:port, return session ID
- **connect_data** — same as connect, then write an optional base64 first chunk and return the first read (one RTT for speak-first protocols)
- **data** — write client data, return server response
- **udp_open** — open UDP to host:port, optionally send the first datagram
- **udp_data** — send one UDP datagram, or poll for returned datagrams when `d` is omitted
- **close** — tear down session
- **batch** — process multiple ops in one HTTP request (reduces round trips)
- Error responses may include a structured **`code`** field (e.g. `UNSUPPORTED_OP` for unknown ops) so clients detect tunnel-node / Apps Script version skew without parsing the message text.

## Deployment

### Cloud Run

```bash
cd tunnel-node
gcloud run deploy tunnel-node \
  --source . \
  --region us-central1 \
  --allow-unauthenticated \
  --set-env-vars TUNNEL_AUTH_KEY=$(openssl rand -hex 24) \
  --memory 256Mi \
  --cpu 1 \
  --max-instances 1
```

### Docker — prebuilt image (any VPS)

The release workflow publishes a multi-arch image for `linux/amd64` and `linux/arm64`. Pulling it avoids installing Rust on a small VPS.

```bash
SECRET=$(openssl rand -hex 24)
echo "Your TUNNEL_AUTH_KEY: $SECRET"

docker run -d \
  --name mhrv-tunnel \
  --restart unless-stopped \
  -p 8080:8080 \
  -e TUNNEL_AUTH_KEY="$SECRET" \
  ghcr.io/OWNER/mhrv-tunnel-node:latest
```

Replace `OWNER` with the GitHub account or organization that published your release image. For production, pin a version tag instead of `latest`.

### Docker — build from source

```bash
cd tunnel-node
docker build -t tunnel-node .
docker run -p 8080:8080 -e TUNNEL_AUTH_KEY=your-secret tunnel-node
```

### Direct binary

```bash
cd tunnel-node
cargo build --release
TUNNEL_AUTH_KEY=your-secret PORT=8080 ./target/release/tunnel-node
```

### Direct binary as a systemd service

For a small VPS without Docker, install the compiled binary under `/opt` and
run it with the included service template:

```bash
cd tunnel-node
cargo build --release
SECRET=$(openssl rand -hex 24)
echo "Use this same TUNNEL_AUTH_KEY in CodeFull.gs: $SECRET"

sudo install -d /opt/mhrv-f/tunnel-node
sudo install -m 0755 target/release/tunnel-node /opt/mhrv-f/tunnel-node/tunnel-node
sudo cp mhrv-tunnel-node.service.example /etc/systemd/system/mhrv-tunnel-node.service
sudo sed -i "s/replace-with-the-same-secret-as-CodeFull-gs/$SECRET/" /etc/systemd/system/mhrv-tunnel-node.service
sudo systemctl daemon-reload
sudo systemctl enable --now mhrv-tunnel-node
curl http://127.0.0.1:8080/healthz
```

Use the final public `https://...` tunnel-node URL in `CodeFull.gs`. The Apps
Script tunnel forwarder intentionally does not follow redirects for
`/tunnel` or `/tunnel/batch`, because those requests carry the tunnel auth key.

## Environment Variables

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `TUNNEL_AUTH_KEY` | Yes | none | Shared secret — must match `TUNNEL_AUTH_KEY` in CodeFull.gs. The service refuses to start if missing |
| `PORT` | No | `8080` | Listen port (Cloud Run sets this automatically) |

`MHRV_AUTH_KEY` is not read by tunnel-node. If you accidentally set that name
instead of `TUNNEL_AUTH_KEY`, startup logs call it out and the server exits.

## Protocol

### Single op: `POST /tunnel`

```json
{"k":"auth","op":"connect","host":"example.com","port":443}
{"k":"auth","op":"connect_data","host":"example.com","port":443,"data":"base64"}
{"k":"auth","op":"data","sid":"uuid","data":"base64"}
{"k":"auth","op":"udp_open","host":"example.com","port":443,"data":"base64"}
{"k":"auth","op":"udp_data","sid":"uuid","data":"base64"}
{"k":"auth","op":"close","sid":"uuid"}
```

### Batch: `POST /tunnel/batch`

```json
{
  "k": "auth",
  "ops": [
    {"op":"connect_data","host":"example.com","port":443,"d":"base64"},
    {"op":"udp_data","sid":"uuid2","d":"base64"},
    {"op":"close","sid":"uuid3"}
  ]
}
→ {"r": [{...}, {...}, {...}]}
```

### Health check: `GET /health` or `GET /healthz` → `ok`

## Performance: account groups and batching

The mhrv-f client runs a pipelined batch multiplexer in full mode. Each Apps Script round-trip takes ~2s, so the client fires multiple batch requests concurrently. Concurrency is limited per Google account group, not per deployment ID.

Extra deployment IDs inside the same account group help with rotation and fallback, but they share that Google account's simultaneous-execution and daily `UrlFetchApp` quota. Add more `account_groups` when you need more real concurrency or quota.

The tunnel-node itself is stateless per-request (sessions are keyed by UUID), so it handles concurrent batches naturally. For best results, deploy one `CodeFull.gs` Web App per Google account group, add extra IDs in that group for fallback if desired, and configure each group with its own `auth_key`/`script_ids`.

UDP support is intentionally bounded: each SOCKS5 `UDP ASSOCIATE` is capped on the client side, datagrams above 9 KiB are dropped before the Apps Script hop, and tunnel-node queues evict oldest packets when the client cannot poll fast enough.
