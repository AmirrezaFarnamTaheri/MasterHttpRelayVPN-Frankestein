//! Native client for the bundled serverless Edge JSON relay.
//!
//! Protocol:
//! request  `{k,m,u,h,b,ct,r}`
//! response `{s,h,b}` or `{e}`

use std::sync::Arc;
use std::time::Duration;

use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine;
use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use rustls::{ClientConfig, DigitallySignedStruct, SignatureScheme};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::{oneshot, Mutex};
use tokio::time::{sleep, timeout};
use tokio_rustls::TlsConnector;
use url::Url;

use crate::config::Config;
use crate::domain_fronter::{error_response, filter_forwarded_headers};

#[derive(Debug, thiserror::Error)]
pub enum VercelRelayError {
    #[error("invalid url: {0}")]
    InvalidUrl(String),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("tls: {0}")]
    Tls(#[from] rustls::Error),
    #[error("invalid dns name: {0}")]
    Dns(#[from] rustls::pki_types::InvalidDnsNameError),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("timeout")]
    Timeout,
    #[error("bad response: {0}")]
    BadResponse(String),
    #[error("relay error: {0}")]
    Relay(String),
}

#[derive(Clone)]
pub struct VercelEdgeRelay {
    endpoint: Url,
    host: String,
    port: u16,
    host_header: String,
    auth_key: String,
    verify_tls: bool,
    request_timeout: Duration,
    max_body_bytes: usize,
    batcher: Option<Arc<BatchState>>,
}

#[derive(Serialize, Clone, Debug)]
struct RelayItem {
    m: String,
    u: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    h: Option<serde_json::Map<String, Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    b: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ct: Option<String>,
    r: bool,
}

#[derive(Serialize)]
struct RelayRequest<'a> {
    k: &'a str,
    m: &'a str,
    u: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    h: Option<serde_json::Map<String, Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    b: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ct: Option<&'a str>,
    r: bool,
}

#[derive(Serialize)]
struct BatchRelayRequest<'a> {
    k: &'a str,
    q: &'a [RelayItem],
}

#[derive(Deserialize, Default)]
struct RelayResponse {
    #[serde(default)]
    s: Option<u16>,
    #[serde(default)]
    h: Option<serde_json::Map<String, Value>>,
    #[serde(default)]
    b: Option<String>,
    #[serde(default)]
    e: Option<String>,
}

#[derive(Deserialize, Default)]
struct BatchRelayResponse {
    #[serde(default)]
    q: Option<Vec<RelayResponse>>,
    #[serde(default)]
    e: Option<String>,
}

struct BatchState {
    inner: Mutex<BatchInner>,
}

#[derive(Default)]
struct BatchInner {
    pending: Vec<PendingRelay>,
    timer_scheduled: bool,
    disabled: bool,
}

struct PendingRelay {
    item: RelayItem,
    tx: oneshot::Sender<Result<Vec<u8>, VercelRelayError>>,
}

const VERCEL_BATCH_MAX: usize = 50;
const VERCEL_BATCH_MICRO_WINDOW: Duration = Duration::from_millis(5);
const VERCEL_BATCH_MACRO_WINDOW: Duration = Duration::from_millis(50);

impl VercelEdgeRelay {
    pub fn new(config: &Config) -> Result<Self, VercelRelayError> {
        let mut endpoint = Url::parse(config.vercel.base_url.trim())
            .map_err(|e| VercelRelayError::InvalidUrl(e.to_string()))?;
        if !matches!(endpoint.scheme(), "https" | "http") {
            return Err(VercelRelayError::InvalidUrl(
                "base_url must be http:// or https://".into(),
            ));
        }

        let host = endpoint
            .host_str()
            .ok_or_else(|| VercelRelayError::InvalidUrl("base_url missing host".into()))?
            .to_string();
        let port = endpoint.port_or_known_default().ok_or_else(|| {
            VercelRelayError::InvalidUrl("base_url missing port/default port".into())
        })?;
        let host_header = match endpoint.port() {
            Some(p)
                if !(endpoint.scheme() == "https" && p == 443
                    || endpoint.scheme() == "http" && p == 80) =>
            {
                format!("{}:{}", host, p)
            }
            _ => host.clone(),
        };

        let relay_path = config.vercel.relay_path.trim();
        endpoint.set_path(relay_path);
        endpoint.set_query(None);

        Ok(Self {
            endpoint,
            host,
            port,
            host_header,
            auth_key: config.vercel.auth_key.trim().to_string(),
            verify_tls: config.vercel.verify_tls,
            request_timeout: Duration::from_secs(config.effective_relay_request_timeout_secs()),
            max_body_bytes: config.vercel.max_body_bytes.max(1024),
            batcher: config.vercel.enable_batching.then(|| {
                Arc::new(BatchState {
                    inner: Mutex::new(BatchInner::default()),
                })
            }),
        })
    }

