/**
 * DomainFront Relay — Google Apps Script
 *
 * TWO modes:
 *   1. Single:  POST { k, m, u, h, b, ct, r }       → { s, h, b }
 *   2. Batch:   POST { k, q: [{m,u,h,b,ct,r}, ...] } → { q: [{s,h,b}, ...] }
 *      Uses UrlFetchApp.fetchAll() — all URLs fetched IN PARALLEL.
 *
 * OPTIONAL SPREADSHEET-BACKED RESPONSE CACHE:
 *   Set CACHE_SPREADSHEET_ID to a valid Google Sheet ID owned by the same
 *   account. Public GET requests can then be served from the sheet on repeat
 *   visits, reducing UrlFetchApp quota. Leave the placeholder unchanged to
 *   disable caching entirely.
 *
 * DEPLOYMENT:
 *   1. Go to https://script.google.com → New project
 *   2. Delete the default code, paste THIS entire file
 *   3. Click Deploy → New deployment
 *   4. Type: Web app  |  Execute as: Me  |  Who has access: Anyone
 *   5. Copy the Deployment ID into config.json under account_groups[].script_ids
 *
 * CHANGE THE AUTH KEY BELOW TO YOUR OWN SECRET!
 */

const AUTH_KEY = "CHANGE_ME_TO_A_STRONG_SECRET";
const HELPER_KIND = "apps_script";
const HELPER_VERSION = "2026-05-02.batch20";
const HELPER_PROTOCOL = "mhrv-f.apps-script.v1";
const HELPER_FEATURES = [
  "single",
  "batch",
  "safe_fetchall_fallback",
  "header_privacy",
  "response_cache_optional",
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
// Optional Spreadsheet response cache (OFF by default).
// ---------------------------------------------------------------------------
const CACHE_SPREADSHEET_ID = "CHANGE_ME_TO_CACHE_SPREADSHEET_ID";
const CACHE_SHEET_NAME = "RelayCache";
const CACHE_META_SHEET_NAME = "RelayMeta";
const CACHE_META_CURSOR_CELL = "A1";
const CACHE_MAX_ROWS = 5000;
const CACHE_MAX_BODY_BYTES = 35000;      // under the Google Sheets cell limit
const CACHE_DEFAULT_TTL_SECONDS = 86400; // 24h fallback
const VARY_KEY_HEADERS = ["accept-encoding", "accept-language"];

// ---------------------------------------------------------------------------
// Header forwarding policy.
//
// We deliberately do NOT forward arbitrary client-supplied headers.
// - Privacy: block forwarded-IP and proxy chain headers.
// - Safety: block Origin/Referer injection from the client.
// - Compatibility: forward User-Agent so sites like YouTube do not downgrade
//   desktop browsers to mobile/bot fallbacks when UrlFetchApp would otherwise
//   use its own Google-Apps-Script agent.
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
  "user-agent": 1,
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

const CACHE_BUSTING_HEADERS = {
  authorization: 1, cookie: 1, "x-api-key": 1,
  "proxy-authorization": 1, "set-cookie": 1,
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
    if (req.k !== AUTH_KEY) return _decoyOrError({ e: "unauthorized" });

    // Batch mode: { k, q: [...] }
    if (Array.isArray(req.q)) {
      _updateUsage("batch", req.q.length);
      return _doBatch(req.q);
    }

    // Single mode
    _updateUsage("single", 1);
    return _doSingle(req);
  } catch (err) {
    return _decoyOrError({ e: String(err) });
  }
}

