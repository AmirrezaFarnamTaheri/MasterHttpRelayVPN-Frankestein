/**
 * DomainFront Relay + Full Tunnel — Google Apps Script
 *
 * FOUR modes:
 *   1. Single relay:  POST { k, m, u, h, b, ct, r }           → { s, h, b }
 *   2. Batch relay:   POST { k, q: [{m,u,h,b,ct,r}, ...] }    → { q: [{s,h,b}, ...] }
 *   3. Tunnel:        POST { k, t, h, p, sid, d }              → { sid, d, eof }
 *   4. Tunnel batch:  POST { k, t:"batch", ops:[...] }         → { r: [...] }
 *      Batch ops include TCP (`connect`, `connect_data`, `data`) and UDP
 *      (`udp_open`, `udp_data`) tunnel-node operations.
 *
 * CHANGE THESE TO YOUR OWN VALUES!
 */

const AUTH_KEY = "CHANGE_ME_TO_A_STRONG_SECRET";
const TUNNEL_SERVER_URL = "https://YOUR_TUNNEL_NODE_URL";
const TUNNEL_AUTH_KEY = "YOUR_TUNNEL_AUTH_KEY";
const HELPER_KIND = "apps_script_full";
const HELPER_VERSION = "2026-05-02.batch20";
const HELPER_PROTOCOL = "mhrv-f.apps-script.v1";
const HELPER_FEATURES = [
  "single",
  "batch",
  "full_tunnel",
  "tunnel_batch",
  "edge_dns_cache",
  "safe_fetchall_fallback",
  "header_privacy",
];

// Set true only while debugging setup/auth mismatches. In normal production,
// unauthorized or malformed probe-shaped requests get a harmless HTML decoy
// instead of proxy-shaped JSON.
const DIAGNOSTIC_MODE = false;

const DECOY_HTML =
  '<!DOCTYPE html><html><head><title>Web App</title></head>' +
  "<body><p>The script completed but did not return anything.</p>" +
  "</body></html>";

function _decoyOrError(jsonBody) {
  if (DIAGNOSTIC_MODE) return _json(jsonBody);
  return ContentService
    .createTextOutput(DECOY_HTML)
    .setMimeType(ContentService.MimeType.HTML);
}

// Edge DNS cache for Full mode. UDP/53 queries normally travel:
// client -> Apps Script -> tunnel-node -> resolver. That first-hop DNS
// round-trip is expensive. When enabled, _doTunnelBatch serves udp_open
// port=53 ops from CacheService or DoH inside Google's network. Any parser,
// cache, or resolver failure returns null and falls through to the existing
// tunnel-node path, so failure is non-regressing.
const ENABLE_EDGE_DNS_CACHE = true;
const EDGE_DNS_RESOLVERS = [
  "https://1.1.1.1/dns-query",
  "https://dns.google/dns-query",
  "https://dns.quad9.net/dns-query",
];
const EDGE_DNS_MIN_TTL_S = 30;
const EDGE_DNS_MAX_TTL_S = 21600; // CacheService ceiling: 6 hours.
const EDGE_DNS_NEG_TTL_S = 45;
const EDGE_DNS_CACHE_PREFIX = "edns:";
const EDGE_DNS_MAX_KEY_LEN = 240;
const EDGE_DNS_REFUSE_QTYPES = { 255: 1 }; // ANY.

// Optional Telegram usage notifications (OFF by default). See Code.gs for docs.
const ENABLE_TELEGRAM_USAGE = false;
const INSTANCE_NAME = "mhrv-relay-full";
const TELEGRAM_BOT_TOKEN = "YOUR_BOT_TOKEN_HERE";
const TELEGRAM_CHAT_ID = "YOUR_CHAT_ID_HERE";
const DAILY_EXECUTION_LIMIT = 20000;
const WARNING_THRESHOLDS = [0.5, 0.75, 0.9, 0.95, 0.99];