    pub async fn relay(
        &self,
        method: &str,
        url: &str,
        headers: &[(String, String)],
        body: &[u8],
    ) -> Vec<u8> {
        match self.relay_inner(method, url, headers, body).await {
            Ok(bytes) => bytes,
            Err(e) => {
                tracing::warn!("serverless JSON relay failed: {}", e);
                let status = match e {
                    VercelRelayError::Timeout => 504,
                    VercelRelayError::Relay(ref msg)
                        if msg.to_ascii_lowercase().contains("unauthorized") =>
                    {
                        502
                    }
                    _ => 502,
                };
                error_response(status, &format!("Serverless JSON relay failed: {e}"))
            }
        }
    }

    async fn relay_inner(
        &self,
        method: &str,
        url: &str,
        headers: &[(String, String)],
        body: &[u8],
    ) -> Result<Vec<u8>, VercelRelayError> {
        if body.len() > self.max_body_bytes {
            return Ok(error_response(
                413,
                &format!(
                    "Request body is {} bytes, above vercel.max_body_bytes={}",
                    body.len(),
                    self.max_body_bytes
                ),
            ));
        }
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err(VercelRelayError::InvalidUrl(format!(
                "upstream URL must be absolute: {url}"
            )));
        }

        let item = self.build_item(method, url, headers, body)?;
        if self.batcher.is_some() {
            self.submit_batch(item).await
        } else {
            self.post_single_item(&item).await
        }
    }

    async fn post_payload(&self, payload: &[u8]) -> Result<Vec<u8>, VercelRelayError> {
        let fut = async {
            if self.endpoint.scheme() == "https" {
                let tcp = TcpStream::connect((self.host.as_str(), self.port)).await?;
                let _ = tcp.set_nodelay(true);
                let connector = self.tls_connector();
                let name = ServerName::try_from(self.host.clone())?;
                let tls = connector.connect(name, tcp).await?;
                self.post_over_stream(tls, payload).await
            } else {
                let tcp = TcpStream::connect((self.host.as_str(), self.port)).await?;
                let _ = tcp.set_nodelay(true);
                self.post_over_stream(tcp, payload).await
            }
        };
        timeout(self.request_timeout, fut)
            .await
            .map_err(|_| VercelRelayError::Timeout)?
    }

    fn build_item(
        &self,
        method: &str,
        url: &str,
        headers: &[(String, String)],
        body: &[u8],
    ) -> Result<RelayItem, VercelRelayError> {
        let forwarded = filter_forwarded_headers(headers);
        let mut hmap = serde_json::Map::new();
        for (k, v) in forwarded {
            if k.eq_ignore_ascii_case("accept-encoding") {
                hmap.insert(k, Value::String(strip_brotli(&v)));
            } else {
                hmap.insert(k, Value::String(v));
            }
        }
        let hmap = if hmap.is_empty() { None } else { Some(hmap) };
        let b_encoded = if body.is_empty() {
            None
        } else {
            Some(B64.encode(body))
        };
        let ct = find_header(headers, "content-type").map(ToOwned::to_owned);
        Ok(RelayItem {
            m: method.to_string(),
            u: url.to_string(),
            h: hmap,
            b: b_encoded,
            ct,
            r: true,
        })
    }

    async fn post_single_item(&self, item: &RelayItem) -> Result<Vec<u8>, VercelRelayError> {
        let req = RelayRequest {
            k: &self.auth_key,
            m: &item.m,
            u: &item.u,
            h: item.h.clone(),
            b: item.b.clone(),
            ct: item.ct.as_deref(),
            r: item.r,
        };
        let payload = serde_json::to_vec(&req)?;
        let body = self.post_payload(&payload).await?;
        parse_relay_json(&body)
    }

    async fn post_batch_items(
        &self,
        items: &[RelayItem],
    ) -> Result<Vec<Vec<u8>>, VercelRelayError> {
        let req = BatchRelayRequest {
            k: &self.auth_key,
            q: items,
        };
        let payload = serde_json::to_vec(&req)?;
        let body = self.post_payload(&payload).await?;
        parse_batch_relay_json(&body, items.len())
    }

    async fn submit_batch(&self, item: RelayItem) -> Result<Vec<u8>, VercelRelayError> {
        let Some(batcher) = &self.batcher else {
            return self.post_single_item(&item).await;
        };
        let (tx, rx) = oneshot::channel();
        let mut maybe_flush = None;
        let mut schedule_timer = false;

        {
            let mut inner = batcher.inner.lock().await;
            if inner.disabled {
                drop(inner);
                return self.post_single_item(&item).await;
            }
            inner.pending.push(PendingRelay { item, tx });
            if inner.pending.len() >= VERCEL_BATCH_MAX {
                maybe_flush = Some(std::mem::take(&mut inner.pending));
                inner.timer_scheduled = false;
            } else if !inner.timer_scheduled {
                inner.timer_scheduled = true;
                schedule_timer = true;
            }
        }

        if let Some(batch) = maybe_flush {
            let this = self.clone();
            tokio::spawn(async move {
                this.flush_batch(batch).await;
            });
        } else if schedule_timer {
            let this = self.clone();
            tokio::spawn(async move {
                this.batch_timer().await;
            });
        }

        match rx.await {
            Ok(result) => result,
            Err(_) => Err(VercelRelayError::BadResponse(
                "serverless JSON batch worker stopped before replying".into(),
            )),
        }
    }

    async fn batch_timer(self) {
        sleep(VERCEL_BATCH_MICRO_WINDOW).await;
        let Some(batcher) = &self.batcher else {
            return;
        };

        let mut batch = {
            let mut inner = batcher.inner.lock().await;
            if inner.pending.len() <= 1 {
                inner.timer_scheduled = false;
                std::mem::take(&mut inner.pending)
            } else {
                Vec::new()
            }
        };
        if batch.is_empty() {
            sleep(VERCEL_BATCH_MACRO_WINDOW - VERCEL_BATCH_MICRO_WINDOW).await;
            let mut inner = batcher.inner.lock().await;
            inner.timer_scheduled = false;
            batch = std::mem::take(&mut inner.pending);
        }

        if !batch.is_empty() {
            self.flush_batch(batch).await;
        }
    }

    async fn flush_batch(&self, batch: Vec<PendingRelay>) {
        if batch.len() == 1 {
            let PendingRelay { item, tx } = batch.into_iter().next().expect("one pending item");
            let _ = tx.send(self.post_single_item(&item).await);
            return;
        }

        let items: Vec<RelayItem> = batch.iter().map(|p| p.item.clone()).collect();
        match self.post_batch_items(&items).await {
            Ok(results) => {
                for (pending, result) in batch.into_iter().zip(results) {
                    let _ = pending.tx.send(Ok(result));
                }
            }
            Err(err) => {
                tracing::warn!(
                    "serverless JSON batch relay failed; disabling batching and falling back to singles: {}",
                    err
                );
                if let Some(batcher) = &self.batcher {
                    let mut inner = batcher.inner.lock().await;
                    inner.disabled = true;
                }
                for pending in batch {
                    let result = self.post_single_item(&pending.item).await;
                    let _ = pending.tx.send(result);
                }
            }
        }
    }

    async fn post_over_stream<S>(
        &self,
        mut stream: S,
        payload: &[u8],
    ) -> Result<Vec<u8>, VercelRelayError>
    where
        S: AsyncRead + AsyncWrite + Unpin,
    {
        let path = self.path_and_query();
        let req_head = format!(
            "POST {path} HTTP/1.1\r\n\
             Host: {host}\r\n\
             Content-Type: application/json\r\n\
             Accept: application/json\r\n\
             Content-Length: {len}\r\n\
             Connection: close\r\n\
             \r\n",
            path = path,
            host = self.host_header,
            len = payload.len(),
        );
        stream.write_all(req_head.as_bytes()).await?;
        stream.write_all(payload).await?;
        stream.flush().await?;

        let (status, headers, body) = read_http_response(&mut stream).await?;
        let content_type = header_get(&headers, "content-type");
        crate::response_quality::log_hint(
            "serverless JSON relay response",
            status,
            content_type,
            &body,
        );

        if status == 401 {
            return Err(VercelRelayError::Relay(
                "unauthorized (check vercel.auth_key and relay AUTH_KEY)".into(),
            ));
        }
        if status != 200 {
            let preview = String::from_utf8_lossy(&body[..body.len().min(200)]);
            return Err(VercelRelayError::BadResponse(format!(
                "Serverless JSON endpoint returned HTTP {}: {}",
                status, preview
            )));
        }
        Ok(body)
    }

    fn path_and_query(&self) -> String {
        match self.endpoint.query() {
            Some(q) => format!("{}?{}", self.endpoint.path(), q),
            None => self.endpoint.path().to_string(),
        }
    }

    fn tls_connector(&self) -> TlsConnector {
        let config = if self.verify_tls {
            let mut roots = rustls::RootCertStore::empty();
            roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
            ClientConfig::builder()
                .with_root_certificates(roots)
                .with_no_client_auth()
        } else {
            ClientConfig::builder()
                .dangerous()
                .with_custom_certificate_verifier(std::sync::Arc::new(NoVerify))
                .with_no_client_auth()
        };
        TlsConnector::from(std::sync::Arc::new(config))
    }
}

