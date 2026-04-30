use crate::config::Config;
use crate::doctor;
use crate::status_api;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, thiserror::Error)]
pub enum SupportBundleError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
}

fn now_ts() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn sanitize_config(mut cfg: Config) -> Config {
    // Strip secrets but keep structure for debugging.
    if let Some(groups) = cfg.account_groups.as_mut() {
        for g in groups {
            if !g.auth_key.trim().is_empty() {
                g.auth_key = "<redacted>".into();
            }
            // Deployment IDs are not strictly secrets, but treat them as sensitive.
            // Keep only masked prefixes for correlation.
            let ids = g.script_ids.clone().into_vec();
            let masked: Vec<String> = ids
                .into_iter()
                .map(|id| {
                    let t = id.trim().to_string();
                    if t.len() <= 10 {
                        "<redacted>".into()
                    } else {
                        format!("{}…{}", &t[..6], &t[t.len() - 4..])
                    }
                })
                .collect();
            g.script_ids = crate::config::ScriptId::Many(masked);
        }
    }
    if !cfg.vercel.auth_key.trim().is_empty() {
        cfg.vercel.auth_key = "<redacted>".into();
    }
    cfg.lan_token = None;
    cfg
}

fn write_text(path: &Path, text: &str) -> Result<(), SupportBundleError> {
    if let Some(p) = path.parent() {
        fs::create_dir_all(p)?;
    }
    let mut f = fs::File::create(path)?;
    f.write_all(text.as_bytes())?;
    Ok(())
}

fn write_json<T: serde::Serialize>(path: &Path, v: &T) -> Result<(), SupportBundleError> {
    let s = serde_json::to_string_pretty(v)?;
    write_text(path, &s)
}

/// Export an anonymized diagnostics bundle into a folder and return its path.
///
/// This is a directory (not a zip) by design: it works everywhere without extra
/// dependencies, and users can inspect it before sharing.
pub async fn export_support_bundle(cfg: &Config) -> Result<PathBuf, SupportBundleError> {
    let base = crate::data_dir::data_dir();
    let out_dir = base
        .join("support-bundles")
        .join(format!("bundle-{}", now_ts()));
    fs::create_dir_all(&out_dir)?;

    // 1) Metadata
    let meta = serde_json::json!({
        "generated_at_unix": now_ts(),
        "version": env!("CARGO_PKG_VERSION"),
        "platform": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
    });
    write_json(&out_dir.join("meta.json"), &meta)?;

    // 2) Config (sanitized)
    let sanitized = sanitize_config(cfg.clone());
    write_json(&out_dir.join("config.redacted.json"), &sanitized)?;

    // 3) Doctor report
    let report = doctor::run(cfg).await;
    // Serialize doctor report in a simple JSON shape.
    let items: Vec<serde_json::Value> = report
        .items
        .iter()
        .map(|it| {
            serde_json::json!({
                "id": it.id,
                "level": match it.level { doctor::DoctorLevel::Ok => "ok", doctor::DoctorLevel::Warn => "warn", doctor::DoctorLevel::Fail => "fail" },
                "title": it.title,
                "detail": it.detail,
                "fix": it.fix,
            })
        })
        .collect();
    write_json(
        &out_dir.join("doctor.json"),
        &serde_json::json!({ "ok": report.ok(), "items": items }),
    )?;

    // 4) Status JSON (best-effort; uses same renderer as local status API)
    // We don't have a running DomainFronter here; render a minimal status view.
    let status = status_api::render_status_json(
        &cfg.mode,
        (&cfg.listen_host, cfg.listen_port),
        cfg.socks5_port.map(|p| (cfg.listen_host.as_str(), p)),
        None,
    );
    write_text(&out_dir.join("status.json"), &status)?;

    Ok(out_dir)
}
