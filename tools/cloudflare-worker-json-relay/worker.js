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
  "x-forwarded-for",
  "x-forwarded-host",
  "x-forwarded-proto",
  "x-forwarded-port",
  "x-real-ip",
  "cf-connecting-ip",
  "cf-ipcountry",
  "cf-ray",
  "cf-visitor",
]);

export default {
  async fetch(request, env) {
    if (request.headers.get("x-relay-hop") === "1") {
      return json({ e: "loop detected" }, 508);
    }

    try {
      const workerAuthKey = (env.WORKER_AUTH_KEY || "").trim();
      if (!workerAuthKey) return json({ e: "WORKER_AUTH_KEY not configured" }, 500);

      const req = await request.json();
      if (req.wk !== workerAuthKey) return json({ e: "unauthorized" }, 401);
      if (!req.u || typeof req.u !== "string") return json({ e: "missing url" }, 400);

      const targetUrl = new URL(req.u);
      if (targetUrl.protocol !== "http:" && targetUrl.protocol !== "https:") {
        return json({ e: "bad url" }, 400);
      }
      if (targetUrl.hostname.endsWith(".workers.dev")) {
        return json({ e: "worker self-fetch blocked" }, 400);
      }

      const headers = new Headers();
      if (req.h && typeof req.h === "object") {
        for (const [key, value] of Object.entries(req.h)) {
          const k = key.toLowerCase();
          if (!STRIP_HEADERS.has(k)) headers.set(key, value);
        }
      }
      headers.set("x-relay-hop", "1");

      const method = (req.m || "GET").toUpperCase();
      const fetchOptions = {
        method,
        headers,
        redirect: req.r === false ? "manual" : "follow",
      };
      if (req.b && method !== "GET" && method !== "HEAD") {
        fetchOptions.body = base64ToBytes(req.b);
      }

      const upstream = await fetch(targetUrl.toString(), fetchOptions);
      const responseHeaders = {};
      upstream.headers.forEach((value, key) => {
        responseHeaders[key] = value;
      });

      return json({
        s: upstream.status,
        h: responseHeaders,
        b: bytesToBase64(new Uint8Array(await upstream.arrayBuffer())),
      });
    } catch (error) {
      console.error("worker relay error:", error);
      return json({ e: "worker relay failed" }, 502);
    }
  },
};

function json(obj, status = 200) {
  return new Response(JSON.stringify(obj), {
    status,
    headers: { "content-type": "application/json" },
  });
}

function base64ToBytes(input) {
  const binary = atob(input);
  const out = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i++) out[i] = binary.charCodeAt(i);
  return out;
}

function bytesToBase64(bytes) {
  let binary = "";
  const chunkSize = 0x8000;
  for (let i = 0; i < bytes.length; i += chunkSize) {
    binary += String.fromCharCode.apply(null, bytes.subarray(i, i + chunkSize));
  }
  return btoa(binary);
}