fn parse_relay_json(body: &[u8]) -> Result<Vec<u8>, VercelRelayError> {
    let text = std::str::from_utf8(body)
        .map_err(|_| VercelRelayError::BadResponse("non-utf8 json".into()))?
        .trim();
    if text.is_empty() {
        return Err(VercelRelayError::BadResponse("empty relay body".into()));
    }

    let data: RelayResponse = match serde_json::from_str(text) {
        Ok(v) => v,
        Err(e) => {
            crate::response_quality::log_hint("serverless JSON parse", 200, None, body);
            return Err(VercelRelayError::BadResponse(format!(
                "Serverless JSON endpoint returned non-JSON; check platform protection/auth/routing. json error: {e}"
            )));
        }
    };
    parse_relay_response(data)
}

fn parse_batch_relay_json(
    body: &[u8],
    expected_len: usize,
) -> Result<Vec<Vec<u8>>, VercelRelayError> {
    let text = std::str::from_utf8(body)
        .map_err(|_| VercelRelayError::BadResponse("non-utf8 batch json".into()))?
        .trim();
    if text.is_empty() {
        return Err(VercelRelayError::BadResponse(
            "empty batch relay body".into(),
        ));
    }

    let data: BatchRelayResponse = match serde_json::from_str(text) {
        Ok(v) => v,
        Err(e) => {
            crate::response_quality::log_hint("serverless JSON batch parse", 200, None, body);
            return Err(VercelRelayError::BadResponse(format!(
                "Serverless JSON endpoint returned non-JSON batch response; check relay version/protection/routing. json error: {e}"
            )));
        }
    };
    if let Some(e) = data.e {
        return Err(VercelRelayError::Relay(e));
    }
    let items = data
        .q
        .ok_or_else(|| VercelRelayError::BadResponse("batch response missing q".into()))?;
    if items.len() != expected_len {
        return Err(VercelRelayError::BadResponse(format!(
            "batch response size mismatch: got {}, expected {}",
            items.len(),
            expected_len
        )));
    }

    items.into_iter().map(parse_relay_response).collect()
}

