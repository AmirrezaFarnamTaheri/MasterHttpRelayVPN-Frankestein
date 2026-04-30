//! Full-mode tunnel client with pipelined batch multiplexer.
//!
//! A central multiplexer collects pending data from ALL active sessions
//! and fires batch requests without waiting for the previous one to return.
//! Each Apps Script account has a per-account concurrency cap. We enforce
//! a per-account semaphore so one busy/full account can't starve the mux.

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex as StdMutex};
use std::time::{Duration, Instant};

// `AtomicU64` from `std::sync::atomic` requires hardware-backed 64-bit
// atomics, which 32-bit MIPS (`mipsel-unknown-linux-musl`) does not provide.
// Reuse `portable-atomic` (already a workspace dep for `domain_fronter`).
use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine;
use portable_atomic::AtomicU64;
use tokio::io::{AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::{mpsc, oneshot, Mutex, Semaphore};

use crate::domain_fronter::{BatchOp, DomainFronter, TunnelResponse};

/// Apps Script allows 30 concurrent executions per account.
const CONCURRENCY_PER_ACCOUNT: usize = 30;

/// Maximum total base64-encoded payload bytes in a single batch request.
/// Apps Script accepts up to 50 MB per fetch, but the tunnel-node must
/// parse and fan-out every op — keeping batches under ~4 MB avoids
/// hitting the 6-minute execution cap on the Apps Script side.
const MAX_BATCH_PAYLOAD_BYTES: usize = 4 * 1024 * 1024;

/// Maximum number of ops in a single batch. Prevents one mega-batch from
/// serializing too many sessions behind a single HTTP round-trip.
const MAX_BATCH_OPS: usize = 50;

// Full-mode batch timeout is read from `DomainFronter::batch_timeout()`,
// sourced from `Config::request_timeout_secs`.

/// If one full-mode deployment flakes out after we already drained data from
/// local sessions, retry the same batch once through another deployment before
/// surfacing the error to the sockets.
const BATCH_FAILOVER_ATTEMPTS: usize = 2;

/// Timeout for a session waiting for its batch reply. If the batch task
/// is slow (e.g. one op in the batch has a dead target on the tunnel-node
/// side), the session gives up and retries on the next tick rather than
/// blocking indefinitely.
const REPLY_TIMEOUT: Duration = Duration::from_secs(35);

/// How long we hold the client socket after CONNECT/SOCKS5 handshake,
/// waiting for first bytes (TLS ClientHello). Bundling with connect saves
/// one Apps Script round-trip per new flow when the tunnel-node supports it.
const CLIENT_FIRST_DATA_WAIT: Duration = Duration::from_millis(50);

/// Adaptive coalesce defaults: after each new op arrives, wait another step
/// for more ops. Resets on every arrival, up to max from the first op.
/// Overridable via config `coalesce_step_ms` / `coalesce_max_ms`.
const DEFAULT_COALESCE_STEP_MS: u64 = 40;
const DEFAULT_COALESCE_MAX_MS: u64 = 1000;

/// Structured error from tunnel-node / Apps Script for unknown ops.
const CODE_UNSUPPORTED_OP: &str = "UNSUPPORTED_OP";

/// Empty poll round-trip latency below which we conclude the tunnel-node is
/// *not* long-polling (fixed-sleep drain instead).
const NO_LONGPOLL_DETECT_THRESHOLD: Duration = Duration::from_millis(1500);

/// How long a deployment stays on the no-long-poll list after a
/// fast-empty observation. This lets upgraded/redeployed tunnel nodes recover
/// without restarting the client.
const NO_LONGPOLL_RECOVER_AFTER: Duration = Duration::from_secs(60);

/// Cache destinations that tunnel-node reported as structurally unreachable.
/// This avoids spending Apps Script batches on repeated OS/browser probes to
/// guaranteed-fail targets such as IPv6-only hosts from IPv4-only networks.
const NEGATIVE_DEST_CACHE_TTL: Duration = Duration::from_secs(30);
const NEGATIVE_DEST_CACHE_MAX: usize = 256;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AppsScriptHtmlErrorHint {
    StandardPlaceholder,
    PersianQuotaPage,
    WorkspaceLandingPage,
}

fn classify_apps_script_html_error(err: &str) -> Option<AppsScriptHtmlErrorHint> {
    let lower = err.to_ascii_lowercase();
    if lower.contains("workspace")
        || (lower.contains("presentations") && lower.contains("spreadsheets"))
    {
        return Some(AppsScriptHtmlErrorHint::WorkspaceLandingPage);
    }
    if lower.contains("lang=\"fa\"")
        || lower.contains("dir=\"rtl\"")
        || err.contains("\u{0633}\u{0647}\u{0645}\u{06cc}\u{0647}")
        || err.contains("\u{067e}\u{0647}\u{0646}\u{0627}\u{06cc} \u{0628}\u{0627}\u{0646}\u{062f}")
    {
        return Some(AppsScriptHtmlErrorHint::PersianQuotaPage);
    }
    if lower.contains("the script completed but did not return anything")
        || (lower.contains("no json in batch response") && lower.contains("<html"))
    {
        return Some(AppsScriptHtmlErrorHint::StandardPlaceholder);
    }
    None
}

fn apps_script_html_error_label(hint: AppsScriptHtmlErrorHint) -> &'static str {
    match hint {
        AppsScriptHtmlErrorHint::StandardPlaceholder => "placeholder/decoy body",
        AppsScriptHtmlErrorHint::PersianQuotaPage => "Persian-localized Apps Script quota page",
        AppsScriptHtmlErrorHint::WorkspaceLandingPage => {
            "Google Workspace landing page from a restricted deployment account"
        }
    }
}

/// Ports where the server speaks first — pre-read gains nothing.
fn is_server_speaks_first(port: u16) -> bool {
    matches!(port, 21 | 22 | 25 | 80 | 110 | 143 | 587)
}

// ---------------------------------------------------------------------------
// Multiplexer
// ---------------------------------------------------------------------------

#[derive(Debug)]
enum MuxMsg {
    Connect {
        host: String,
        port: u16,
        reply: oneshot::Sender<Result<TunnelResponse, String>>,
    },
    ConnectData {
        host: String,
        port: u16,
        data: Arc<Vec<u8>>,
        reply: oneshot::Sender<Result<BatchedReply, String>>,
    },
    Data {
        sid: String,
        data: Vec<u8>,
        reply: oneshot::Sender<Result<BatchedReply, String>>,
    },
    UdpOpen {
        host: String,
        port: u16,
        data: Vec<u8>,
        reply: oneshot::Sender<Result<BatchedReply, String>>,
    },
    UdpData {
        sid: String,
        data: Vec<u8>,
        reply: oneshot::Sender<Result<BatchedReply, String>>,
    },
    Close {
        sid: String,
    },
}