// Header forwarding policy: allowlist + explicit blocklist. User-Agent is
// forwarded for browser compatibility; origin/referer and IP/proxy identity
// headers remain blocked.
const ALLOW_HEADERS = {
  accept: 1,
  "accept-language": 1,
  "accept-encoding": 1,
  "cache-control": 1,
  pragma: 1,
  authorization: 1,
  cookie: 1,
  "user-agent": 1,
  range: 1,
  "if-match": 1,
  "if-none-match": 1,
  "if-modified-since": 1,
  "if-unmodified-since": 1,
  "if-range": 1,
  "sec-ch-ua": 1,
  "sec-ch-ua-mobile": 1,
  "sec-ch-ua-platform": 1,
  "sec-fetch-site": 1,
  "sec-fetch-mode": 1,
  "sec-fetch-dest": 1,
  "sec-fetch-user": 1,
};

const SKIP_HEADERS = {
  host: 1, connection: 1, "content-length": 1,
  "transfer-encoding": 1, "proxy-connection": 1, "proxy-authorization": 1,
  "priority": 1, te: 1,
  origin: 1,
  referer: 1,
  // Privacy: never forward client IP hints. If these leak upstream,
  // sanctions blocks and fingerprinting get easier.
  forwarded: 1,
  via: 1,
  "x-forwarded-for": 1,
  "x-forwarded-host": 1,
  "x-forwarded-proto": 1,
  "x-forwarded-port": 1,
  "x-forwarded-server": 1,
  "x-real-ip": 1,
  "x-client-ip": 1,
  "client-ip": 1,
  "true-client-ip": 1,
  "cf-connecting-ip": 1,
  "fastly-client-ip": 1,
  "fly-client-ip": 1,
  "x-cluster-client-ip": 1,
  "x-originating-ip": 1,
  "proxy-client-ip": 1,
  "wl-proxy-client-ip": 1,
  "x-proxyuser-ip": 1,
  "remote-addr": 1,
};

// If fetchAll fails as a whole, retry only methods that are safe to replay.
const SAFE_REPLAY_METHODS = { GET: 1, HEAD: 1, OPTIONS: 1 };

function _trim(s) {
  return String(s).replace(/^\s+|\s+$/g, "");
}
function _headerName(k) {
  return _trim(k).toLowerCase();
}
function _shouldForwardHeader(k) {
  var name = _headerName(k);
  return !!name && ALLOW_HEADERS[name] && !SKIP_HEADERS[name];
}
function _usageStoreKey() {
  return "daily_usage_data";
}
function _getUsageStore() {
  var props = PropertiesService.getScriptProperties();
  var raw = props.getProperty(_usageStoreKey());
  var now = new Date();
  var today = now.toDateString();
  var usage = raw ? JSON.parse(raw) : null;
  if (!usage || usage.date !== today) {
    usage = {
      date: today,
      totalRequests: 0,
      batchRequests: 0,
      singleRequests: 0,
      lastUpdated: now.toISOString(),
      warningSent: [],
    };
    props.setProperty(_usageStoreKey(), JSON.stringify(usage));
  }
  return usage;
}
function _saveUsageStore(usage) {
  PropertiesService.getScriptProperties().setProperty(
    _usageStoreKey(),
    JSON.stringify(usage)
  );
}
function _sendTelegramMessage(message) {
  if (!ENABLE_TELEGRAM_USAGE) return false;
  if (TELEGRAM_BOT_TOKEN === "YOUR_BOT_TOKEN_HERE") return false;
  if (TELEGRAM_CHAT_ID === "YOUR_CHAT_ID_HERE") return false;
  var url = "https://api.telegram.org/bot" + TELEGRAM_BOT_TOKEN + "/sendMessage";
  var payload = {
    chat_id: TELEGRAM_CHAT_ID,
    text: message,
  };
  try {
    var resp = UrlFetchApp.fetch(url, {
      method: "post",
      contentType: "application/json",
      payload: JSON.stringify(payload),
      muteHttpExceptions: true,
      followRedirects: true,
    });
    var result = JSON.parse(resp.getContentText());
    return !!result.ok;
  } catch (_e) {
    return false;
  }
}
function _updateUsage(kind, count) {
  if (!ENABLE_TELEGRAM_USAGE) return;
  count = count || 1;
  var usage = _getUsageStore();
  usage.totalRequests += count;
  if (kind === "batch") usage.batchRequests += count;
  if (kind === "single") usage.singleRequests += count;
  usage.lastUpdated = new Date().toISOString();
  _saveUsageStore(usage);
  var ratio = usage.totalRequests / DAILY_EXECUTION_LIMIT;
  for (var i = 0; i < WARNING_THRESHOLDS.length; i++) {
    var t = WARNING_THRESHOLDS[i];
    if (ratio >= t && usage.warningSent.indexOf(t) === -1) {
      usage.warningSent.push(t);
      _saveUsageStore(usage);
      var pct = Math.round(ratio * 100);
      _sendTelegramMessage(
        "[" + INSTANCE_NAME + "] Apps Script usage warning: " + pct + "% of daily limit\n" +
        "total=" + usage.totalRequests + " limit=" + DAILY_EXECUTION_LIMIT + "\n" +
        "single=" + usage.singleRequests + " batch=" + usage.batchRequests
      );
    }
  }
}
function checkUsageAndNotify() {
  if (!ENABLE_TELEGRAM_USAGE) return;
  var usage = _getUsageStore();
  var ratio = (usage.totalRequests / DAILY_EXECUTION_LIMIT) * 100;
  _sendTelegramMessage(
    "[" + INSTANCE_NAME + "] Apps Script usage: " + ratio.toFixed(1) + "%\n" +
    "total=" + usage.totalRequests + " limit=" + DAILY_EXECUTION_LIMIT + "\n" +
    "single=" + usage.singleRequests + " batch=" + usage.batchRequests
  );
}

