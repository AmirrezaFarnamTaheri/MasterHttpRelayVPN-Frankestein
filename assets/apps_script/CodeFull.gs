/**
 * DomainFront Relay + Full Tunnel — Google Apps Script
 *
 * FOUR modes:
 *   1. Single relay:  POST { k, m, u, h, b, ct, r }           → { s, h, b }
 *   2. Batch relay:   POST { k, q: [{m,u,h,b,ct,r}, ...] }    → { q: [{s,h,b}, ...] }
 *   3. Tunnel:        POST { k, t, h, p, sid, d }              → { sid, d, eof }
 *   4. Tunnel batch:  POST { k, t:"batch", ops:[...] }         → { r: [...] }
 *
 * CHANGE THESE TO YOUR OWN VALUES!
 */

const AUTH_KEY = "CHANGE_ME_TO_A_STRONG_SECRET";
const TUNNEL_SERVER_URL = "https://YOUR_TUNNEL_NODE_URL";
const TUNNEL_AUTH_KEY = "YOUR_TUNNEL_AUTH_KEY";

// Optional Telegram usage notifications (OFF by default). See Code.gs for docs.
const ENABLE_TELEGRAM_USAGE = false;
const INSTANCE_NAME = "mhrv-relay-full";
const TELEGRAM_BOT_TOKEN = "YOUR_BOT_TOKEN_HERE";
const TELEGRAM_CHAT_ID = "YOUR_CHAT_ID_HERE";
const DAILY_EXECUTION_LIMIT = 20000;
const WARNING_THRESHOLDS = [0.5, 0.75, 0.9, 0.95, 0.99];

// Header forwarding policy: allowlist + explicit blocklist.
const ALLOW_HEADERS = {
  accept: 1,
  "accept-language": 1,
  "accept-encoding": 1,
  "cache-control": 1,
  pragma: 1,
  authorization: 1,
  cookie: 1,
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
  "user-agent": 1,
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
    if (req.k !== AUTH_KEY) return _json({ e: "unauthorized" });

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
    return _json({ e: String(err) });
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
    followRedirects: true,
  });

  if (resp.getResponseCode() !== 200) {
    return _json({ e: "tunnel node HTTP " + resp.getResponseCode() });
  }

  return ContentService.createTextOutput(resp.getContentText())
    .setMimeType(ContentService.MimeType.JSON);
}

// Batch tunnel: forward all ops in one request to /tunnel/batch
function _doTunnelBatch(req) {
  var payload = {
    k: TUNNEL_AUTH_KEY,
    ops: req.ops || [],
  };

  var resp = UrlFetchApp.fetch(TUNNEL_SERVER_URL + "/tunnel/batch", {
    method: "post",
    contentType: "application/json",
    payload: JSON.stringify(payload),
    muteHttpExceptions: true,
    followRedirects: true,
  });

  if (resp.getResponseCode() !== 200) {
    return _json({ e: "tunnel batch HTTP " + resp.getResponseCode() });
  }

  return ContentService.createTextOutput(resp.getContentText())
    .setMimeType(ContentService.MimeType.JSON);
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
    if (!item.u || typeof item.u !== "string" || !item.u.match(/^https?:\/\//i)) {
      errorMap[i] = "bad url";
      continue;
    }
    var opts = _buildOpts(item);
    opts.url = item.u;
    fetchArgs.push({ _i: i, _o: opts });
  }
  var responses = [];
  if (fetchArgs.length > 0) {
    responses = UrlFetchApp.fetchAll(fetchArgs.map(function(x) { return x._o; }));
  }
  var results = [];
  var rIdx = 0;
  for (var i = 0; i < items.length; i++) {
    if (errorMap.hasOwnProperty(i)) {
      results.push({ e: errorMap[i] });
    } else {
      var resp = responses[rIdx++];
      results.push({
        s: resp.getResponseCode(),
        h: _respHeaders(resp),
        b: Utilities.base64Encode(resp.getContent()),
      });
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
  return HtmlService.createHtmlOutput(
    "<!DOCTYPE html><html><head><title>My App</title></head>" +
      '<body style="font-family:sans-serif;max-width:600px;margin:40px auto">' +
      "<h1>Welcome</h1><p>This application is running normally.</p>" +
      "</body></html>"
  );
}

function _json(obj) {
  return ContentService.createTextOutput(JSON.stringify(obj)).setMimeType(
    ContentService.MimeType.JSON
  );
}