#[derive(Debug)]
struct BatchedReply {
    response: TunnelResponse,
    script_id: String,
}

pub struct TunnelMux {
    tx: mpsc::Sender<MuxMsg>,
    connect_data_unsupported: Arc<AtomicBool>,
    no_longpoll_deployments: StdMutex<HashMap<String, Instant>>,
    all_no_longpoll: Arc<AtomicBool>,
    num_scripts: usize,
    negative_dest_cache: Arc<Mutex<NegativeDestCache>>,
    preread_win: AtomicU64,
    preread_loss: AtomicU64,
    preread_skip_port: AtomicU64,
    preread_skip_unsupported: AtomicU64,
    preread_win_total_us: AtomicU64,
    preread_total_events: AtomicU64,
}

#[derive(Default)]
struct NegativeDestCache {
    map: HashMap<String, Instant>,
    order: VecDeque<String>,
}

impl NegativeDestCache {
    fn contains(&mut self, key: &str) -> bool {
        self.prune();
        self.map.contains_key(key)
    }

    fn insert(&mut self, key: String) {
        self.prune();
        if !self.map.contains_key(&key) {
            self.order.push_back(key.clone());
        }
        self.map.insert(key, Instant::now());
        while self.map.len() > NEGATIVE_DEST_CACHE_MAX {
            let Some(old) = self.order.pop_front() else {
                break;
            };
            self.map.remove(&old);
        }
    }

    fn prune(&mut self) {
        let now = Instant::now();
        while let Some(front) = self.order.front() {
            let expired = self
                .map
                .get(front)
                .map(|at| now.duration_since(*at) > NEGATIVE_DEST_CACHE_TTL)
                .unwrap_or(true);
            if !expired {
                break;
            }
            if let Some(old) = self.order.pop_front() {
                self.map.remove(&old);
            }
        }
    }
}

impl TunnelMux {
    pub fn start(
        fronter: Arc<DomainFronter>,
        coalesce_step_ms: u64,
        coalesce_max_ms: u64,
    ) -> Arc<Self> {
        let n_accounts = fronter.num_accounts();
        let unique_script_count = fronter
            .script_ids_by_account()
            .into_iter()
            .flatten()
            .collect::<HashSet<_>>()
            .len();
        tracing::info!(
            "tunnel mux: {} account(s), {} unique deployment(s), {} concurrent per account",
            n_accounts,
            unique_script_count,
            CONCURRENCY_PER_ACCOUNT
        );
        let step = if coalesce_step_ms > 0 {
            coalesce_step_ms
        } else {
            DEFAULT_COALESCE_STEP_MS
        };
        let max = if coalesce_max_ms > 0 {
            coalesce_max_ms
        } else {
            DEFAULT_COALESCE_MAX_MS
        };
        tracing::info!("batch coalesce: step={}ms max={}ms", step, max);
        let (tx, rx) = mpsc::channel(512);
        tokio::spawn(mux_loop(rx, fronter, step, max));
        Arc::new(Self {
            tx,
            connect_data_unsupported: Arc::new(AtomicBool::new(false)),
            no_longpoll_deployments: StdMutex::new(HashMap::new()),
            all_no_longpoll: Arc::new(AtomicBool::new(false)),
            num_scripts: unique_script_count,
            negative_dest_cache: Arc::new(Mutex::new(NegativeDestCache::default())),
            preread_win: AtomicU64::new(0),
            preread_loss: AtomicU64::new(0),
            preread_skip_port: AtomicU64::new(0),
            preread_skip_unsupported: AtomicU64::new(0),
            preread_win_total_us: AtomicU64::new(0),
            preread_total_events: AtomicU64::new(0),
        })
    }

    async fn send(&self, msg: MuxMsg) {
        let _ = self.tx.send(msg).await;
    }

    async fn negative_dest_cached(&self, host: &str, port: u16) -> bool {
        let key = dest_key(host, port);
        self.negative_dest_cache.lock().await.contains(&key)
    }

    async fn record_negative_dest_if_applicable(&self, host: &str, port: u16, err: &str) {
        if !is_negative_destination_error(err) {
            return;
        }
        let key = dest_key(host, port);
        self.negative_dest_cache.lock().await.insert(key.clone());
        tracing::warn!(
            "tunnel destination {} cached as unreachable for {:?}: {}",
            key,
            NEGATIVE_DEST_CACHE_TTL,
            err
        );
    }

    pub async fn udp_open(
        &self,
        host: &str,
        port: u16,
        data: Vec<u8>,
    ) -> Result<TunnelResponse, String> {
        let (reply_tx, reply_rx) = oneshot::channel();
        self.send(MuxMsg::UdpOpen {
            host: host.to_string(),
            port,
            data,
            reply: reply_tx,
        })
        .await;
        match reply_rx.await {
            Ok(Ok(r)) => Ok(r.response),
            Ok(Err(e)) => Err(e),
            Err(_) => Err("mux channel closed".into()),
        }
    }

    pub async fn udp_data(&self, sid: &str, data: Vec<u8>) -> Result<TunnelResponse, String> {
        let (reply_tx, reply_rx) = oneshot::channel();
        self.send(MuxMsg::UdpData {
            sid: sid.to_string(),
            data,
            reply: reply_tx,
        })
        .await;
        match reply_rx.await {
            Ok(Ok(r)) => Ok(r.response),
            Ok(Err(e)) => Err(e),
            Err(_) => Err("mux channel closed".into()),
        }
    }

    pub async fn close_session(&self, sid: &str) {
        self.send(MuxMsg::Close {
            sid: sid.to_string(),
        })
        .await;
    }

    fn all_servers_no_longpoll(&self) -> bool {
        if !self.all_no_longpoll.load(Ordering::Relaxed) {
            return false;
        }
        let now = Instant::now();
        let mut deps = match self.no_longpoll_deployments.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        deps.retain(|_, marked_at| now.duration_since(*marked_at) < NO_LONGPOLL_RECOVER_AFTER);
        let still_all = self.num_scripts > 0 && deps.len() >= self.num_scripts;
        if !still_all {
            self.all_no_longpoll.store(false, Ordering::Relaxed);
        }
        still_all
    }

