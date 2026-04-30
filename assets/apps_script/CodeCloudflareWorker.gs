/**
 * mhrv-f Apps Script relay with Cloudflare Worker exit.
 *
 * FLOW:
 *   mhrv-f client -> Apps Script -> Cloudflare Worker -> target website
 *
 * This keeps the normal mhrv-f Apps Script JSON protocol:
 *   Single: POST { k, m, u, h, b, ct, r } -> { s, h, b }
 *   Batch:  POST { k, q: [{m,u,h,b,ct,r}, ...] } -> { q: [{s,h,b}, ...] }
 *
 * CHANGE THESE:
 *   AUTH_KEY: client-facing secret, same value you put in mhrv-f config.
 *   WORKER_URL: your Cloudflare Worker URL.
 *   WORKER_AUTH_KEY: Worker-facing secret, same value as the Worker env var.
 */

const AUTH_KEY = "CHANGE_ME_TO_A_STRONG_CLIENT_SECRET";
const WORKER_URL = "https://example.workers.dev";
const WORKER_AUTH_KEY = "CHANGE_ME_TO_A_STRONG_WORKER_SECRET";
const DIAGNOSTIC_MODE = false;

const DECOY_HTML = '<!DOCTYPE html><html><head><title>Apps Script</title></head><body>The script completed but did not return anything.</body></html>';

const SKIP_HEADERS = {
  host: 1,
  connection: 1,
  "content-length": 1,
  "transfer-encoding": 1,
  "proxy-connection": 1,
  "proxy-authorization": 1,
  "proxy-authenticate": 1,
  "keep-alive": 1,
  te: 1,
  trailer: 1,
  upgrade: 1,
  forwarded: 1,
  "x-forwarded-for": 1,
  "x-forwarded-host": 1,
  "x-forwarded-proto": 1,
  "x-forwarded-port": 1,
  "x-real-ip": 1,
  origin: 1,
  referer: 1,
};

function doPost(e) {
  try {
    var req = JSON.parse(e.postData.contents);
    if (req.k !== AUTH_KEY) return _decoyOrError({ e: "unauthorized" });

    if (Array.isArray(req.q)) return _doBatch(req.q);
    return _doSingle(req);
  } catch (err) {
    return _decoyOrError({ e: "malformed request" });
  }
}

function _doSingle(req) {
  if (!_validUrl(req.u)) return _json({ e: "bad url" });

  var resp = UrlFetchApp.fetch(WORKER_URL, {
    method: "post",
    contentType: "application/json",
    payload: JSON.stringify(_buildWorkerPayload(req)),
    muteHttpExceptions: true,
    followRedirects: false,
  });

  return _parseWorkerResponse(resp);
}

function _doBatch(items) {
  var fetchArgs = [];
  var errorMap = {};

  for (var i = 0; i < items.length; i++) {
    var item = items[i];
    if (!_validUrl(item.u)) {
      errorMap[i] = "bad url";
      continue;
    }
    fetchArgs.push({
      _i: i,
      _o: {
        url: WORKER_URL,
        method: "post",
        contentType: "application/json",
        payload: JSON.stringify(_buildWorkerPayload(item)),
        muteHttpExceptions: true,
        followRedirects: false,
      },
    });
  }

  var responses = [];
  if (fetchArgs.length > 0) {
    responses = UrlFetchApp.fetchAll(fetchArgs.map(function(x) { return x._o; }));
  }

  var results = [];
  var rIdx = 0;
  for (var j = 0; j < items.length; j++) {
    if (errorMap.hasOwnProperty(j)) {
      results.push({ e: errorMap[j] });
    } else {
      var parsed = _parseWorkerJson(responses[rIdx++]);
      results.push(parsed);
    }
  }
  return _json({ q: results });
}

function _buildWorkerPayload(req) {
  var headers = {};
  if (req.h && typeof req.h === "object") {
    for (var k in req.h) {
      if (req.h.hasOwnProperty(k) && !SKIP_HEADERS[String(k).toLowerCase()]) {
        headers[k] = req.h[k];
      }
    }
  }
  return {
    wk: WORKER_AUTH_KEY,
    u: req.u,
    m: (req.m || "GET").toUpperCase(),
    h: headers,
    b: req.b || null,
    ct: req.ct || null,
    r: req.r !== false,
  };
}

function _parseWorkerResponse(resp) {
  return _json(_parseWorkerJson(resp));
}

function _parseWorkerJson(resp) {
  try {
    var parsed = JSON.parse(resp.getContentText());
    if (parsed && typeof parsed === "object") return parsed;
  } catch (_e) {}
  return { e: "invalid worker response", s: resp.getResponseCode() };
}

function _validUrl(url) {
  return typeof url === "string" && /^https?:\/\//i.test(url);
}

function doGet(_e) {
  return ContentService
    .createTextOutput(DECOY_HTML)
    .setMimeType(ContentService.MimeType.HTML);
}

function _decoyOrError(jsonBody) {
  if (DIAGNOSTIC_MODE) return _json(jsonBody);
  return ContentService
    .createTextOutput(DECOY_HTML)
    .setMimeType(ContentService.MimeType.HTML);
}

function _json(obj) {
  return ContentService
    .createTextOutput(JSON.stringify(obj))
    .setMimeType(ContentService.MimeType.JSON);
}
