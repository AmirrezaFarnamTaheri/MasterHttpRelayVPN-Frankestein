/**
 * DomainFront Relay — Google Apps Script
 *
 * TWO modes:
 *   1. Single:  POST { k, m, u, h, b, ct, r }       → { s, h, b }
 *   2. Batch:   POST { k, q: [{m,u,h,b,ct,r}, ...] } → { q: [{s,h,b}, ...] }
 *      Uses UrlFetchApp.fetchAll() — all URLs fetched IN PARALLEL.
 *
 * DEPLOYMENT:
 *   1. Go to https://script.google.com → New project
 *   2. Delete the default code, paste THIS entire file
 *   3. Click Deploy → New deployment
 *   4. Type: Web app  |  Execute as: Me  |  Who has access: Anyone
 *   5. Copy the Deployment ID into config.json as "script_id"
 *
 * CHANGE THE AUTH KEY BELOW TO YOUR OWN SECRET!
 */

const AUTH_KEY = "CHANGE_ME_TO_A_STRONG_SECRET";

// ---------------------------------------------------------------------------
// Optional Telegram usage notifications (OFF by default).
//
// Why: Apps Script has a daily UrlFetchApp quota per Google account. Many users
// only notice the quota after things start timing out. This can send a short
// warning when usage crosses thresholds.
//
// How to enable:
//   1) Create a bot via @BotFather and put its token in TELEGRAM_BOT_TOKEN
//   2) Put your numeric chat id in TELEGRAM_CHAT_ID (often starts with -100...)
//   3) Set ENABLE_TELEGRAM_USAGE=true
//   4) (Optional) Add a trigger: Triggers → Add trigger → checkUsageAndNotify
//      Time-driven → Minutes timer → every N minutes
//
// Notes:
// - Keep it OFF unless you want it: this is extra HTTP calls from the script.
// - Messages are plain (no emojis) to match the project style.
// ---------------------------------------------------------------------------
const ENABLE_TELEGRAM_USAGE = false;
const INSTANCE_NAME = "mhrv-relay";
const TELEGRAM_BOT_TOKEN = "YOUR_BOT_TOKEN_HERE";
const TELEGRAM_CHAT_ID = "YOUR_CHAT_ID_HERE";
const DAILY_EXECUTION_LIMIT = 20000;
const WARNING_THRESHOLDS = [0.5, 0.75, 0.9, 0.95, 0.99];

// ---------------------------------------------------------------------------
// Header forwarding policy.
//
// We deliberately do NOT forward arbitrary client-supplied headers.
// - Privacy: block forwarded-IP and proxy chain headers.
// - Safety: block Origin/Referer/User-Agent injection from the client.
// - Compatibility: allow the "browser capability" headers (sec-ch-ua*,
//   sec-fetch-*) used for gating by some sites (e.g. Google Meet).
// ---------------------------------------------------------------------------
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
  // Browser capability hints.
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

  // Don't let callers spoof identity / origin.
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

function checkUsageAndNotify() {
  // Trigger-friendly periodic message. OFF by default.
  if (!ENABLE_TELEGRAM_USAGE) return;
  var usage = _getUsageStore();
  var ratio = (usage.totalRequests / DAILY_EXECUTION_LIMIT) * 100;
  _sendTelegramMessage(
    "[" + INSTANCE_NAME + "] Apps Script usage: " + ratio.toFixed(1) + "%\n" +
    "total=" + usage.totalRequests + " limit=" + DAILY_EXECUTION_LIMIT + "\n" +
    "single=" + usage.singleRequests + " batch=" + usage.batchRequests
  );
}

function doPost(e) {
  try {
    var req = JSON.parse(e.postData.contents);
    if (req.k !== AUTH_KEY) return _json({ e: "unauthorized" });

    // Batch mode: { k, q: [...] }
    if (Array.isArray(req.q)) {
      _updateUsage("batch", req.q.length);
      return _doBatch(req.q);
    }

    // Single mode
    _updateUsage("single", 1);
    return _doSingle(req);
  } catch (err) {
    return _json({ e: String(err) });
  }
}

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

  // fetchAll() processes all requests in parallel inside Google
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