    fn mark_server_no_longpoll(&self, script_id: &str) {
        if script_id.is_empty() {
            return;
        }
        let now = Instant::now();
        let mut deps = match self.no_longpoll_deployments.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        deps.retain(|_, marked_at| now.duration_since(*marked_at) < NO_LONGPOLL_RECOVER_AFTER);
        let was_new = deps.insert(script_id.to_string(), now).is_none();
        let all_no_longpoll = self.num_scripts > 0 && deps.len() >= self.num_scripts;
        self.all_no_longpoll
            .store(all_no_longpoll, Ordering::Relaxed);
        if was_new {
            tracing::warn!(
                "tunnel deployment {} returned an empty poll faster than {:?}; treating it as no-long-poll for {:?} ({}/{})",
                short_id(script_id),
                NO_LONGPOLL_DETECT_THRESHOLD,
                NO_LONGPOLL_RECOVER_AFTER,
                deps.len(),
                self.num_scripts
            );
        }
        if all_no_longpoll {
            tracing::warn!(
                "all tunnel deployments currently look non-long-polling; using skip-empty-when-idle to avoid quota waste"
            );
        }
    }

    fn connect_data_unsupported(&self) -> bool {
        self.connect_data_unsupported.load(Ordering::Relaxed)
    }

    fn mark_connect_data_unsupported(&self) {
        if !self.connect_data_unsupported.swap(true, Ordering::Relaxed) {
            tracing::warn!(
                "tunnel-node doesn't support connect_data; falling back to plain connect + data for all future sessions"
            );
        }
    }

    fn record_preread_win(&self, port: u16, elapsed: Duration) {
        self.preread_win.fetch_add(1, Ordering::Relaxed);
        self.preread_win_total_us
            .fetch_add(elapsed.as_micros() as u64, Ordering::Relaxed);
        tracing::debug!("preread win: port={} took={:?}", port, elapsed);
        self.maybe_log_preread_summary();
    }

    fn record_preread_loss(&self, port: u16) {
        self.preread_loss.fetch_add(1, Ordering::Relaxed);
        tracing::debug!(
            "preread loss: port={} (empty within {:?})",
            port,
            CLIENT_FIRST_DATA_WAIT
        );
        self.maybe_log_preread_summary();
    }

    fn record_preread_skip_port(&self, port: u16) {
        self.preread_skip_port.fetch_add(1, Ordering::Relaxed);
        tracing::debug!("preread skip: port={} (server-speaks-first)", port);
        self.maybe_log_preread_summary();
    }

    fn record_preread_skip_unsupported(&self, port: u16) {
        self.preread_skip_unsupported
            .fetch_add(1, Ordering::Relaxed);
        tracing::debug!("preread skip: port={} (connect_data unsupported)", port);
        self.maybe_log_preread_summary();
    }

    fn maybe_log_preread_summary(&self) {
        let new_count = self.preread_total_events.fetch_add(1, Ordering::Relaxed) + 1;
        if !new_count.is_multiple_of(100) {
            return;
        }
        let win = self.preread_win.load(Ordering::Relaxed);
        let loss = self.preread_loss.load(Ordering::Relaxed);
        let skip_port = self.preread_skip_port.load(Ordering::Relaxed);
        let skip_unsup = self.preread_skip_unsupported.load(Ordering::Relaxed);
        let total_us = self.preread_win_total_us.load(Ordering::Relaxed);
        let mean_us = total_us.checked_div(win).unwrap_or(0);
        tracing::info!(
            "connect_data preread: {} win / {} loss / {} skip(port) / {} skip(unsup), mean win time {}µs (ceiling {}µs)",
            win,
            loss,
            skip_port,
            skip_unsup,
            mean_us,
            CLIENT_FIRST_DATA_WAIT.as_micros(),
        );
    }
}