fn parse_relay_response(data: RelayResponse) -> Result<Vec<u8>, VercelRelayError> {
    if let Some(e) = data.e {
        return Err(VercelRelayError::Relay(e));
    }
    let status = data.s.unwrap_or(200);
    let resp_body = match data.b {
        Some(b) => B64
            .decode(b)
            .map_err(|e| VercelRelayError::BadResponse(format!("bad relay body base64: {e}")))?,
        None => Vec::new(),
    };

    let mut out = Vec::with_capacity(resp_body.len() + 256);
    out.extend_from_slice(format!("HTTP/1.1 {} {}\r\n", status, status_text(status)).as_bytes());
    const SKIP: &[&str] = &[
        "transfer-encoding",
        "connection",
        "keep-alive",
        "content-length",
        "content-encoding",
    ];
    if let Some(hmap) = data.h {
        for (k, v) in hmap {
            let lk = k.to_ascii_lowercase();
            if SKIP.contains(&lk.as_str()) {
                continue;
            }
            match v {
                Value::String(s) => push_header(&mut out, &k, &s),
                Value::Array(arr) => {
                    for item in arr {
                        if let Some(s) = item.as_str() {
                            push_header(&mut out, &k, s);
                        }
                    }
                }
                other => push_header(&mut out, &k, &other.to_string()),
            }
        }
    }
    out.extend_from_slice(format!("Content-Length: {}\r\n\r\n", resp_body.len()).as_bytes());
    out.extend_from_slice(&resp_body);
    Ok(out)
}

