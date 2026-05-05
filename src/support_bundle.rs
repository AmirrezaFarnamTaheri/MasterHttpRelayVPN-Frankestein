use crate::config::Config;
use crate::doctor;
use crate::redaction::{
    mask_deployment_id as shared_mask_deployment_id, redact_config_secrets_in_text,
    REDACTED_AUTH_KEY, REDACTED_SERVERLESS_AUTH_KEY,
};
use crate::status_api;
use crate::trust_center;
use serde::Serialize;
use std::cmp::Reverse;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const MAX_RECENT_LOG_BYTES: usize = 64 * 1024;

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

#[derive(Clone, Debug, Serialize)]
pub struct SupportBundleManifest {
    pub schema_version: u32,
    pub output_kind: &'static str,
    pub redaction: RedactionPolicy,
    pub files: Vec<SupportBundleFile>,
    pub review_before_sharing: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct RedactionPolicy {
    pub auth_keys: &'static str,
    pub lan_tokens: &'static str,
    pub deployment_ids: &'static str,
    pub private_keys: &'static str,
}

#[derive(Clone, Debug, Serialize)]
pub struct SupportBundleFile {
    pub path: &'static str,
    pub category: &'static str,
    pub description: &'static str,
    pub contains_sensitive_material: bool,
}

pub fn preview_manifest() -> SupportBundleManifest {
    SupportBundleManifest {
        schema_version: 1,
        output_kind: "directory",
        redaction: RedactionPolicy {
            auth_keys: "redacted",
            lan_tokens: "removed",
            deployment_ids: "masked_prefix_suffix",
            private_keys: "not_included",
        },
        files: vec![
            SupportBundleFile {
                path: "manifest.json",
                category: "index",
                description: "Machine-readable bundle table of contents and redaction policy.",
                contains_sensitive_material: false,
            },
            SupportBundleFile {
                path: "meta.json",
                category: "environment",
                description: "Version, platform, architecture, and generation timestamp.",
                contains_sensitive_material: false,
            },
            SupportBundleFile {
                path: "config.redacted.json",
                category: "configuration",
                description: "Config structure with auth keys redacted, LAN token removed, and deployment IDs masked.",
                contains_sensitive_material: true,
            },
            SupportBundleFile {
                path: "doctor.json",
                category: "diagnostics",
                description: "Structured Doctor result, including IDs, levels, details, and suggested fixes.",
                contains_sensitive_material: true,
            },
            SupportBundleFile {
                path: "status.json",
                category: "runtime",
                description: "Minimal status snapshot rendered through the local status API shape.",
                contains_sensitive_material: false,
            },
            SupportBundleFile {
                path: "trust.json",
                category: "trust",
                description: "Trust Center snapshot for CA requirement, CA file/trust state, Android CA caveats, and signing policy pointers.",
                contains_sensitive_material: true,
            },
            SupportBundleFile {
                path: "recent-logs.txt",
                category: "logs",
                description: "Bounded recent log excerpt when a persistent log file is present; otherwise a note explaining that no persistent log file was found.",
                contains_sensitive_material: true,
            },
        ],
        review_before_sharing: true,
    }
}

fn sanitize_config(mut cfg: Config) -> Config {
    // Strip secrets but keep structure for debugging.
    if let Some(groups) = cfg.account_groups.as_mut() {
        for g in groups {
            if !g.auth_key.trim().is_empty() {
                g.auth_key = REDACTED_AUTH_KEY.into();
            }
            // Deployment IDs are not strictly secrets, but treat them as sensitive.
            // Keep only masked prefixes for correlation.
            let ids = g.script_ids.clone().into_vec();
            let masked: Vec<String> = ids
                .into_iter()
                .map(|id| shared_mask_deployment_id(id.trim()))
                .collect();
            g.script_ids = crate::config::ScriptId::Many(masked);
        }
    }
    if !cfg.vercel.auth_key.trim().is_empty() {
        cfg.vercel.auth_key = REDACTED_SERVERLESS_AUTH_KEY.into();
    }
    cfg.lan_token = None;
    cfg
}

fn redact_sensitive_text(text: &str, cfg: &Config) -> String {
    redact_config_secrets_in_text(text, cfg)
}

fn recent_logs_text(base: &Path, cfg: &Config) -> String {
    let candidates = latest_log_candidates(base);
    let Some(path) = candidates.into_iter().next() else {
        return "No persistent recent log file was found. Desktop and Android UI logs may still be available in their live log panels.\n".into();
    };
    let header = format!(
        "# Recent logs\nsource={}\nmax_bytes={}\n\n",
        path.display(),
        MAX_RECENT_LOG_BYTES
    );
    match fs::read(&path) {
        Ok(bytes) => {
            let start = bytes.len().saturating_sub(MAX_RECENT_LOG_BYTES);
            let text = String::from_utf8_lossy(&bytes[start..]);
            format!("{header}{}", redact_sensitive_text(&text, cfg))
        }
        Err(e) => format!(
            "# Recent logs\nsource={}\nerror=failed to read: {}\n",
            path.display(),
            e
        ),
    }
}

fn latest_log_candidates(base: &Path) -> Vec<PathBuf> {
    let mut found: Vec<(SystemTime, PathBuf)> = Vec::new();
    for dir in [base.to_path_buf(), base.join("logs")] {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("log") {
                continue;
            }
            let modified = entry
                .metadata()
                .and_then(|m| m.modified())
                .unwrap_or(UNIX_EPOCH);
            found.push((modified, path));
        }
    }
    found.sort_by_key(|item| Reverse(item.0));
    found.into_iter().map(|(_, path)| path).collect()
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