async fn mux_loop(
    mut rx: mpsc::Receiver<MuxMsg>,
    fronter: Arc<DomainFronter>,
    coalesce_step_ms: u64,
    coalesce_max_ms: u64,
) {
    let coalesce_step = Duration::from_millis(coalesce_step_ms.max(1));
    let coalesce_max = Duration::from_millis(coalesce_max_ms.max(coalesce_step_ms.max(1)));
    let sems: Arc<HashMap<usize, Arc<Semaphore>>> = Arc::new(
        (0..fronter.num_accounts())
            .map(|i| (i, Arc::new(Semaphore::new(CONCURRENCY_PER_ACCOUNT))))
            .collect(),
    );

    loop {
        let mut msgs = Vec::new();
        // Block until the first message arrives (with a short timeout to avoid
        // a permanently-sleeping task if the sender side dies silently).
        match tokio::time::timeout(Duration::from_millis(30), rx.recv()).await {
            Ok(Some(msg)) => msgs.push(msg),
            Ok(None) => break,
            Err(_) => continue,
        }

        // Adaptive coalescing: reset the short window every time another op
        // arrives, but never exceed the hard cap from the first op.
        let hard_deadline = tokio::time::Instant::now() + coalesce_max;
        let mut soft_deadline = tokio::time::Instant::now() + coalesce_step;
        loop {
            // Drain anything that's already queued without waiting.
            while let Ok(msg) = rx.try_recv() {
                msgs.push(msg);
                soft_deadline = tokio::time::Instant::now() + coalesce_step;
            }
            let now = tokio::time::Instant::now();
            let wait_until = soft_deadline.min(hard_deadline);
            if now >= wait_until {
                break;
            }
            match tokio::time::timeout(wait_until - now, rx.recv()).await {
                Ok(Some(msg)) => {
                    msgs.push(msg);
                    soft_deadline = tokio::time::Instant::now() + coalesce_step;
                }
                Ok(None) => return,
                Err(_) => break,
            }
        }

        let mut data_ops: Vec<BatchOp> = Vec::new();
        let mut data_replies: Vec<(usize, oneshot::Sender<Result<BatchedReply, String>>)> =
            Vec::new();
        let mut close_sids: Vec<String> = Vec::new();
        let mut batch_payload_bytes: usize = 0;

        for msg in msgs {
            match msg {
                MuxMsg::Connect { host, port, reply } => {
                    let f = fronter.clone();
                    tokio::spawn(async move {
                        let result = f
                            .tunnel_request("connect", Some(&host), Some(port), None, None)
                            .await;
                        match result {
                            Ok(resp) => {
                                let _ = reply.send(Ok(resp));
                            }
                            Err(e) => {
                                let _ = reply.send(Err(format!("{}", e)));
                            }
                        }
                    });
                }
                MuxMsg::ConnectData {
                    host,
                    port,
                    data,
                    reply,
                } => {
                    let encoded = Some(B64.encode(data.as_slice()));
                    let op_bytes = encoded.as_ref().map(|s| s.len()).unwrap_or(0);

                    if !data_ops.is_empty()
                        && (data_ops.len() >= MAX_BATCH_OPS
                            || batch_payload_bytes + op_bytes > MAX_BATCH_PAYLOAD_BYTES)
                    {
                        fire_batch(
                            &sems,
                            &fronter,
                            std::mem::take(&mut data_ops),
                            std::mem::take(&mut data_replies),
                        )
                        .await;
                        batch_payload_bytes = 0;
                    }

                    let idx = data_ops.len();
                    data_ops.push(BatchOp {
                        op: "connect_data".into(),
                        sid: None,
                        host: Some(host),
                        port: Some(port),
                        d: encoded,
                    });
                    data_replies.push((idx, reply));
                    batch_payload_bytes += op_bytes;
                }
                MuxMsg::Data { sid, data, reply } => {
                    let encoded = if data.is_empty() {
                        None
                    } else {
                        Some(B64.encode(&data))
                    };
                    let op_bytes = encoded.as_ref().map(|s| s.len()).unwrap_or(0);

                    if !data_ops.is_empty()
                        && (data_ops.len() >= MAX_BATCH_OPS
                            || batch_payload_bytes + op_bytes > MAX_BATCH_PAYLOAD_BYTES)
                    {
                        fire_batch(
                            &sems,
                            &fronter,
                            std::mem::take(&mut data_ops),
                            std::mem::take(&mut data_replies),
                        )
                        .await;
                        batch_payload_bytes = 0;
                    }

                    let idx = data_ops.len();
                    data_ops.push(BatchOp {
                        op: "data".into(),
                        sid: Some(sid),
                        host: None,
                        port: None,
                        d: encoded,
                    });
                    data_replies.push((idx, reply));
                    batch_payload_bytes += op_bytes;
                }
                MuxMsg::UdpOpen {
                    host,
                    port,
                    data,
                    reply,
                } => {
                    let encoded = if data.is_empty() {
                        None
                    } else {
                        Some(B64.encode(&data))
                    };
                    let op_bytes = encoded.as_ref().map(|s| s.len()).unwrap_or(0);

                    if !data_ops.is_empty()
                        && (data_ops.len() >= MAX_BATCH_OPS
                            || batch_payload_bytes + op_bytes > MAX_BATCH_PAYLOAD_BYTES)
                    {
                        fire_batch(
                            &sems,
                            &fronter,
                            std::mem::take(&mut data_ops),
                            std::mem::take(&mut data_replies),
                        )
                        .await;
                        batch_payload_bytes = 0;
                    }

                    let idx = data_ops.len();
                    data_ops.push(BatchOp {
                        op: "udp_open".into(),
                        sid: None,
                        host: Some(host),
                        port: Some(port),
                        d: encoded,
                    });
                    data_replies.push((idx, reply));
                    batch_payload_bytes += op_bytes;
                }
                MuxMsg::UdpData { sid, data, reply } => {
                    let encoded = if data.is_empty() {
                        None
                    } else {
                        Some(B64.encode(&data))
                    };
                    let op_bytes = encoded.as_ref().map(|s| s.len()).unwrap_or(0);

                    if !data_ops.is_empty()
                        && (data_ops.len() >= MAX_BATCH_OPS
                            || batch_payload_bytes + op_bytes > MAX_BATCH_PAYLOAD_BYTES)
                    {
                        fire_batch(
                            &sems,
                            &fronter,
                            std::mem::take(&mut data_ops),
                            std::mem::take(&mut data_replies),
                        )
                        .await;
                        batch_payload_bytes = 0;
                    }

                    let idx = data_ops.len();
                    data_ops.push(BatchOp {
                        op: "udp_data".into(),
                        sid: Some(sid),
                        host: None,
                        port: None,
                        d: encoded,
                    });
                    data_replies.push((idx, reply));
                    batch_payload_bytes += op_bytes;
                }
                MuxMsg::Close { sid } => {
                    close_sids.push(sid);
                }
            }
        }

        for sid in close_sids {
            data_ops.push(BatchOp {
                op: "close".into(),
                sid: Some(sid),
                host: None,
                port: None,
                d: None,
            });
        }

        if data_ops.is_empty() {
            continue;
        }

        fire_batch(&sems, &fronter, data_ops, data_replies).await;
    }
}