function _doSingle(req) {
  if (!req.u || typeof req.u !== "string" || !req.u.match(/^https?:\/\//i)) {
    return _json({ e: "bad url" });
  }
  if (_canUseCache(req)) {
    var cached = _getFromCache(req.u, req.h);
    if (cached) {
      return _json({
        s: cached.status,
        h: JSON.parse(cached.headers),
        b: cached.body,
        cached: true,
      });
    }
    var fetched = _fetchAndCache(req.u, req.h);
    if (fetched) {
      return _json({
        s: fetched.status,
        h: JSON.parse(fetched.headers),
        b: fetched.body,
        cached: false,
      });
    }
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

  // fetchAll() processes all requests in parallel inside Google
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

function _initCacheSheet() {
  if (CACHE_SPREADSHEET_ID === "CHANGE_ME_TO_CACHE_SPREADSHEET_ID") return null;
  try {
    var ss = SpreadsheetApp.openById(CACHE_SPREADSHEET_ID);
    var sheet = ss.getSheetByName(CACHE_SHEET_NAME);
    if (!sheet) {
      sheet = ss.insertSheet(CACHE_SHEET_NAME);
      sheet.getRange(1, 1, 1, 7).setValues([[
        "URL_Hash", "URL", "Status", "Headers", "Body", "Timestamp", "Expires_At"
      ]]);
    }
    return sheet;
  } catch (_e) {
    return null;
  }
}

function _getMetaSheet() {
  if (CACHE_SPREADSHEET_ID === "CHANGE_ME_TO_CACHE_SPREADSHEET_ID") return null;
  try {
    var ss = SpreadsheetApp.openById(CACHE_SPREADSHEET_ID);
    var sheet = ss.getSheetByName(CACHE_META_SHEET_NAME);
    if (!sheet) {
      sheet = ss.insertSheet(CACHE_META_SHEET_NAME);
      sheet.getRange(CACHE_META_CURSOR_CELL).setValue(2);
      sheet.hideSheet();
    }
    return sheet;
  } catch (_e) {
    return null;
  }
}

function _ensureRowsAllocated(sheet) {
  var totalRows = sheet.getDataRange().getNumRows();
  if (totalRows < CACHE_MAX_ROWS + 1) {
    sheet.insertRowsAfter(totalRows, CACHE_MAX_ROWS + 1 - totalRows);
  }
}

function _getNextCursor(sheet, metaSheet) {
  var cursor = metaSheet.getRange(CACHE_META_CURSOR_CELL).getValue();
  if (typeof cursor !== "number" || cursor < 2) cursor = 2;
  var totalRows = sheet.getDataRange().getNumRows();
  if (totalRows < CACHE_MAX_ROWS + 1) return totalRows + 1;
  return cursor;
}

function _advanceCursor(metaSheet, currentRow) {
  var nextRow = currentRow + 1;
  if (nextRow > CACHE_MAX_ROWS + 1) nextRow = 2;
  metaSheet.getRange(CACHE_META_CURSOR_CELL).setValue(nextRow);
}

function _getHeaderCaseInsensitive(headers, targetKey) {
  var target = targetKey.toLowerCase();
  for (var k in headers || {}) {
    if (headers.hasOwnProperty(k) && k.toLowerCase() === target) return headers[k];
  }
  return null;
}

function _md5Hex(input) {
  var rawHash = Utilities.computeDigest(Utilities.DigestAlgorithm.MD5, input);
  return rawHash.map(function (byte) {
    var v = byte < 0 ? 256 + byte : byte;
    return ("0" + v.toString(16)).slice(-2);
  }).join("");
}

function _getCacheKey(url, reqHeaders) {
  var parts = [url];
  for (var i = 0; i < VARY_KEY_HEADERS.length; i++) {
    var name = VARY_KEY_HEADERS[i];
    var raw = _getHeaderCaseInsensitive(reqHeaders || {}, name);
    if (raw && String(raw).trim() !== "") {
      parts.push(name + ":" + String(raw).toLowerCase().replace(/\s/g, ""));
    } else {
      parts.push(name + ":<none>");
    }
  }
  return _md5Hex(parts.join("|"));
}

function _canUseCache(req) {
  if ((req.m || "GET").toUpperCase() !== "GET") return false;
  if (req.b) return false;
  if (!req.u || !req.u.match(/^https?:\/\//i)) return false;
  if (CACHE_SPREADSHEET_ID === "CHANGE_ME_TO_CACHE_SPREADSHEET_ID") return false;
  if (req.h && typeof req.h === "object") {
    for (var k in req.h) {
      if (req.h.hasOwnProperty(k) && CACHE_BUSTING_HEADERS[k.toLowerCase()]) return false;
    }
  }
  return true;
}

function _parseMaxAge(cacheControlHeader) {
  if (!cacheControlHeader) return CACHE_DEFAULT_TTL_SECONDS;
  var lower = String(cacheControlHeader).toLowerCase();
  if (
    lower.indexOf("no-cache") !== -1 ||
    lower.indexOf("no-store") !== -1 ||
    lower.indexOf("private") !== -1
  ) {
    return 0;
  }
  var match = lower.match(/max-age=(\d+)/);
  if (match) {
    var ttl = parseInt(match[1], 10);
    return Math.max(60, Math.min(ttl, 2592000));
  }
  return CACHE_DEFAULT_TTL_SECONDS;
}

function _refreshCachedHeaders(headersJson, timestamp) {
  var headers = JSON.parse(headersJson);
  var cachedAt = new Date(timestamp);
  var now = new Date();
  var ageSeconds = Math.floor((now.getTime() - cachedAt.getTime()) / 1000);
  if (ageSeconds < 0) ageSeconds = 0;
  headers["Date"] = now.toUTCString();
  headers["Age"] = String(ageSeconds);
  var originalCc = headers["Cache-Control"] || headers["cache-control"];
  if (originalCc) headers["X-Original-Cache-Control"] = originalCc;
  headers["Cache-Control"] = "public, max-age=" +
    Math.max(0, _parseMaxAge(originalCc) - ageSeconds);
  headers["X-Cache"] = "HIT from relay-spreadsheet";
  headers["X-Cached-At"] = cachedAt.toUTCString();
  return JSON.stringify(headers);
}

function _getFromCache(url, reqHeaders) {
  var sheet = _initCacheSheet();
  if (!sheet) return null;
  var hash = _getCacheKey(url, reqHeaders);
  var found = sheet.createTextFinder(hash).matchEntireCell(true).findNext();
  if (!found) return null;
  var row = sheet.getRange(found.getRow(), 1, 1, 7).getValues()[0];
  var expiresAt = row[6];
  if (expiresAt && expiresAt instanceof Date && expiresAt < new Date()) return null;
  return {
    status: row[2],
    headers: _refreshCachedHeaders(row[3], row[5]),
    body: row[4],
  };
}

function _fetchAndCache(url, reqHeaders) {
  var sheet = _initCacheSheet();
  if (!sheet) return null;
  try {
    var response = UrlFetchApp.fetch(url, _buildOpts({ m: "GET", h: reqHeaders || {} }));
    var status = response.getResponseCode();
    var headers = _respHeaders(response);
    var body = Utilities.base64Encode(response.getContent());
    if (body.length > CACHE_MAX_BODY_BYTES) {
      return { status: status, headers: JSON.stringify(headers), body: body };
    }
    var cacheControl = headers["Cache-Control"] || headers["cache-control"] || null;
    var ttlSeconds = _parseMaxAge(cacheControl);
    if (ttlSeconds === 0) {
      return { status: status, headers: JSON.stringify(headers), body: body };
    }
    var hash = _getCacheKey(url, reqHeaders);
    var timestamp = new Date();
    var expiresAt = new Date(timestamp.getTime() + ttlSeconds * 1000);
    if (isNaN(expiresAt.getTime())) {
      expiresAt = new Date(timestamp.getTime() + CACHE_DEFAULT_TTL_SECONDS * 1000);
    }
    var rowData = [
      hash,
      url,
      status,
      JSON.stringify(headers),
      body,
      timestamp.toISOString(),
      expiresAt,
    ];
    var metaSheet = _getMetaSheet();
    if (metaSheet) {
      _ensureRowsAllocated(sheet);
      var writeRow = _getNextCursor(sheet, metaSheet);
      sheet.getRange(writeRow, 1, 1, 7).setValues([rowData]);
      _advanceCursor(metaSheet, writeRow);
    } else {
      sheet.appendRow(rowData);
    }
    return { status: status, headers: JSON.stringify(headers), body: body };
  } catch (_e) {
    return null;
  }
}

function getCacheStats() {
  var sheet = _initCacheSheet();
  if (!sheet) {
    console.log("Cache is not enabled or spreadsheet unavailable.");
    return;
  }
  var data = sheet.getDataRange().getValues();
  var totalEntries = data.length - 1;
  var now = new Date();
  var expiredCount = 0;
  for (var i = 1; i < data.length; i++) {
    var expiresAt = data[i][6];
    if (expiresAt && expiresAt instanceof Date && expiresAt < now) expiredCount++;
  }
  var metaSheet = _getMetaSheet();
  var cursorInfo = metaSheet ? String(metaSheet.getRange(CACHE_META_CURSOR_CELL).getValue()) : "N/A";
  console.log("=== CACHE STATS ===");
  console.log("Total rows used: " + totalEntries + " / " + CACHE_MAX_ROWS);
  console.log("Active entries: " + (totalEntries - expiredCount));
  console.log("Expired entries: " + expiredCount);
  console.log("Cursor position: " + cursorInfo);
}

function clearExpiredCache() {
  var sheet = _initCacheSheet();
  if (!sheet) return;
  var data = sheet.getDataRange().getValues();
  var now = new Date();
  var cleared = 0;
  for (var i = 1; i < data.length; i++) {
    var expiresAt = data[i][6];
    if (expiresAt && expiresAt instanceof Date && expiresAt < now) {
      sheet.getRange(i + 1, 1, 1, 7).clearContent();
      cleared++;
    }
  }
  console.log("Cleared " + cleared + " expired entries.");
}

function clearEntireCache() {
  var sheet = _initCacheSheet();
  if (sheet) {
    var totalRows = sheet.getDataRange().getNumRows();
    if (totalRows > 1) sheet.getRange(2, 1, totalRows - 1, 7).clearContent();
  }
  var metaSheet = _getMetaSheet();
  if (metaSheet) metaSheet.getRange(CACHE_META_CURSOR_CELL).setValue(2);
  console.log("Cache wiped. Cursor reset to row 2.");
}

function _json(obj) {
  return ContentService.createTextOutput(JSON.stringify(obj)).setMimeType(
    ContentService.MimeType.JSON
  );
}