    // 1) Manifest / preview
    write_json(&out_dir.join("manifest.json"), &preview_manifest())?;

    // 2) Metadata
    let meta = serde_json::json!({
        "generated_at_unix": now_ts(),
        "version": env!("CARGO_PKG_VERSION"),
        "platform": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
    });
    write_json(&out_dir.join("meta.json"), &meta)?;

    // 3) Config (sanitized)
    let sanitized = sanitize_config(cfg.clone());
    write_json(&out_dir.join("config.redacted.json"), &sanitized)?;

    // 4) Doctor report
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

    // 5) Status JSON (best-effort; uses same renderer as local status API)
    // We don't have a running DomainFronter here; render a minimal status view.
    let status = status_api::render_status_json(
        &cfg.mode,
        (&cfg.listen_host, cfg.listen_port),
        cfg.socks5_port.map(|p| (cfg.listen_host.as_str(), p)),
        None,
    );
    write_text(&out_dir.join("status.json"), &status)?;

    // 6) Trust Center snapshot: shared, non-mutating view of CA/signing trust.
    write_json(&out_dir.join("trust.json"), &trust_center::snapshot(cfg))?;

    // 7) Bounded, redacted recent logs when a persistent log file exists.
    write_text(
        &out_dir.join("recent-logs.txt"),
        &recent_logs_text(&base, cfg),
    )?;

    Ok(out_dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preview_manifest_lists_actual_bundle_files() {
        let manifest = preview_manifest();
        let files: Vec<&str> = manifest.files.iter().map(|f| f.path).collect();
        assert_eq!(
            files,
            vec![
                "manifest.json",
                "meta.json",
                "config.redacted.json",
                "doctor.json",
                "status.json",
                "trust.json",
                "recent-logs.txt"
            ]
        );
        assert!(manifest.review_before_sharing);
    }

    #[test]
    fn preview_manifest_documents_redaction_policy() {
        let text = serde_json::to_string(&preview_manifest()).expect("manifest json");
        assert!(text.contains("auth_keys"));
        assert!(text.contains("redacted"));
        assert!(text.contains("private_keys"));
        assert!(text.contains("not_included"));
        assert!(!text.contains("CHANGE_ME_TO_A_STRONG_SECRET"));
    }

    #[test]
    fn redact_sensitive_text_masks_known_config_secrets() {
        let cfg = Config::from_json_str(
            r#"{
                "mode": "apps_script",
                "account_groups": [{
                    "auth_key": "secret-auth-key",
                    "script_ids": ["AKfycb1234567890abcdef"]
                }],
                "lan_token": "lan-secret-token"
            }"#,
        )
        .expect("config");
        let text = "secret-auth-key AKfycb1234567890abcdef lan-secret-token should disappear";
        let redacted = redact_sensitive_text(text, &cfg);
        assert!(!redacted.contains("secret-auth-key"));
        assert!(!redacted.contains("AKfycb1234567890abcdef"));
        assert!(!redacted.contains("lan-secret-token"));
        assert!(redacted.contains("<redacted-auth-key>"));
        assert!(redacted.contains("AKfycb...cdef"));
        assert!(redacted.contains("<redacted-lan-token>"));
    }

    #[test]
    fn sanitize_config_uses_shared_redaction_tokens() {
        let cfg = Config::from_json_str(
            r#"{
                "mode": "vercel_edge",
                "account_groups": [{
                    "auth_key": "secret-auth-key",
                    "script_ids": ["AKfycb1234567890abcdef"]
                }],
                "vercel": {
                    "base_url": "https://example.net",
                    "auth_key": "serverless-secret"
                },
                "lan_token": "lan-secret-token"
            }"#,
        )
        .expect("config");
        let sanitized = sanitize_config(cfg);
        let group = sanitized.account_groups.as_ref().unwrap().first().unwrap();
        assert_eq!(group.auth_key, REDACTED_AUTH_KEY);
        assert_eq!(
            group.script_ids.clone().into_vec(),
            vec!["AKfycb...cdef".to_string()]
        );
        assert_eq!(sanitized.vercel.auth_key, REDACTED_SERVERLESS_AUTH_KEY);
        assert!(sanitized.lan_token.is_none());
    }
}