async fn fire_batch(
    sems: &Arc<HashMap<usize, Arc<Semaphore>>>,
    fronter: &Arc<DomainFronter>,
    data_ops: Vec<BatchOp>,
    data_replies: Vec<(usize, oneshot::Sender<Result<BatchedReply, String>>)>,
) {
    if fronter.num_scripts() > 1 {
        let f = fronter.clone();
        let sems = sems.clone();
        tokio::spawn(async move {
            let n_ops = data_ops.len();
            let mut last_err = "batch failed".to_string();
            let batch_timeout = f.batch_timeout();

            for attempt in 1..=BATCH_FAILOVER_ATTEMPTS {
                let (ai, auth_key, script_id) = f.next_script_target_for_tunnel();
                let sem = sems
                    .get(&ai)
                    .cloned()
                    .unwrap_or_else(|| Arc::new(Semaphore::new(CONCURRENCY_PER_ACCOUNT)));
                let permit = sem.acquire_owned().await.unwrap();
                let t0 = std::time::Instant::now();

                let result = tokio::time::timeout(
                    batch_timeout,
                    f.tunnel_batch_request_to(&auth_key, &script_id, &data_ops),
                )
                .await;
                drop(permit);

                tracing::info!(
                    "batch: {} ops -> {} (acct {}, attempt {}/{}), rtt={:?}",
                    n_ops,
                    &script_id[..script_id.len().min(8)],
                    ai,
                    attempt,
                    BATCH_FAILOVER_ATTEMPTS,
                    t0.elapsed()
                );

                match result {
                    Ok(Ok(batch_resp)) => {
                        for (idx, reply) in data_replies {
                            if let Some(resp) = batch_resp.r.get(idx) {
                                let _ = reply.send(Ok(BatchedReply {
                                    response: resp.clone(),
                                    script_id: script_id.clone(),
                                }));
                            } else {
                                let _ = reply.send(Err("missing response in batch".into()));
                            }
                        }
                        return;
                    }
                    Ok(Err(e)) => {
                        last_err = format!("{}", e);
                        tracing::warn!(
                            "batch failed on {} (attempt {}/{}): {}",
                            &script_id[..script_id.len().min(8)],
                            attempt,
                            BATCH_FAILOVER_ATTEMPTS,
                            last_err
                        );
                        f.mark_tunnel_script_unhealthy(&script_id, &last_err);
                    }
                    Err(_) => {
                        last_err = "batch timed out".into();
                        tracing::warn!(
                            "batch timed out after {:?} on {} ({} ops, attempt {}/{})",
                            batch_timeout,
                            &script_id[..script_id.len().min(8)],
                            n_ops,
                            attempt,
                            BATCH_FAILOVER_ATTEMPTS
                        );
                        f.mark_tunnel_script_unhealthy(&script_id, &last_err);
                    }
                }

                if attempt < BATCH_FAILOVER_ATTEMPTS {
                    tracing::warn!("retrying batch through an alternate tunnel deployment");
                }
            }

            for (_, reply) in data_replies {
                let _ = reply.send(Err(last_err.clone()));
            }
        });
        return;
    }

    let (ai, auth_key, script_id) = fronter.next_script_target_for_tunnel();
    let sem = sems
        .get(&ai)
        .cloned()
        .unwrap_or_else(|| Arc::new(Semaphore::new(CONCURRENCY_PER_ACCOUNT)));
    let permit = sem.acquire_owned().await.unwrap();
    let f = fronter.clone();

    tokio::spawn(async move {
        let _permit = permit;
        let t0 = std::time::Instant::now();
        let n_ops = data_ops.len();
        let batch_timeout = f.batch_timeout();

        let result = tokio::time::timeout(
            batch_timeout,
            f.tunnel_batch_request_to(&auth_key, &script_id, &data_ops),
        )
        .await;
        tracing::info!(
            "batch: {} ops → {} (acct {}), rtt={:?}",
            n_ops,
            &script_id[..script_id.len().min(8)],
            ai,
            t0.elapsed()
        );

        match result {
            Ok(Ok(batch_resp)) => {
                for (idx, reply) in data_replies {
                    if let Some(resp) = batch_resp.r.get(idx) {
                        let _ = reply.send(Ok(BatchedReply {
                            response: resp.clone(),
                            script_id: script_id.clone(),
                        }));
                    } else {
                        let _ = reply.send(Err("missing response in batch".into()));
                    }
                }
            }
            Ok(Err(e)) => {
                let err_msg = format!("{}", e);
                let sid_short = &script_id[..script_id.len().min(8)];
                if let Some(hint) = classify_apps_script_html_error(&err_msg) {
                    tracing::error!(
                        "batch failed (script {}): got Apps Script HTML instead of JSON ({}). \
                         Common causes: AUTH_KEY mismatch or stale Apps Script deployment, \
                         Code.gs vs CodeFull.gs mismatch for the selected mode, Apps Script \
                         quota/timeout pressure, ISP/body truncation, or a restricted Google \
                         account serving a Workspace landing page. Set DIAGNOSTIC_MODE=true \
                         in Code.gs/CodeFull.gs and redeploy as a new version; only an auth \
                         mismatch turns into explicit JSON unauthorized in diagnostic mode. \
                         Raw error: {}",
                        sid_short,
                        apps_script_html_error_label(hint),
                        err_msg
                    );
                } else {
                    tracing::warn!("batch failed (script {}): {}", sid_short, err_msg);
                }
                f.mark_tunnel_script_unhealthy(&script_id, &err_msg);
                for (_, reply) in data_replies {
                    let _ = reply.send(Err(err_msg.clone()));
                }
            }
            Err(_) => {
                tracing::warn!("batch timed out after {:?} ({} ops)", batch_timeout, n_ops);
                f.mark_tunnel_script_unhealthy(&script_id, "batch timed out");
                for (_, reply) in data_replies {
                    let _ = reply.send(Err("batch timed out".into()));
                }
            }
        }
    });
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

pub async fn tunnel_connection(
    mut sock: TcpStream,
    host: &str,
    port: u16,
    mux: &Arc<TunnelMux>,
) -> std::io::Result<()> {
    let initial_data = if mux.connect_data_unsupported() {
        mux.record_preread_skip_unsupported(port);
        None
    } else if is_server_speaks_first(port) {
        mux.record_preread_skip_port(port);
        None
    } else {
        let mut buf = vec![0u8; 65536];
        let t0 = Instant::now();
        match tokio::time::timeout(CLIENT_FIRST_DATA_WAIT, sock.read(&mut buf)).await {
            Ok(Ok(0)) => return Ok(()),
            Ok(Ok(n)) => {
                mux.record_preread_win(port, t0.elapsed());
                buf.truncate(n);
                Some(Arc::new(buf))
            }
            Ok(Err(e)) => return Err(e),
            Err(_) => {
                mux.record_preread_loss(port);
                None
            }
        }
    };

    let (sid, first_resp, pending_client_data) = match initial_data {
        Some(data) => match connect_with_initial_data(host, port, data.clone(), mux).await? {
            ConnectDataOutcome::Opened { sid, response } => (sid, Some(response), None),
            ConnectDataOutcome::Unsupported => {
                mux.mark_connect_data_unsupported();
                let sid = connect_plain(host, port, mux).await?;
                let bytes = Arc::try_unwrap(data).unwrap_or_else(|a| (*a).clone());
                (sid, None, Some(bytes))
            }
        },
        None => (connect_plain(host, port, mux).await?, None, None),
    };

    tracing::info!("tunnel session {} opened for {}:{}", sid, host, port);

    let result = async {
        if let Some(resp) = first_resp {
            match write_tunnel_response(&mut sock, &resp).await? {
                WriteOutcome::Wrote | WriteOutcome::NoData => {}
                WriteOutcome::BadBase64 => {
                    tracing::error!(
                        "tunnel session {}: bad base64 in connect_data response",
                        sid
                    );
                    return Ok(());
                }
            }
            if resp.eof.unwrap_or(false) {
                return Ok(());
            }
        }
        tunnel_loop(&mut sock, &sid, mux, pending_client_data).await
    }
    .await;

    mux.send(MuxMsg::Close { sid: sid.clone() }).await;
    tracing::info!("tunnel session {} closed for {}:{}", sid, host, port);
    result
}

enum ConnectDataOutcome {
    Opened {
        sid: String,
        response: TunnelResponse,
    },
    Unsupported,
}

fn dest_key(host: &str, port: u16) -> String {
    format!(
        "{}:{}",
        host.trim_end_matches('.').to_ascii_lowercase(),
        port
    )
}

fn short_id(script_id: &str) -> &str {
    &script_id[..script_id.len().min(8)]
}

fn is_negative_destination_error(err: &str) -> bool {
    let e = err.to_ascii_lowercase();
    e.contains("network is unreachable")
        || e.contains("no route to host")
        || e.contains("host unreachable")
        || e.contains("address not available")
        || e.contains("cannot assign requested address")
}

async fn connect_plain(host: &str, port: u16, mux: &Arc<TunnelMux>) -> std::io::Result<String> {
    if mux.negative_dest_cached(host, port).await {
        return Err(std::io::Error::other(format!(
            "cached unreachable destination: {}",
            dest_key(host, port)
        )));
    }

    let (reply_tx, reply_rx) = oneshot::channel();
    mux.send(MuxMsg::Connect {
        host: host.to_string(),
        port,
        reply: reply_tx,
    })
    .await;

    match reply_rx.await {
        Ok(Ok(resp)) => {
            if let Some(ref e) = resp.e {
                if resp.code.as_deref() == Some(CODE_UNSUPPORTED_OP) {
                    tracing::warn!("tunnel connect: unsupported op / node version skew: {}", e);
                } else {
                    tracing::error!("tunnel connect error for {}:{}: {}", host, port, e);
                }
                mux.record_negative_dest_if_applicable(host, port, e).await;
                return Err(std::io::Error::new(
                    std::io::ErrorKind::ConnectionRefused,
                    e.clone(),
                ));
            }
            resp.sid
                .ok_or_else(|| std::io::Error::other("tunnel connect: no session id"))
        }
        Ok(Err(e)) => {
            tracing::error!("tunnel connect error for {}:{}: {}", host, port, e);
            mux.record_negative_dest_if_applicable(host, port, &e).await;
            Err(std::io::Error::new(
                std::io::ErrorKind::ConnectionRefused,
                e,
            ))
        }
        Err(_) => Err(std::io::Error::other("mux channel closed")),
    }
}

async fn connect_with_initial_data(
    host: &str,
    port: u16,
    data: Arc<Vec<u8>>,
    mux: &Arc<TunnelMux>,
) -> std::io::Result<ConnectDataOutcome> {
    if mux.negative_dest_cached(host, port).await {
        return Err(std::io::Error::other(format!(
            "cached unreachable destination: {}",
            dest_key(host, port)
        )));
    }

    let (reply_tx, reply_rx) = oneshot::channel();
    mux.send(MuxMsg::ConnectData {
        host: host.to_string(),
        port,
        data,
        reply: reply_tx,
    })
    .await;

    let resp = match reply_rx.await {
        Ok(Ok(reply)) => reply.response,
        Ok(Err(e)) => {
            if is_connect_data_unsupported_error_str(&e) {
                tracing::debug!("connect_data unsupported for {}:{}: {}", host, port, e);
                return Ok(ConnectDataOutcome::Unsupported);
            }
            tracing::error!("tunnel connect_data error for {}:{}: {}", host, port, e);
            mux.record_negative_dest_if_applicable(host, port, &e).await;
            return Err(std::io::Error::new(
                std::io::ErrorKind::ConnectionRefused,
                e,
            ));
        }
        Err(_) => {
            return Err(std::io::Error::other("mux channel closed"));
        }
    };

    if is_connect_data_unsupported_response(&resp) {
        tracing::debug!(
            "connect_data unsupported for {}:{}: {:?}",
            host,
            port,
            resp.e
        );
        return Ok(ConnectDataOutcome::Unsupported);
    }

    if let Some(ref e) = resp.e {
        tracing::error!("tunnel connect_data error for {}:{}: {}", host, port, e);
        mux.record_negative_dest_if_applicable(host, port, e).await;
        return Err(std::io::Error::new(
            std::io::ErrorKind::ConnectionRefused,
            e.clone(),
        ));
    }

    let Some(sid) = resp.sid.clone() else {
        return Err(std::io::Error::other("tunnel connect_data: no session id"));
    };

    Ok(ConnectDataOutcome::Opened {
        sid,
        response: resp,
    })
}

fn is_connect_data_unsupported_response(resp: &TunnelResponse) -> bool {
    if resp.code.as_deref() == Some(CODE_UNSUPPORTED_OP) {
        return true;
    }
    resp.e
        .as_deref()
        .map(is_connect_data_unsupported_error_str)
        .unwrap_or(false)
}

fn is_connect_data_unsupported_error_str(e: &str) -> bool {
    let e = e.to_ascii_lowercase();
    (e.contains("unknown op") || e.contains("unknown tunnel op")) && e.contains("connect_data")
}

async fn tunnel_loop(
    sock: &mut TcpStream,
    sid: &str,
    mux: &Arc<TunnelMux>,
    mut pending_client_data: Option<Vec<u8>>,
) -> std::io::Result<()> {
    let (mut reader, mut writer) = sock.split();
    let mut buf = vec![0u8; 65536];
    let mut consecutive_empty = 0u32;

    loop {
        // Cadence depends on whether the tunnel-node is doing long-poll drains.
        // With long-poll, the server holds empty polls open and returns on push
        // or deadline; in fixed-sleep mode, hammering empty polls wastes
        // Apps Script quota, so we skip empty polls when sustained-idle.
        let no_longpoll_mode = mux.all_servers_no_longpoll();
        let client_data = if let Some(data) = pending_client_data.take() {
            Some(data)
        } else {
            let read_timeout = match (no_longpoll_mode, consecutive_empty) {
                (_, 0) => Duration::from_millis(20),
                (_, 1) => Duration::from_millis(80),
                (_, 2) => Duration::from_millis(200),
                (false, _) => Duration::from_millis(500),
                (true, _) => Duration::from_secs(30),
            };

            match tokio::time::timeout(read_timeout, reader.read(&mut buf)).await {
                Ok(Ok(0)) => break,
                Ok(Ok(n)) => {
                    consecutive_empty = 0;
                    Some(buf[..n].to_vec())
                }
                Ok(Err(_)) => break,
                Err(_) => None,
            }
        };

        // No-long-poll server skip: against a non-long-polling tunnel-node, an empty
        // poll is wasted work — preserve the pre-long-poll behavior of going
        // quiet after a few empties.
        if no_longpoll_mode && client_data.is_none() && consecutive_empty > 3 {
            continue;
        }

        let data = client_data.unwrap_or_default();
        let was_empty_poll = data.is_empty();

        let (reply_tx, reply_rx) = oneshot::channel();
        let send_at = Instant::now();
        mux.send(MuxMsg::Data {
            sid: sid.to_string(),
            data,
            reply: reply_tx,
        })
        .await;

        let reply = match tokio::time::timeout(REPLY_TIMEOUT, reply_rx).await {
            Ok(Ok(Ok(r))) => r,
            Ok(Ok(Err(e))) => {
                tracing::debug!("tunnel data error: {}", e);
                break;
            }
            Ok(Err(_)) => break,
            Err(_) => {
                tracing::warn!("sess {}: reply timeout, retrying", &sid[..sid.len().min(8)]);
                consecutive_empty = consecutive_empty.saturating_add(1);
                continue;
            }
        };
        let script_id = reply.script_id;
        let resp = reply.response;

        // No-long-poll server detection: an empty-in/empty-out round trip that
        // completes well under NO_LONGPOLL_DETECT_THRESHOLD is structurally
        // incompatible with long-poll. One observation flips a sticky flag.
        if !no_longpoll_mode && was_empty_poll {
            let reply_was_empty = resp.d.as_deref().map(str::is_empty).unwrap_or(true);
            if reply_was_empty && send_at.elapsed() < NO_LONGPOLL_DETECT_THRESHOLD {
                mux.mark_server_no_longpoll(&script_id);
            }
        }

        if let Some(ref e) = resp.e {
            if resp.code.as_deref() == Some(CODE_UNSUPPORTED_OP) {
                tracing::warn!("tunnel: unsupported op / node version skew ({})", e);
            } else {
                tracing::debug!("tunnel error: {}", e);
            }
            break;
        }

        let got_data = match write_tunnel_response(&mut writer, &resp).await? {
            WriteOutcome::Wrote => true,
            WriteOutcome::NoData => false,
            WriteOutcome::BadBase64 => break,
        };

        if resp.eof.unwrap_or(false) {
            break;
        }

        if got_data {
            consecutive_empty = 0;
        } else {
            consecutive_empty = consecutive_empty.saturating_add(1);
        }
    }

    Ok(())
}

enum WriteOutcome {
    Wrote,
    NoData,
    BadBase64,
}

async fn write_tunnel_response<W>(
    writer: &mut W,
    resp: &TunnelResponse,
) -> std::io::Result<WriteOutcome>
where
    W: AsyncWrite + Unpin,
{
    let Some(ref d) = resp.d else {
        return Ok(WriteOutcome::NoData);
    };
    if d.is_empty() {
        return Ok(WriteOutcome::NoData);
    }

    match B64.decode(d) {
        Ok(bytes) if !bytes.is_empty() => {
            writer.write_all(&bytes).await?;
            writer.flush().await?;
            Ok(WriteOutcome::Wrote)
        }
        Ok(_) => Ok(WriteOutcome::NoData),
        Err(e) => {
            tracing::error!("tunnel bad base64: {}", e);
            Ok(WriteOutcome::BadBase64)
        }
    }
}

pub fn decode_udp_packets(resp: &TunnelResponse) -> Result<Vec<Vec<u8>>, String> {
    let Some(pkts) = resp.pkts.as_ref() else {
        return Ok(Vec::new());
    };
    pkts.iter()
        .map(|pkt| {
            B64.decode(pkt)
                .map_err(|e| format!("bad UDP packet base64: {}", e))
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn resp_with(code: Option<&str>, e: Option<&str>) -> TunnelResponse {
        TunnelResponse {
            sid: None,
            d: None,
            pkts: None,
            eof: None,
            e: e.map(str::to_string),
            code: code.map(str::to_string),
        }
    }

    #[test]
    fn unsupported_detection_via_structured_code() {
        assert!(is_connect_data_unsupported_response(&resp_with(
            Some("UNSUPPORTED_OP"),
            None
        )));
        assert!(is_connect_data_unsupported_response(&resp_with(
            Some("UNSUPPORTED_OP"),
            Some("unknown op: connect_data"),
        )));
    }

    #[test]
    fn unsupported_detection_via_plain_tunnel_node_string() {
        assert!(is_connect_data_unsupported_response(&resp_with(
            None,
            Some("unknown op: connect_data"),
        )));
        assert!(is_connect_data_unsupported_response(&resp_with(
            None,
            Some("Unknown Op: CONNECT_DATA"),
        )));
    }

    #[test]
    fn unsupported_detection_via_plain_apps_script_string() {
        assert!(is_connect_data_unsupported_response(&resp_with(
            None,
            Some("unknown tunnel op: connect_data"),
        )));
    }

    #[test]
    fn unsupported_detection_rejects_unrelated_errors() {
        assert!(!is_connect_data_unsupported_response(&resp_with(
            None,
            Some("connect failed: refused"),
        )));
        assert!(!is_connect_data_unsupported_response(&resp_with(
            None,
            Some("bad base64")
        )));
        assert!(!is_connect_data_unsupported_response(&resp_with(
            None, None
        )));
        assert!(!is_connect_data_unsupported_response(&resp_with(
            None,
            Some("connect_data: bad port"),
        )));
    }

    #[test]
    fn server_speaks_first_covers_common_protocols() {
        for p in [21u16, 22, 25, 80, 110, 143, 587] {
            assert!(
                is_server_speaks_first(p),
                "port {} should be server-first",
                p
            );
        }
        for p in [443u16, 8443, 853, 993, 1234] {
            assert!(
                !is_server_speaks_first(p),
                "port {} should NOT be server-first",
                p
            );
        }
    }

    #[test]
    fn apps_script_html_error_classifier_distinguishes_known_bodies() {
        assert_eq!(
            classify_apps_script_html_error("bad response: no json in batch response: The script completed but did not return anything"),
            Some(AppsScriptHtmlErrorHint::StandardPlaceholder)
        );
        assert_eq!(
            classify_apps_script_html_error(
                "<html lang=\"fa\" dir=\"rtl\"><body>quota</body></html>"
            ),
            Some(AppsScriptHtmlErrorHint::PersianQuotaPage)
        );
        assert_eq!(
            classify_apps_script_html_error(
                "<html><body>Google Workspace presentations and spreadsheets</body></html>"
            ),
            Some(AppsScriptHtmlErrorHint::WorkspaceLandingPage)
        );
        assert_eq!(classify_apps_script_html_error("connect failed"), None);
    }

    fn mux_for_test() -> (Arc<TunnelMux>, mpsc::Receiver<MuxMsg>) {
        mux_for_test_with(1)
    }

    fn mux_for_test_with(num_scripts: usize) -> (Arc<TunnelMux>, mpsc::Receiver<MuxMsg>) {
        let (tx, rx) = mpsc::channel(16);
        let mux = Arc::new(TunnelMux {
            tx,
            connect_data_unsupported: Arc::new(AtomicBool::new(false)),
            no_longpoll_deployments: StdMutex::new(HashMap::new()),
            all_no_longpoll: Arc::new(AtomicBool::new(false)),
            num_scripts,
            negative_dest_cache: Arc::new(Mutex::new(NegativeDestCache::default())),
            preread_win: AtomicU64::new(0),
            preread_loss: AtomicU64::new(0),
            preread_skip_port: AtomicU64::new(0),
            preread_skip_unsupported: AtomicU64::new(0),
            preread_win_total_us: AtomicU64::new(0),
            preread_total_events: AtomicU64::new(0),
        });
        (mux, rx)
    }

    #[test]
    fn negative_destination_error_detects_structural_reachability_failures() {
        for e in [
            "connect failed: Network is unreachable",
            "connect failed: No route to host",
            "connect failed: host unreachable",
            "connect failed: Cannot assign requested address",
        ] {
            assert!(is_negative_destination_error(e), "{e}");
        }
        assert!(!is_negative_destination_error(
            "connect failed: connection refused"
        ));
        assert!(!is_negative_destination_error("batch timed out"));
        assert!(!is_negative_destination_error("unauthorized"));
    }

    #[test]
    fn destination_cache_key_normalizes_case_and_trailing_dot() {
        assert_eq!(dest_key("Example.COM.", 443), "example.com:443");
    }

    #[tokio::test]
    async fn tunnel_loop_replays_pending_client_data_before_reading_socket() {
        use tokio::net::TcpListener;

        let listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
        let addr = listener.local_addr().unwrap();
        let accept = tokio::spawn(async move { listener.accept().await.unwrap().0 });
        let _client = TcpStream::connect(addr).await.unwrap();
        let mut server_side = accept.await.unwrap();

        let (mux, mut rx) = mux_for_test();
        let pending = Some(b"CLIENTHELLO".to_vec());

        let loop_handle = tokio::spawn({
            let mux = mux.clone();
            async move { tunnel_loop(&mut server_side, "sid-under-test", &mux, pending).await }
        });

        let msg = tokio::time::timeout(Duration::from_secs(2), rx.recv())
            .await
            .expect("tunnel_loop did not send a message within 2s")
            .expect("mux channel closed unexpectedly");

        match msg {
            MuxMsg::Data { sid, data, reply } => {
                assert_eq!(sid, "sid-under-test");
                assert_eq!(&data[..], b"CLIENTHELLO");
                let _ = reply.send(Ok(BatchedReply {
                    response: TunnelResponse {
                        sid: Some("sid-under-test".into()),
                        d: None,
                        pkts: None,
                        eof: Some(true),
                        e: None,
                        code: None,
                    },
                    script_id: "script-A".into(),
                }));
            }
            other => panic!("unexpected first mux message: {:?}", other),
        }

        let _ = tokio::time::timeout(Duration::from_secs(2), loop_handle)
            .await
            .expect("tunnel_loop did not exit after eof");
    }

    #[test]
    fn unsupported_cache_is_sticky() {
        let (mux, _rx) = mux_for_test();
        assert!(!mux.connect_data_unsupported());
        mux.mark_connect_data_unsupported();
        assert!(mux.connect_data_unsupported());
        mux.mark_connect_data_unsupported();
        assert!(mux.connect_data_unsupported());
    }

    #[test]
    fn no_longpoll_state_is_per_deployment() {
        let (mux, _rx) = mux_for_test_with(2);
        mux.mark_server_no_longpoll("script-A");
        let deps = mux.no_longpoll_deployments.lock().unwrap();
        assert!(deps.contains_key("script-A"));
        assert!(!deps.contains_key("script-B"));
        assert!(!mux.all_no_longpoll.load(Ordering::Relaxed));
    }

    #[test]
    fn all_servers_no_longpoll_requires_every_deployment() {
        let (mux, _rx) = mux_for_test_with(2);
        assert!(!mux.all_servers_no_longpoll());
        mux.mark_server_no_longpoll("script-A");
        assert!(!mux.all_servers_no_longpoll());
        mux.mark_server_no_longpoll("script-B");
        assert!(mux.all_servers_no_longpoll());
        mux.mark_server_no_longpoll("script-A");
        assert!(mux.all_servers_no_longpoll());
    }

    #[test]
    fn no_longpoll_state_recovers_after_ttl() {
        let (mux, _rx) = mux_for_test_with(2);
        mux.mark_server_no_longpoll("script-A");
        {
            let mut deps = mux.no_longpoll_deployments.lock().unwrap();
            let stale = Instant::now()
                .checked_sub(NO_LONGPOLL_RECOVER_AFTER + Duration::from_secs(1))
                .expect("monotonic clock should be far enough along");
            deps.insert("script-A".to_string(), stale);
        }
        mux.mark_server_no_longpoll("script-B");
        let deps = mux.no_longpoll_deployments.lock().unwrap();
        assert!(!deps.contains_key("script-A"));
        assert!(deps.contains_key("script-B"));
        assert!(!mux.all_no_longpoll.load(Ordering::Relaxed));
    }

    #[test]
    fn all_servers_no_longpoll_self_corrects_when_entries_expire() {
        let (mux, _rx) = mux_for_test_with(2);
        mux.mark_server_no_longpoll("script-A");
        mux.mark_server_no_longpoll("script-B");
        assert!(mux.all_servers_no_longpoll());
        {
            let mut deps = mux.no_longpoll_deployments.lock().unwrap();
            let stale = Instant::now()
                .checked_sub(NO_LONGPOLL_RECOVER_AFTER + Duration::from_secs(1))
                .expect("monotonic clock should be far enough along");
            for marked_at in deps.values_mut() {
                *marked_at = stale;
            }
        }
        assert!(!mux.all_servers_no_longpoll());
        assert!(!mux.all_no_longpoll.load(Ordering::Relaxed));
    }

    #[test]
    fn preread_counters_track_each_outcome() {
        let (mux, _rx) = mux_for_test();

        mux.record_preread_win(443, Duration::from_micros(3_500));
        mux.record_preread_win(443, Duration::from_micros(1_500));
        mux.record_preread_loss(443);
        mux.record_preread_skip_port(80);
        mux.record_preread_skip_unsupported(443);

        assert_eq!(mux.preread_win.load(Ordering::Relaxed), 2);
        assert_eq!(mux.preread_loss.load(Ordering::Relaxed), 1);
        assert_eq!(mux.preread_skip_port.load(Ordering::Relaxed), 1);
        assert_eq!(mux.preread_skip_unsupported.load(Ordering::Relaxed), 1);
        assert_eq!(mux.preread_win_total_us.load(Ordering::Relaxed), 5_000);
        assert_eq!(mux.preread_total_events.load(Ordering::Relaxed), 5);
    }
}
