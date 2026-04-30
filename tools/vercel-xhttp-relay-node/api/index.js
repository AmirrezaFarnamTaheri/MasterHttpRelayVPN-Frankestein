import { Readable } from "node:stream";
import { pipeline } from "node:stream/promises";

export const config = {
  api: { bodyParser: false },
  supportsResponseStreaming: true,
  maxDuration: 60,
};

const TARGET_BASE = (process.env.TARGET_DOMAIN || "").replace(/\/$/, "");

const STRIP_HEADERS = new Set([
  "host",
  "connection",
  "keep-alive",
  "proxy-authenticate",
  "proxy-authorization",
  "te",
  "trailer",
  "transfer-encoding",
  "upgrade",
  "forwarded",
  "x-forwarded-host",
  "x-forwarded-proto",
  "x-forwarded-port",
]);

function copyRequestHeaders(input) {
  const headers = {};
  let clientIp = null;

  for (const key of Object.keys(input)) {
    const k = key.toLowerCase();
    const value = input[key];
    if (STRIP_HEADERS.has(k)) continue;
    if (k.startsWith("x-vercel-")) continue;
    if (k === "x-real-ip") {
      clientIp = value;
      continue;
    }
    if (k === "x-forwarded-for") {
      if (!clientIp) clientIp = value;
      continue;
    }
    headers[k] = Array.isArray(value) ? value.join(", ") : value;
  }

  if (clientIp) {
    headers["x-forwarded-for"] = Array.isArray(clientIp) ? clientIp.join(", ") : clientIp;
  }
  return headers;
}

export default async function handler(req, res) {
  if (!TARGET_BASE) {
    res.statusCode = 500;
    return res.end("Misconfigured: TARGET_DOMAIN is not set");
  }

  try {
    const method = req.method || "GET";
    const hasBody = method !== "GET" && method !== "HEAD";
    const fetchOptions = {
      method,
      headers: copyRequestHeaders(req.headers),
      redirect: "manual",
    };

    if (hasBody) {
      fetchOptions.body = Readable.toWeb(req);
      fetchOptions.duplex = "half";
    }

    const upstream = await fetch(TARGET_BASE + req.url, fetchOptions);

    res.statusCode = upstream.status;
    res.statusMessage = upstream.statusText;
    for (const [key, value] of upstream.headers) {
      const k = key.toLowerCase();
      if (k === "transfer-encoding" || k === "connection" || k === "keep-alive") continue;
      try {
        res.setHeader(key, value);
      } catch {
        // Ignore platform-rejected response headers.
      }
    }

    if (upstream.body) {
      await pipeline(Readable.fromWeb(upstream.body), res);
    } else {
      res.end();
    }
  } catch (error) {
    console.error("relay error:", error);
    if (!res.headersSent) {
      res.statusCode = 502;
      res.end("Bad Gateway: Tunnel Failed");
    } else {
      res.destroy(error);
    }
  }
}