// ========================== Entry point ==========================

function doPost(e) {
  try {
    var req = JSON.parse(e.postData.contents);
    if (req.k !== AUTH_KEY) return _decoyOrError({ e: "unauthorized" });

    // Tunnel mode
    if (req.t) return _doTunnel(req);

    // Batch relay mode
    if (Array.isArray(req.q)) {
      _updateUsage("batch", req.q.length);
      return _doBatch(req.q);
    }

    // Single relay mode
    _updateUsage("single", 1);
    return _doSingle(req);
  } catch (err) {
    return _decoyOrError({ e: String(err) });
  }
}

// ========================== Tunnel mode ==========================

function _doTunnel(req) {
  // Batch tunnel: { k, t:"batch", ops:[...] }
  if (req.t === "batch") {
    return _doTunnelBatch(req);
  }

  // Single tunnel op
  var payload = { k: TUNNEL_AUTH_KEY };
  switch (req.t) {
    case "connect":
      payload.op = "connect";
      payload.host = req.h;
      payload.port = req.p;
      break;
    case "connect_data":
      payload.op = "connect_data";
      payload.host = req.h;
      payload.port = req.p;
      if (req.d) payload.data = req.d;
      break;
    case "data":
      payload.op = "data";
      payload.sid = req.sid;
      if (req.d) payload.data = req.d;
      break;
    case "close":
      payload.op = "close";
      payload.sid = req.sid;
      break;
    default:
      // Structured `code` lets the Rust client detect version skew without
      // substring-matching the error text. Must match CODE_UNSUPPORTED_OP
      // in `tunnel-node/src/main.rs` and `src/tunnel_client.rs`.
      return _json({ e: "unknown tunnel op: " + req.t, code: "UNSUPPORTED_OP" });
  }

  var resp = UrlFetchApp.fetch(TUNNEL_SERVER_URL + "/tunnel", {
    method: "post",
    contentType: "application/json",
    payload: JSON.stringify(payload),
    muteHttpExceptions: true,
    // Tunnel payloads include the tunnel auth key. A wrong URL should fail
    // loudly instead of following a redirect and forwarding secrets elsewhere.
    followRedirects: false,
  });

  if (resp.getResponseCode() !== 200) {
    return _json({ e: "tunnel node HTTP " + resp.getResponseCode() });
  }

  return ContentService.createTextOutput(resp.getContentText())
    .setMimeType(ContentService.MimeType.JSON);
}

// Batch tunnel: forward all ops in one request to /tunnel/batch
function _doTunnelBatch(req) {
  var ops = (req && req.ops) || [];
  if (!ENABLE_EDGE_DNS_CACHE) {
    return _doTunnelBatchForward(ops);
  }

  var results = new Array(ops.length);
  var forwardOps = [];
  var forwardIdx = [];
  for (var i = 0; i < ops.length; i++) {
    var op = ops[i];
    if (op && op.op === "udp_open" && op.port === 53 && op.d) {
      var synth = _edgeDnsTry(op);
      if (synth) {
        results[i] = synth;
        continue;
      }
    }
    forwardOps.push(op);
    forwardIdx.push(i);
  }

  if (forwardOps.length === 0) {
    return _json({ r: results });
  }
  if (forwardOps.length === ops.length) {
    return _doTunnelBatchForward(ops);
  }

  var resp = _doTunnelBatchFetch(forwardOps);
  if (resp.error) return _json({ e: resp.error });
  if (resp.r.length !== forwardOps.length) {
    return _json({ e: "tunnel batch length mismatch" });
  }
  return _json({ r: _spliceTunnelResults(forwardIdx, resp.r, results) });
}

