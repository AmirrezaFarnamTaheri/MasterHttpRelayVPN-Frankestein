//! Optional XHTTP relay deployment to Vercel or Netlify from the desktop UI.
//! Provider tokens are held in RAM only; they are never written to `config.json`.

use std::io::Cursor;
use std::io::Write;
use std::sync::mpsc::Sender;
use std::time::Duration;

use rand::Rng;
use reqwest::blocking::Client;
use serde::Deserialize;
use serde_json::json;

const VERCEL_API: &str = "https://api.vercel.com";
const NETLIFY_API: &str = "https://api.netlify.com/api/v1";

/// Messages from a background deploy thread back to egui.
pub enum XhttpDeployWorkerMsg {
    Log(String),
    Done(Result<String, String>),
}

pub fn log_line(tx: &Sender<XhttpDeployWorkerMsg>, line: impl Into<String>) {
    let _ = tx.send(XhttpDeployWorkerMsg::Log(line.into()));
}

struct UsedIds {
    used: std::collections::HashSet<String>,
}

impl UsedIds {
    fn new() -> Self {
        let mut used = std::collections::HashSet::new();
        for s in [
            "config", "process", "env", "Set", "Headers", "Response", "fetch", "console",
        ] {
            used.insert(s.to_string());
        }
        Self { used }
    }

    fn unique_id(&mut self, min_len: usize, max_len: usize, rng: &mut impl Rng) -> String {
        loop {
            let len = rng.gen_range(min_len..=max_len);
            let s = random_identifier(len, rng);
            if self.used.insert(s.clone()) {
                return s;
            }
        }
    }

    fn unique_const(&mut self, min_len: usize, max_len: usize, rng: &mut impl Rng) -> String {
        loop {
            let len = rng.gen_range(min_len..=max_len);
            let s = random_identifier(len, rng).to_uppercase();
            if self.used.insert(s.clone()) {
                return s;
            }
        }
    }
}

fn random_identifier(len: usize, rng: &mut impl Rng) -> String {
    let letters = b"abcdefghijklmnopqrstuvwxyz";
    let chars = b"abcdefghijklmnopqrstuvwxyz0123456789";
    let mut s = String::with_capacity(len.max(2));
    s.push(letters[rng.gen_range(0..letters.len())] as char);
    for _ in 1..len.max(2) {
        s.push(chars[rng.gen_range(0..chars.len())] as char);
    }
    s
}

fn random_route_name(rng: &mut impl Rng) -> String {
    let pool = [
        "handler", "route", "edge", "fn", "svc", "app", "core", "main", "entry", "run", "gw",
        "node", "index", "serve", "mod",
    ];
    let base = pool[rng.gen_range(0..pool.len())];
    let suf = random_identifier(rng.gen_range(2..=5), rng);
    format!("{base}{suf}")
}

fn random_project_name(rng: &mut impl Rng, start_with_letter: bool) -> String {
    let letters = b"abcdefghijklmnopqrstuvwxyz";
    let chars = b"abcdefghijklmnopqrstuvwxyz0123456789";
    let mut s = String::with_capacity(12);
    if start_with_letter {
        s.push(letters[rng.gen_range(0..letters.len())] as char);
        for _ in 1..12 {
            s.push(chars[rng.gen_range(0..chars.len())] as char);
        }
    } else {
        for _ in 0..12 {
            s.push(chars[rng.gen_range(0..chars.len())] as char);
        }
    }
    s
}

fn random_description(rng: &mut impl Rng) -> String {
    let adj = [
        "Lightweight",
        "Minimal",
        "Simple",
        "Fast",
        "Tiny",
        "Modern",
        "Edge",
        "Serverless",
        "Generic",
        "Flexible",
    ];
    let subj = [
        "HTTP middleware",
        "request handler",
        "edge function",
        "API endpoint",
        "web service",
        "utility module",
        "request forwarder",
        "HTTP service",
    ];
    let suf = [
        "for serverless platforms",
        "for edge runtimes",
        "using Web APIs",
        "with zero dependencies",
        "",
    ];
    format!(
        "{} {} {}",
        adj[rng.gen_range(0..adj.len())],
        subj[rng.gen_range(0..subj.len())],
        suf[rng.gen_range(0..suf.len())]
    )
    .split_whitespace()
    .collect::<Vec<_>>()
    .join(" ")
}