fn push_header(out: &mut Vec<u8>, key: &str, value: &str) {
    if key.contains('\r') || key.contains('\n') || value.contains('\r') || value.contains('\n') {
        return;
    }
    out.extend_from_slice(key.as_bytes());
    out.extend_from_slice(b": ");
    out.extend_from_slice(value.as_bytes());
    out.extend_from_slice(b"\r\n");
}

fn find_header<'a>(headers: &'a [(String, String)], name: &str) -> Option<&'a str> {
    headers
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case(name))
        .map(|(_, v)| v.as_str())
}

fn strip_brotli(v: &str) -> String {
    let parts: Vec<&str> = v
        .split(',')
        .map(str::trim)
        .filter(|p| !p.eq_ignore_ascii_case("br") && !p.is_empty())
        .collect();
    if parts.is_empty() {
        "gzip, deflate".into()
    } else {
        parts.join(", ")
    }
}

async fn read_http_response<S>(
    stream: &mut S,
) -> Result<(u16, Vec<(String, String)>, Vec<u8>), VercelRelayError>
where
    S: AsyncRead + Unpin,
{
    let mut buf = Vec::with_capacity(8192);
    let mut tmp = [0u8; 8192];
    let header_end = loop {
        if let Some(p) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
            break p;
        }
        let n = stream.read(&mut tmp).await?;
        if n == 0 {
            return Err(VercelRelayError::BadResponse(
                "connection closed before response headers".into(),
            ));
        }
        buf.extend_from_slice(&tmp[..n]);
        if buf.len() > 1024 * 1024 {
            return Err(VercelRelayError::BadResponse(
                "response headers too large".into(),
            ));
        }
    };

    let head = String::from_utf8_lossy(&buf[..header_end]);
    let mut lines = head.split("\r\n");
    let status_line = lines.next().unwrap_or("");
    let status = parse_status(status_line)?;
    let mut headers = Vec::new();
    for line in lines {
        if let Some((k, v)) = line.split_once(':') {
            headers.push((k.trim().to_string(), v.trim().to_string()));
        }
    }

    let mut body = buf[header_end + 4..].to_vec();
    let content_length = header_get(&headers, "content-length").and_then(|v| v.parse().ok());
    let is_chunked = header_get(&headers, "transfer-encoding")
        .map(|v| v.to_ascii_lowercase().contains("chunked"))
        .unwrap_or(false);
    if is_chunked {
        body = read_chunked(stream, body).await?;
    } else if let Some(cl) = content_length {
        while body.len() < cl {
            let need = cl - body.len();
            let take = tmp.len().min(need);
            let n = stream.read(&mut tmp[..take]).await?;
            if n == 0 {
                return Err(VercelRelayError::BadResponse(
                    "connection closed before full response body".into(),
                ));
            }
            body.extend_from_slice(&tmp[..n]);
        }
        body.truncate(cl);
    } else {
        loop {
            let n = stream.read(&mut tmp).await?;
            if n == 0 {
                break;
            }
            body.extend_from_slice(&tmp[..n]);
        }
    }

    Ok((status, headers, body))
}

fn parse_status(line: &str) -> Result<u16, VercelRelayError> {
    let mut parts = line.split_whitespace();
    let _http = parts.next();
    let code = parts
        .next()
        .ok_or_else(|| VercelRelayError::BadResponse(format!("bad status line: {line}")))?;
    code.parse::<u16>()
        .map_err(|_| VercelRelayError::BadResponse(format!("bad status code: {code}")))
}

fn header_get<'a>(headers: &'a [(String, String)], name: &str) -> Option<&'a str> {
    headers
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case(name))
        .map(|(_, v)| v.as_str())
}