function _doTunnelBatchForward(ops) {
  var resp = UrlFetchApp.fetch(TUNNEL_SERVER_URL + "/tunnel/batch", {
    method: "post",
    contentType: "application/json",
    payload: JSON.stringify({ k: TUNNEL_AUTH_KEY, ops: ops }),
    muteHttpExceptions: true,
    // Tunnel payloads include the tunnel auth key. A wrong URL should fail
    // loudly instead of following a redirect and forwarding secrets elsewhere.
    followRedirects: false,
  });

  if (resp.getResponseCode() !== 200) {
    return _json({ e: "tunnel batch HTTP " + resp.getResponseCode() });
  }

  return ContentService.createTextOutput(resp.getContentText())
    .setMimeType(ContentService.MimeType.JSON);
}

function _doTunnelBatchFetch(ops) {
  var resp = UrlFetchApp.fetch(TUNNEL_SERVER_URL + "/tunnel/batch", {
    method: "post",
    contentType: "application/json",
    payload: JSON.stringify({ k: TUNNEL_AUTH_KEY, ops: ops }),
    muteHttpExceptions: true,
    followRedirects: false,
  });
  if (resp.getResponseCode() !== 200) {
    return { error: "tunnel batch HTTP " + resp.getResponseCode() };
  }
  try {
    var parsed = JSON.parse(resp.getContentText());
    return { r: (parsed && parsed.r) || [] };
  } catch (_err) {
    return { error: "tunnel batch parse error" };
  }
}

function _spliceTunnelResults(forwardIdx, forwardedResults, allResults) {
  for (var j = 0; j < forwardIdx.length; j++) {
    allResults[forwardIdx[j]] = forwardedResults[j];
  }
  return allResults;
}

// ========================== HTTP relay mode ==========================