fn build_randomized_vercel_edge_js(
    env_var: &str,
    used: &mut UsedIds,
    rng: &mut impl Rng,
) -> String {
    let base_const = used.unique_const(4, 8, rng);
    let skip_set = used.unique_const(4, 8, rng);
    let fn_name = used.unique_id(4, 9, rng);
    let req_arg = used.unique_id(1, 2, rng);
    let idx = used.unique_id(2, 4, rng);
    let dest = used.unique_id(3, 5, rng);
    let hdrs = used.unique_id(2, 4, rng);
    let ip_var = used.unique_id(2, 5, rng);
    let k_var = used.unique_id(1, 2, rng);
    let v_var = used.unique_id(1, 2, rng);
    let m_var = used.unique_id(2, 4, rng);
    let body_flag = used.unique_id(2, 5, rng);
    let err_var = used.unique_id(1, 2, rng);

    format!(
        r#"export const config = {{ runtime: "edge" }};

const {base_const} = (process.env.{env_var} || "").replace(/\/$/, "");

const {skip_set} = new Set([
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

async function {fn_name}({req_arg}) {{
  if (!{base_const}) {{
    return new Response("Service unavailable", {{ status: 500 }});
  }}

  try {{
    const {idx} = {req_arg}.url.indexOf("/", 8);
    const {dest} =
      {idx} === -1 ? {base_const} + "/" : {base_const} + {req_arg}.url.slice({idx});

    const {hdrs} = new Headers();
    let {ip_var} = null;
    for (const [{k_var}, {v_var}] of {req_arg}.headers) {{
      if ({skip_set}.has({k_var})) continue;
      if ({k_var}.startsWith("x-vercel-")) continue;
      if ({k_var} === "x-real-ip") {{
        {ip_var} = {v_var};
        continue;
      }}
      if ({k_var} === "x-forwarded-for") {{
        if (!{ip_var}) {ip_var} = {v_var};
        continue;
      }}
      {hdrs}.set({k_var}, {v_var});
    }}
    if ({ip_var}) {hdrs}.set("x-forwarded-for", {ip_var});

    const {m_var} = {req_arg}.method;
    const {body_flag} = {m_var} !== "GET" && {m_var} !== "HEAD";

    return await fetch({dest}, {{
      method: {m_var},
      headers: {hdrs},
      body: {body_flag} ? {req_arg}.body : undefined,
      duplex: "half",
      redirect: "manual",
    }});
  }} catch ({err_var}) {{
    return new Response("Service error", {{ status: 502 }});
  }}
}}

export default {fn_name};
"#
    )
}

fn plain_vercel_edge_js() -> &'static str {
    include_str!("../tools/vercel-xhttp-relay/api/index.js")
}

fn vercel_deployment_payload(
    project_name: &str,
    target_env_key: &str,
    target_url: &str,
    randomize_names: bool,
    rng: &mut impl Rng,
) -> Result<String, String> {
    let mut used = UsedIds::new();
    let (api_path, api_route_dest) = if randomize_names {
        let route_base = random_route_name(rng);
        let p = format!("api/{route_base}.js");
        (p, format!("/api/{route_base}"))
    } else {
        ("api/index.js".into(), "/api/index".into())
    };

    let api_js = if randomize_names {
        build_randomized_vercel_edge_js(target_env_key, &mut used, rng)
    } else {
        plain_vercel_edge_js().to_string()
    };

    let vercel_json_val = json!({
        "version": 2,
        "name": project_name,
        "rewrites": [{"source": "/(.*)", "destination": api_route_dest}],
        "trailingSlash": false
    });

    let pkg_description = if randomize_names {
        random_description(rng)
    } else {
        "Vercel Edge XHTTP relay for Xray/V2Ray backends".to_string()
    };
    let pkg_val = json!({
        "name": project_name,
        "version": "1.0.0",
        "description": pkg_description,
        "private": true,
        "license": "MIT"
    });

    let files_arr = vec![
        json!({"file":"vercel.json","data":serde_json::to_string_pretty(&vercel_json_val).map_err(|e| e.to_string())?}),
        json!({"file":"package.json","data":serde_json::to_string_pretty(&pkg_val).map_err(|e| e.to_string())?}),
        json!({"file": api_path, "data": api_js}),
    ];

    let mut env_map = serde_json::Map::new();
    env_map.insert(
        target_env_key.to_string(),
        serde_json::Value::String(target_url.to_string()),
    );
    let mut build_env_wrap = serde_json::Map::new();
    build_env_wrap.insert(
        "env".to_string(),
        serde_json::Value::Object(env_map.clone()),
    );

    let body = serde_json::json!({
        "name": project_name,
        "files": files_arr,
        "projectSettings": {
            "framework": null,
            "buildCommand": null,
            "outputDirectory": null
        },
        "env": serde_json::Value::Object(env_map.clone()),
        "build": serde_json::Value::Object(build_env_wrap.clone()),
        "target": "production"
    });

    serde_json::to_string(&body).map_err(|e| e.to_string())
}

fn vercel_api(
    client: &Client,
    token: &str,
    method: reqwest::Method,
    path: &str,
    body: Option<String>,
) -> Result<serde_json::Value, String> {
    let url = format!("{VERCEL_API}{path}");
    let mut req = client
        .request(method, &url)
        .header("Authorization", format!("Bearer {token}"));
    if let Some(b) = body {
        req = req.header("Content-Type", "application/json").body(b);
    }
    let resp = req.send().map_err(|e| e.to_string())?;
    let status = resp.status();
    let text = resp.text().map_err(|e| e.to_string())?;
    if !status.is_success() {
        let err = serde_json::from_str::<serde_json::Value>(&text)
            .ok()
            .and_then(|v| {
                v.get("error")
                    .and_then(|e| e.get("message"))
                    .or_else(|| v.get("message"))
                    .and_then(|m| m.as_str().map(String::from))
            })
            .unwrap_or_else(|| text.clone());
        return Err(format!("Vercel API {status}: {err}"));
    }
    serde_json::from_str(&text).map_err(|e| format!("invalid JSON: {e}: {text}"))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct VercelDeployment {
    id: String,
    url: Option<String>,
    #[serde(default)]
    ready_state: Option<String>,
}

#[derive(Deserialize)]
struct VercelProjects {
    #[serde(default)]
    projects: Vec<VercelProjectBrief>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct VercelProjectBrief {
    id: String,
    name: String,
}

/// Deploy a Vercel Edge XHTTP relay.
pub fn deploy_vercel_xhttp(
    token: &str,
    target_domain: &str,
    randomize_names: bool,
    tx: &Sender<XhttpDeployWorkerMsg>,
) -> Result<String, String> {
    let token = token.trim();
    if token.is_empty() {
        return Err("Vercel API token is required.".into());
    }
    let target_domain = target_domain.trim();
    if target_domain.is_empty() {
        return Err("TARGET_DOMAIN URL is required.".into());
    }
    url::Url::parse(target_domain).map_err(|_| "TARGET_DOMAIN must be a valid URL.".to_string())?;

    let mut rng = rand::thread_rng();
    let project_name = random_project_name(&mut rng, true);
    let env_key = if randomize_names {
        random_project_name(&mut rng, true).to_uppercase()
    } else {
        "TARGET_DOMAIN".into()
    };

    log_line(
        tx,
        format!("project={project_name} env={env_key} randomized_names={randomize_names}"),
    );

    let deployment_json = vercel_deployment_payload(
        &project_name,
        &env_key,
        target_domain,
        randomize_names,
        &mut rng,
    )?;

    let client = Client::builder()
        .timeout(Duration::from_secs(180))
        .build()
        .map_err(|e| e.to_string())?;

    log_line(tx, "Creating Vercel deployment...");
    let dep_val = vercel_api(
        &client,
        token,
        reqwest::Method::POST,
        "/v13/deployments",
        Some(deployment_json),
    )?;
    let deployment: VercelDeployment =
        serde_json::from_value(dep_val).map_err(|e| e.to_string())?;
    let dep_id = deployment.id.clone();
    log_line(tx, format!("deployment id={dep_id}"));

    let mut ready: VercelDeployment = deployment;
    for attempt in 0..60 {
        let state = ready.ready_state.as_deref().unwrap_or("");
        log_line(tx, format!("status: {state}"));
        if state == "READY" {
            break;
        }
        if state == "ERROR" || state == "CANCELED" {
            return Err(format!("Deployment failed with state={state}"));
        }
        std::thread::sleep(Duration::from_secs(2));
        let v = vercel_api(
            &client,
            token,
            reqwest::Method::GET,
            &format!("/v13/deployments/{dep_id}"),
            None,
        )?;
        ready = serde_json::from_value(v).map_err(|e| e.to_string())?;
        if attempt == 59 {
            return Err("Deployment timed out waiting for READY.".into());
        }
    }

    let host = ready
        .url
        .ok_or_else(|| "Vercel response missing deployment hostname.".to_string())?;
    log_line(tx, format!("ready: {host}"));

    if let Ok(v) = vercel_api(&client, token, reqwest::Method::GET, "/v9/projects", None) {
        let list: VercelProjects =
            serde_json::from_value(v).unwrap_or(VercelProjects { projects: vec![] });
        if let Some(p) = list.projects.into_iter().find(|p| p.name == project_name) {
            log_line(
                tx,
                format!("PATCH project {} (disable deployment protection)", p.id),
            );
            let patch = json!({
                "passwordProtection": serde_json::Value::Null,
                "ssoProtection": serde_json::Value::Null
            });
            let _ = vercel_api(
                &client,
                token,
                reqwest::Method::PATCH,
                &format!("/v9/projects/{}", p.id),
                Some(patch.to_string()),
            );
        }
    }

    Ok(host)
}

// â”€â”€â”€ Netlify: site + zip upload (backend URL baked into edge script) â”€â”€â”€â”€â”€â”€â”€â”€â”€

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct NetlifySiteCreate {
    id: String,
    #[serde(default)]
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct NetlifyDeployInfo {
    id: String,
    #[serde(default)]
    state: Option<String>,
    /// Present when Netlify has published this deploy to production (authoritative success signal).
    #[serde(default)]
    published_at: Option<String>,
    #[serde(default)]
    error_message: Option<String>,
}

/// Resolve one GET `/deploys/{id}` payload into whether polling should stop.
#[derive(Debug, Clone, PartialEq, Eq)]
enum NetlifyPollResolved {
    /// Keep polling - build/upload/processing still in flight (or state not yet populated).
    Continue,
    /// Deploy is live / Netlify marks it published.
    Success,
    /// Terminal failure; message is user-facing.
    Failed(String),
}

fn netlify_deploy_poll_resolution(info: &NetlifyDeployInfo) -> NetlifyPollResolved {
    let published = info
        .published_at
        .as_deref()
        .map(|s| !s.trim().is_empty())
        .unwrap_or(false);
    if published {
        return NetlifyPollResolved::Success;
    }

    let raw = info.state.as_deref().unwrap_or("").trim();
    let state = raw.to_ascii_lowercase();

    match state.as_str() {
        // Terminal success (ZIP / CLI parity - Netlify mostly uses `ready`; some tooling surfaces `published`).
        "ready" | "published" | "live" => NetlifyPollResolved::Success,
        // Terminal failures
        "error" | "fatal" => {
            let fallback = if state == "fatal" {
                "Netlify deploy failed (fatal)."
            } else {
                "Netlify reported deploy error."
            };
            let detail = info
                .error_message
                .as_deref()
                .filter(|m| !m.trim().is_empty())
                .unwrap_or(fallback);
            NetlifyPollResolved::Failed(detail.to_string())
        }
        "timeout" => {
            NetlifyPollResolved::Failed("Netlify deploy timed out during build/upload.".into())
        }
        // Duplicate / noop deploy - treat as success only if already published (handled above).
        // Otherwise keep polling briefly in case the API fills `published_at` late.
        "skipped" => NetlifyPollResolved::Continue,
        // Empty / unknown early responses
        "" => NetlifyPollResolved::Continue,
        // In-flight states seen from live Build API / UI (normalize case via `state`)
        "new" | "pending" | "pending_review" | "uploaded" | "uploading" | "building"
        | "building_site" | "processing" | "preparing" | "prepared" | "draft" | "init"
        | "enqueued" | "started" | "retrying" => NetlifyPollResolved::Continue,
        other => {
            // Fail-safe: avoid exiting early on an unrecognized terminal string Netlify adds later.
            if matches!(
                other,
                "cancelled" | "canceled" | "failed" | "rejected" | "blocked"
            ) {
                let suffix = info
                    .error_message
                    .as_deref()
                    .filter(|m| !m.trim().is_empty())
                    .map(|m| format!(" - {m}"))
                    .unwrap_or_default();
                NetlifyPollResolved::Failed(format!(
                    "Netlify deploy stopped (state={raw}).{suffix}"
                ))
            } else {
                NetlifyPollResolved::Continue
            }
        }
    }
}

fn netlify_js_with_baked_origin(
    target_origin: &str,
    randomize_names: bool,
    rng: &mut impl Rng,
) -> String {
    let base_trim = target_origin.trim().trim_end_matches('/');
    let lit = serde_json::to_string(base_trim).expect("literal");
    let base_js = format!(
        r#"const TARGET_BASE = ({lit} || "").replace(/\/$/, "");

const STRIP_HEADERS = new Set([
  "host","connection","keep-alive","proxy-authenticate","proxy-authorization",
  "te","trailer","transfer-encoding","upgrade","forwarded","x-forwarded-host",
  "x-forwarded-proto","x-forwarded-port",
]);

function copyRequestHeaders(input) {{
  const headers = new Headers();
  let clientIp = null;
  for (const [key, value] of input) {{
    const k = key.toLowerCase();
    if (STRIP_HEADERS.has(k)) continue;
    if (k.startsWith("x-nf-")) continue;
    if (k.startsWith("x-netlify-")) continue;
    if (k === "x-real-ip") {{ clientIp = value; continue; }}
    if (k === "x-forwarded-for") {{ if (!clientIp) clientIp = value; continue; }}
    headers.set(k, value);
  }}
  if (clientIp) headers.set("x-forwarded-for", clientIp);
  return headers;
}}

function copyResponseHeaders(input) {{
  const headers = new Headers();
  for (const [key, value] of input) {{
    const k = key.toLowerCase();
    if (k === "transfer-encoding" || k === "connection" || k === "keep-alive") continue;
    headers.set(k, value);
  }}
  return headers;
}}

export default async function HANDLER_REPLACE(request) {{
  if (!TARGET_BASE) {{
    return new Response("Misconfigured relay", {{ status: 500 }});
  }}
  try {{
    const url = new URL(request.url);
    const targetUrl = TARGET_BASE + url.pathname + url.search;
    const method = request.method;
    const hasBody = method !== "GET" && method !== "HEAD";

    const upstream = await fetch(targetUrl, {{
      method,
      headers: copyRequestHeaders(request.headers),
      body: hasBody ? request.body : undefined,
      redirect: "manual",
    }});

    return new Response(upstream.body, {{
      status: upstream.status,
      statusText: upstream.statusText,
      headers: copyResponseHeaders(upstream.headers),
    }});
  }} catch (error) {{
    console.error("relay error:", error);
    return new Response("Bad Gateway", {{ status: 502 }});
  }}
}}
"#,
        lit = lit,
    );

    if !randomize_names {
        return base_js.replace("HANDLER_REPLACE", "handler");
    }

    let h = random_identifier(5 + rng.gen_range(0..3), rng);
    base_js.replace("HANDLER_REPLACE", &h)
}

const NETLIFY_TOML: &str = r#"[build]
publish = "public"

[[edge_functions]]
function = "relay"
path = "/p4r34m"

[[edge_functions]]
function = "relay"
path = "/p4r34m/*"
"#;

fn netlify_site_zip(
    site_name: &str,
    target_origin: &str,
    randomize_names: bool,
    rng: &mut impl Rng,
) -> Result<Vec<u8>, String> {
    let relay_js = netlify_js_with_baked_origin(target_origin, randomize_names, rng);
    let pkg = r#"{"name":"relay-site","version":"1.0.0","private":true,"type":"module"}"#
        .replace("relay-site", site_name);

    let index_html = r#"<!doctype html><html lang="en"><head><meta charset="utf-8"><title>XHTTP relay</title></head>\
<body><p>Relay targets <code>/p4r34m</code> -&gt; backend baked at deploy.</p></body></html>"#;

    let cursor = Cursor::new(Vec::<u8>::new());
    let mut zip = zip::ZipWriter::new(cursor);
    let opts = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    zip.start_file("netlify.toml", opts)
        .map_err(|e| e.to_string())?;
    zip.write_all(NETLIFY_TOML.as_bytes())
        .map_err(|e| e.to_string())?;

    zip.start_file("package.json", opts)
        .map_err(|e| e.to_string())?;
    zip.write_all(pkg.as_bytes()).map_err(|e| e.to_string())?;

    zip.start_file("public/index.html", opts)
        .map_err(|e| e.to_string())?;
    zip.write_all(index_html.as_bytes())
        .map_err(|e| e.to_string())?;

    zip.start_file("netlify/edge-functions/relay.js", opts)
        .map_err(|e| e.to_string())?;
    zip.write_all(relay_js.as_bytes())
        .map_err(|e| e.to_string())?;

    let cursor = zip.finish().map_err(|e| e.to_string())?;
    Ok(cursor.into_inner())
}

#[derive(Deserialize)]
struct NetlifyErrorBody {
    message: Option<String>,
}

/// Netlify PAT deploy using the ZIP-based Sites API.
pub fn deploy_netlify_xhttp(
    token: &str,
    target_domain: &str,
    randomize_names: bool,
    tx: &Sender<XhttpDeployWorkerMsg>,
) -> Result<String, String> {
    let token = token.trim();
    if token.is_empty() {
        return Err("Netlify personal access token is required.".into());
    }
    let target_domain = target_domain.trim();
    if target_domain.is_empty() {
        return Err("Backend origin URL is required.".into());
    }
    url::Url::parse(target_domain)
        .map_err(|_| "Backend URL must include http/https.".to_string())?;

    let mut rng = rand::thread_rng();
    let client = Client::builder()
        .timeout(Duration::from_secs(240))
        .build()
        .map_err(|e| e.to_string())?;

    let mut site_slug = random_project_name(&mut rng, true);
    log_line(tx, format!("creating site {site_slug}..."));

    let site: NetlifySiteCreate = loop {
        let resp = client
            .post(format!("{NETLIFY_API}/sites"))
            .header("Authorization", format!("Bearer {token}"))
            .header("Content-Type", "application/json")
            .body(json!({ "name": site_slug }).to_string())
            .send()
            .map_err(|e| e.to_string())?;

        let status = resp.status();
        let text = resp.text().map_err(|e| e.to_string())?;

        if status.is_success() {
            break serde_json::from_str(&text).map_err(|e| format!("{e}: {text}"))?;
        }

        let err_txt = serde_json::from_str::<NetlifyErrorBody>(&text)
            .ok()
            .and_then(|x| x.message);
        let retryable_name_conflict = status.as_u16() == 422
            && err_txt
                .as_deref()
                .map(|m| {
                    let m = m.to_ascii_lowercase();
                    m.contains("name") && (m.contains("taken") || m.contains("exists"))
                })
                .unwrap_or(false);
        if retryable_name_conflict {
            log_line(
                tx,
                format!(
                    "site create {status}: {} - retrying new name...",
                    err_txt.as_deref().unwrap_or(text.trim())
                ),
            );
            site_slug = random_project_name(&mut rng, true);
        } else {
            return Err(format!(
                "Netlify site create {status}: {}",
                err_txt.as_deref().unwrap_or(text.trim())
            ));
        }
    };

    let site_id = site.id;
    log_line(tx, format!("site_id={site_id}"));
    log_line(tx, "uploading Edge bundle zip...");

    let zip_bytes = netlify_site_zip(
        site.name.as_deref().unwrap_or("site"),
        target_domain,
        randomize_names,
        &mut rng,
    )?;

    let resp = client
        .post(format!("{NETLIFY_API}/sites/{site_id}/deploys"))
        .header("Authorization", format!("Bearer {token}"))
        .header("Content-Type", "application/zip")
        .body(zip_bytes)
        .send()
        .map_err(|e| e.to_string())?;

    let status = resp.status();
    let text = resp.text().map_err(|e| e.to_string())?;
    if !status.is_success() {
        return Err(format!("Netlify deploy start {status}: {text}"));
    }

    let deploy: NetlifyDeployInfo =
        serde_json::from_str(&text).map_err(|e| format!("{e}: {text}"))?;

    let deploy_id = deploy.id.clone();
    log_line(tx, format!("deploy_id={deploy_id}"));

    let mut last = deploy;
    for attempt in 0..90 {
        let st = last.state.as_deref().unwrap_or("(none)");
        let published_flag = last
            .published_at
            .as_deref()
            .map(|s| !s.trim().is_empty())
            .unwrap_or(false);
        log_line(
            tx,
            format!(
                "deploy [{attempt}/89]: state={st}{}",
                if published_flag {
                    " published_at=yes"
                } else {
                    ""
                }
            ),
        );

        match netlify_deploy_poll_resolution(&last) {
            NetlifyPollResolved::Success => break,
            NetlifyPollResolved::Failed(msg) => return Err(msg),
            NetlifyPollResolved::Continue => {}
        }

        if attempt == 89 {
            return Err(format!(
                "Netlify deploy timed out (last state={:?}, published_at={:?}).",
                last.state, last.published_at
            ));
        }

        std::thread::sleep(Duration::from_secs(2));
        let r = client
            .get(format!("{NETLIFY_API}/deploys/{deploy_id}"))
            .header("Authorization", format!("Bearer {token}"))
            .send()
            .map_err(|e| e.to_string())?;

        let st_code = r.status();
        let ok = st_code.is_success();
        let t = r.text().map_err(|e| e.to_string())?;
        if !ok {
            return Err(format!("poll deploy {st_code}: {t}"));
        }
        last = serde_json::from_str(&t).map_err(|e| format!("poll parse: {e}"))?;
    }

    let hostname = format!(
        "{}.netlify.app",
        site.name.unwrap_or_else(|| site_slug.clone())
    );
    log_line(tx, format!("published: {hostname}"));
    Ok(hostname)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vercel_randomized_contains_edge_runtime_string() {
        let mut u = UsedIds::new();
        let mut rng = rand::thread_rng();
        let s = build_randomized_vercel_edge_js("XENV", &mut u, &mut rng);
        assert!(s.contains("\"edge\""));
        assert!(s.contains("export default"));
        assert!(s.contains("process.env.XENV"));
    }

    #[test]
    fn netlify_zip_nonempty() {
        let mut rng = rand::thread_rng();
        let z = netlify_site_zip(
            "t",
            "https://backend.example.invalid:8443/",
            false,
            &mut rng,
        )
        .unwrap();
        assert!(z.len() > 120);
        assert!(z.starts_with(b"PK"));
    }

    #[test]
    fn netlify_deploy_info_deserializes_api_shape() {
        let j = r#"{"id":"d1","state":"processing","publishedAt":null,"errorMessage":null}"#;
        let d: NetlifyDeployInfo = serde_json::from_str(j).unwrap();
        assert_eq!(d.id, "d1");
        assert_eq!(d.state.as_deref(), Some("processing"));
        assert!(d.published_at.is_none());
    }

    #[test]
    fn netlify_poll_ready_case_insensitive() {
        let i = NetlifyDeployInfo {
            id: "1".into(),
            state: Some("READY".into()),
            published_at: None,
            error_message: None,
        };
        assert_eq!(
            netlify_deploy_poll_resolution(&i),
            NetlifyPollResolved::Success
        );
    }

    #[test]
    fn netlify_poll_published_at_overrides_processing_state() {
        let i = NetlifyDeployInfo {
            id: "1".into(),
            state: Some("processing".into()),
            published_at: Some("2026-04-01T12:00:00.000Z".into()),
            error_message: None,
        };
        assert_eq!(
            netlify_deploy_poll_resolution(&i),
            NetlifyPollResolved::Success
        );
    }

    #[test]
    fn netlify_poll_error_surfaces_message() {
        let i = NetlifyDeployInfo {
            id: "1".into(),
            state: Some("error".into()),
            published_at: None,
            error_message: Some("Edge bundling failed".into()),
        };
        assert_eq!(
            netlify_deploy_poll_resolution(&i),
            NetlifyPollResolved::Failed("Edge bundling failed".into())
        );
    }

    #[test]
    fn netlify_poll_building_keeps_polling() {
        let i = NetlifyDeployInfo {
            id: "1".into(),
            state: Some("building".into()),
            published_at: None,
            error_message: None,
        };
        assert_eq!(
            netlify_deploy_poll_resolution(&i),
            NetlifyPollResolved::Continue
        );
    }
}
