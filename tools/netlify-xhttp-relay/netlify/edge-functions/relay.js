const TARGET_BASE = (Netlify.env.get("TARGET_DOMAIN") || "").replace(/\/$/, "");

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
  const headers = new Headers();
  let clientIp = null;

  for (const [key, value] of input) {
    const k = key.toLowerCase();
    if (STRIP_HEADERS.has(k)) continue;
    if (k.startsWith("x-nf-")) continue;
    if (k.startsWith("x-netlify-")) continue;
    if (k === "x-real-ip") {
      clientIp = value;
      continue;
    }
    if (k === "x-forwarded-for") {
      if (!clientIp) clientIp = value;
      continue;
    }
    headers.set(k, value);
  }

  if (clientIp) headers.set("x-forwarded-for", clientIp);
  return headers;
}

function copyResponseHeaders(input) {
  const headers = new Headers();
  for (const [key, value] of input) {
    const k = key.toLowerCase();
    if (k === "transfer-encoding" || k === "connection" || k === "keep-alive") continue;
    headers.set(k, value);
  }
  return headers;
}

export default async function handler(request) {
  if (!TARGET_BASE) {
    return new Response("Misconfigured: TARGET_DOMAIN is not set", { status: 500 });
  }

  try {
    const url = new URL(request.url);
    const targetUrl = TARGET_BASE + url.pathname + url.search;
    const method = request.method;
    const hasBody = method !== "GET" && method !== "HEAD";

    const upstream = await fetch(targetUrl, {
      method,
      headers: copyRequestHeaders(request.headers),
      body: hasBody ? request.body : undefined,
      redirect: "manual",
    });

    return new Response(upstream.body, {
      status: upstream.status,
      statusText: upstream.statusText,
      headers: copyResponseHeaders(upstream.headers),
    });
  } catch (error) {
    console.error("relay error:", error);
    return new Response("Bad Gateway: Tunnel Failed", { status: 502 });
  }
}