function _doSingle(req) {
  if (!req.u || typeof req.u !== "string" || !req.u.match(/^https?:\/\//i)) {
    return _json({ e: "bad url" });
  }
  var opts = _buildOpts(req);
  var resp = UrlFetchApp.fetch(req.u, opts);
  return _json({
    s: resp.getResponseCode(),
    h: _respHeaders(resp),
    b: Utilities.base64Encode(resp.getContent()),
  });
}

function _doBatch(items) {
  var fetchArgs = [];
  var errorMap = {};
  for (var i = 0; i < items.length; i++) {
    var item = items[i];
    if (!item || typeof item !== "object") {
      errorMap[i] = "bad item";
      continue;
    }
    if (!item.u || typeof item.u !== "string" || !item.u.match(/^https?:\/\//i)) {
      errorMap[i] = "bad url";
      continue;
    }
    try {
      var opts = _buildOpts(item);
      opts.url = item.u;
      fetchArgs.push({
        _i: i,
        _m: (item.m || "GET").toUpperCase(),
        _o: opts,
      });
    } catch (err) {
      errorMap[i] = String(err);
    }
  }
  var responseMap = {};
  if (fetchArgs.length > 0) {
    try {
      var responses = UrlFetchApp.fetchAll(fetchArgs.map(function(x) { return x._o; }));
      for (var a = 0; a < fetchArgs.length; a++) {
        responseMap[fetchArgs[a]._i] = responses[a];
      }
    } catch (err) {
      for (var j = 0; j < fetchArgs.length; j++) {
        try {
          if (!SAFE_REPLAY_METHODS[fetchArgs[j]._m]) {
            errorMap[fetchArgs[j]._i] = "batch fetchAll failed; unsafe method not replayed";
            continue;
          }
          var fallbackReq = fetchArgs[j]._o;
          var fallbackUrl = fallbackReq.url;
          var fallbackOpts = {};
          for (var key in fallbackReq) {
            if (fallbackReq.hasOwnProperty(key) && key !== "url") {
              fallbackOpts[key] = fallbackReq[key];
            }
          }
          responseMap[fetchArgs[j]._i] = UrlFetchApp.fetch(fallbackUrl, fallbackOpts);
        } catch (singleErr) {
          errorMap[fetchArgs[j]._i] = String(singleErr);
        }
      }
    }
  }
  var results = [];
  for (var i = 0; i < items.length; i++) {
    if (errorMap.hasOwnProperty(i)) {
      results.push({ e: errorMap[i] });
    } else {
      var resp = responseMap[i];
      if (!resp) {
        results.push({ e: "fetch failed" });
      } else {
        results.push({
          s: resp.getResponseCode(),
          h: _respHeaders(resp),
          b: Utilities.base64Encode(resp.getContent()),
        });
      }
    }
  }
  return _json({ q: results });
}

// ========================== Helpers ==========================

function _buildOpts(req) {
  var opts = {
    method: (req.m || "GET").toLowerCase(),
    muteHttpExceptions: true,
    followRedirects: req.r !== false,
    validateHttpsCertificates: true,
    escaping: false,
  };
  if (req.h && typeof req.h === "object") {
    var headers = {};
    for (var k in req.h) {
      if (req.h.hasOwnProperty(k) && _shouldForwardHeader(k)) {
        headers[_trim(k)] = req.h[k];
      }
    }
    opts.headers = headers;
  }
  if (req.b) {
    opts.payload = Utilities.base64Decode(req.b);
    if (req.ct) opts.contentType = req.ct;
  }
  return opts;
}

function _respHeaders(resp) {
  try {
    if (typeof resp.getAllHeaders === "function") {
      return resp.getAllHeaders();
    }
  } catch (err) {}
  return resp.getHeaders();
}

function doGet(e) {
  if (e && e.parameter && e.parameter.compat === "1") {
    return _json(_compatInfo());
  }
  return ContentService
    .createTextOutput(DECOY_HTML)
    .setMimeType(ContentService.MimeType.HTML);
}

function _compatInfo() {
  return {
    kind: HELPER_KIND,
    version: HELPER_VERSION,
    protocol: HELPER_PROTOCOL,
    features: HELPER_FEATURES,
  };
}

function _json(obj) {
  return ContentService.createTextOutput(JSON.stringify(obj)).setMimeType(
    ContentService.MimeType.JSON
  );
}

// ========================== Edge DNS helpers ==========================

function _edgeDnsTry(op) {
  try {
    var bytes = Utilities.base64Decode(op.d);
    if (!bytes || bytes.length < 12) return null;
    var q = _dnsParseQuestion(bytes);
    if (!q) return null;
    if (EDGE_DNS_REFUSE_QTYPES[q.qtype]) return null;
    var key = EDGE_DNS_CACHE_PREFIX + q.qtype + ":" + q.qname;
    if (key.length > EDGE_DNS_MAX_KEY_LEN) return null;

    var cache = CacheService.getScriptCache();
    var stored = null;
    try { stored = cache.get(key); } catch (_e) {}
    if (stored) {
      try {
        var hit = Utilities.base64Decode(stored);
        if (hit && hit.length >= 12) {
          return {
            sid: "edns-cache",
            pkts: [Utilities.base64Encode(_dnsRewriteTxid(hit, q.txid))],
            eof: true,
          };
        }
      } catch (_badCache) {}
    }

    for (var i = 0; i < EDGE_DNS_RESOLVERS.length; i++) {
      var reply = _edgeDnsDoh(EDGE_DNS_RESOLVERS[i], bytes);
      if (!reply) continue;
      var rcode = reply[3] & 0x0F;
      var ttl;
      if (rcode === 2 || rcode === 3) {
        ttl = EDGE_DNS_NEG_TTL_S;
      } else {
        var minTtl = _dnsMinTtl(reply);
        ttl = (minTtl === null) ? EDGE_DNS_NEG_TTL_S : minTtl;
        if (ttl < EDGE_DNS_MIN_TTL_S) ttl = EDGE_DNS_MIN_TTL_S;
        if (ttl > EDGE_DNS_MAX_TTL_S) ttl = EDGE_DNS_MAX_TTL_S;
      }
      try {
        cache.put(key, Utilities.base64Encode(reply), ttl);
      } catch (_cachePut) {}
      return {
        sid: "edns-doh",
        pkts: [Utilities.base64Encode(_dnsRewriteTxid(reply, q.txid))],
        eof: true,
      };
    }
    return null;
  } catch (_err) {
    return null;
  }
}

function _edgeDnsDoh(url, queryBytes) {
  try {
    var dns = Utilities.base64EncodeWebSafe(queryBytes).replace(/=+$/, "");
    var resp = UrlFetchApp.fetch(url + "?dns=" + dns, {
      method: "get",
      muteHttpExceptions: true,
      followRedirects: true,
      headers: { accept: "application/dns-message" },
    });
    if (resp.getResponseCode() !== 200) return null;
    var body = resp.getContent();
    if (!body || body.length < 12) return null;
    return body;
  } catch (_err) {
    return null;
  }
}

function _dnsParseQuestion(bytes) {
  if (bytes.length < 12) return null;
  var qdcount = ((bytes[4] & 0xFF) << 8) | (bytes[5] & 0xFF);
  if (qdcount !== 1) return null;
  var off = 12;
  var labels = [];
  var nameLen = 0;
  while (off < bytes.length) {
    var len = bytes[off] & 0xFF;
    if (len === 0) { off++; break; }
    if ((len & 0xC0) !== 0) return null;
    if (len > 63) return null;
    off++;
    if (off + len > bytes.length) return null;
    var label = "";
    for (var i = 0; i < len; i++) {
      var c = bytes[off + i] & 0xFF;
      if (c >= 0x41 && c <= 0x5A) c += 0x20;
      label += String.fromCharCode(c);
    }
    labels.push(label);
    off += len;
    nameLen += len + 1;
    if (nameLen > 255) return null;
  }
  if (off + 4 > bytes.length) return null;
  var qtype = ((bytes[off] & 0xFF) << 8) | (bytes[off + 1] & 0xFF);
  return {
    txid: ((bytes[0] & 0xFF) << 8) | (bytes[1] & 0xFF),
    qname: labels.join("."),
    qtype: qtype,
  };
}

function _dnsMinTtl(bytes) {
  if (bytes.length < 12) return null;
  var qdcount = ((bytes[4] & 0xFF) << 8) | (bytes[5] & 0xFF);
  var ancount = ((bytes[6] & 0xFF) << 8) | (bytes[7] & 0xFF);
  var nscount = ((bytes[8] & 0xFF) << 8) | (bytes[9] & 0xFF);
  var off = 12;
  for (var q = 0; q < qdcount; q++) {
    off = _dnsSkipName(bytes, off);
    if (off < 0 || off + 4 > bytes.length) return null;
    off += 4;
  }
  var min = null;
  var rrTotal = ancount + nscount;
  for (var r = 0; r < rrTotal; r++) {
    off = _dnsSkipName(bytes, off);
    if (off < 0 || off + 10 > bytes.length) return null;
    var ttl = ((bytes[off + 4] & 0xFF) * 0x1000000)
            + (((bytes[off + 5] & 0xFF) << 16)
            |  ((bytes[off + 6] & 0xFF) << 8)
            |   (bytes[off + 7] & 0xFF));
    if (ttl < 0 || ttl > 0x7FFFFFFF) ttl = 0;
    if (min === null || ttl < min) min = ttl;
    var rdlen = ((bytes[off + 8] & 0xFF) << 8) | (bytes[off + 9] & 0xFF);
    off += 10 + rdlen;
    if (off > bytes.length) return null;
  }
  return min;
}

function _dnsSkipName(bytes, off) {
  while (off < bytes.length) {
    var len = bytes[off] & 0xFF;
    if (len === 0) return off + 1;
    if ((len & 0xC0) === 0xC0) {
      if (off + 2 > bytes.length) return -1;
      return off + 2;
    }
    if ((len & 0xC0) !== 0) return -1;
    if (len > 63) return -1;
    off += 1 + len;
  }
  return -1;
}

function _dnsRewriteTxid(bytes, txid) {
  var out = [];
  for (var i = 0; i < bytes.length; i++) out.push(bytes[i]);
  var hi = (txid >> 8) & 0xFF;
  var lo = txid & 0xFF;
  out[0] = hi > 127 ? hi - 256 : hi;
  out[1] = lo > 127 ? lo - 256 : lo;
  return out;
}