async fn read_chunked<S>(stream: &mut S, mut buf: Vec<u8>) -> Result<Vec<u8>, VercelRelayError>
where
    S: AsyncRead + Unpin,
{
    let mut out = Vec::new();
    let mut tmp = [0u8; 8192];
    loop {
        let line = read_crlf_line(stream, &mut buf, &mut tmp).await?;
        if line.is_empty() {
            continue;
        }
        let line_str = std::str::from_utf8(&line)
            .map_err(|_| VercelRelayError::BadResponse("bad chunk size utf8".into()))?
            .trim();
        let size = usize::from_str_radix(line_str.split(';').next().unwrap_or(""), 16)
            .map_err(|_| VercelRelayError::BadResponse(format!("bad chunk size '{line_str}'")))?;
        if size == 0 {
            loop {
                let trailer = read_crlf_line(stream, &mut buf, &mut tmp).await?;
                if trailer.is_empty() {
                    return Ok(out);
                }
            }
        }
        while buf.len() < size + 2 {
            let n = stream.read(&mut tmp).await?;
            if n == 0 {
                return Err(VercelRelayError::BadResponse(
                    "connection closed mid-chunked response".into(),
                ));
            }
            buf.extend_from_slice(&tmp[..n]);
        }
        if &buf[size..size + 2] != b"\r\n" {
            return Err(VercelRelayError::BadResponse(
                "chunk missing trailing CRLF".into(),
            ));
        }
        out.extend_from_slice(&buf[..size]);
        buf.drain(..size + 2);
    }
}

async fn read_crlf_line<S>(
    stream: &mut S,
    buf: &mut Vec<u8>,
    tmp: &mut [u8],
) -> Result<Vec<u8>, VercelRelayError>
where
    S: AsyncRead + Unpin,
{
    loop {
        if let Some(idx) = buf.windows(2).position(|w| w == b"\r\n") {
            let line = buf[..idx].to_vec();
            buf.drain(..idx + 2);
            return Ok(line);
        }
        let n = stream.read(tmp).await?;
        if n == 0 {
            return Err(VercelRelayError::BadResponse(
                "EOF in chunked response".into(),
            ));
        }
        buf.extend_from_slice(&tmp[..n]);
    }
}

fn status_text(status: u16) -> &'static str {
    match status {
        200 => "OK",
        201 => "Created",
        202 => "Accepted",
        204 => "No Content",
        206 => "Partial Content",
        301 => "Moved Permanently",
        302 => "Found",
        304 => "Not Modified",
        307 => "Temporary Redirect",
        308 => "Permanent Redirect",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        429 => "Too Many Requests",
        500 => "Internal Server Error",
        502 => "Bad Gateway",
        504 => "Gateway Timeout",
        _ => "OK",
    }
}

#[derive(Debug)]
struct NoVerify;

impl ServerCertVerifier for NoVerify {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: UnixTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        vec![
            SignatureScheme::RSA_PKCS1_SHA256,
            SignatureScheme::RSA_PKCS1_SHA384,
            SignatureScheme::RSA_PKCS1_SHA512,
            SignatureScheme::ECDSA_NISTP256_SHA256,
            SignatureScheme::ECDSA_NISTP384_SHA384,
            SignatureScheme::RSA_PSS_SHA256,
            SignatureScheme::RSA_PSS_SHA384,
            SignatureScheme::RSA_PSS_SHA512,
            SignatureScheme::ED25519,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_relay_json() {
        let raw =
            parse_relay_json(br#"{"s":200,"h":{"Content-Type":"text/plain"},"b":"T0s="}"#).unwrap();
        let s = String::from_utf8_lossy(&raw);
        assert!(s.starts_with("HTTP/1.1 200 OK\r\n"));
        assert!(s.contains("Content-Type: text/plain\r\n"));
        assert!(s.ends_with("OK"));
    }

    #[test]
    fn parses_batch_relay_json() {
        let raw = parse_batch_relay_json(
            br#"{"q":[{"s":200,"h":{"Content-Type":"text/plain"},"b":"T25l"},{"s":404,"h":{},"b":"Tm8="}]}"#,
            2,
        )
        .unwrap();
        assert_eq!(raw.len(), 2);
        assert!(String::from_utf8_lossy(&raw[0]).contains("HTTP/1.1 200 OK"));
        assert!(String::from_utf8_lossy(&raw[1]).contains("HTTP/1.1 404 Not Found"));
    }

    #[test]
    fn strips_brotli() {
        assert_eq!(strip_brotli("gzip, deflate, br"), "gzip, deflate");
        assert_eq!(strip_brotli("br"), "gzip, deflate");
    }
}
