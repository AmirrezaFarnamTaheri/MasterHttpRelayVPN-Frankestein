//! MasterHttpRelayVPN-Frankestein — `mhrv-f doctor` (first-run diagnostics).
//!
//! Goal: maximize first-run success by detecting the common failure modes and
//! printing actionable next steps.

use crate::cert_installer::{install_ca, is_ca_trusted};
use crate::config::{Config, Mode};
use crate::mitm::{MitmCertManager, CA_CERT_FILE};
use crate::test_cmd;

#[derive(Clone, Debug)]
pub enum DoctorLevel {
    Ok,
    Warn,
    Fail,
}

#[derive(Clone, Debug)]
pub struct DoctorItem {
    pub id: &'static str,
    pub level: DoctorLevel,
    pub title: String,
    pub detail: String,
    pub fix: Option<String>,
}

#[derive(Clone, Debug)]
pub struct DoctorReport {
    pub items: Vec<DoctorItem>,
}

impl DoctorReport {
    pub fn ok(&self) -> bool {
        !self
            .items
            .iter()
            .any(|i| matches!(i.level, DoctorLevel::Fail))
    }
}

pub async fn run(config: &Config) -> DoctorReport {
    let mut items: Vec<DoctorItem> = Vec::new();

    // 1) Config safety warnings (non-fatal).
    let warns = config.unsafe_warnings();
    if warns.is_empty() {
        items.push(DoctorItem {
            id: "config_warnings",
            level: DoctorLevel::Ok,
            title: "Config warnings".into(),
            detail: "No unsafe-setting warnings detected.".into(),
            fix: None,
        });
    } else {
        items.push(DoctorItem {
            id: "config_warnings",
            level: DoctorLevel::Warn,
            title: "Config warnings".into(),
            detail: warns.join("\n- "),
            fix: Some("Review the warnings above and adjust config.json if needed.".into()),
        });
    }

    // 2) Mode sanity.
    let mode = match config.mode_kind() {
        Ok(m) => m,
        Err(e) => {
            items.push(DoctorItem {
                id: "mode",
                level: DoctorLevel::Fail,
                title: "Mode".into(),
                detail: format!("Invalid mode: {e}"),
                fix: Some("Set `mode` to one of: apps_script, vercel_edge, direct, full.".into()),
            });
            return DoctorReport { items };
        }
    };
    items.push(DoctorItem {
        id: "mode",
        level: DoctorLevel::Ok,
        title: "Mode".into(),
        detail: format!("mode = {}", mode.as_str()),
        fix: None,
    });

    // 3) Relay config presence.
    if matches!(mode, Mode::AppsScript | Mode::Full) {
        let groups = config.account_groups_resolved();
        if groups.is_empty() {
            items.push(DoctorItem {
                id: "account_groups",
                level: DoctorLevel::Fail,
                title: "Apps Script accounts".into(),
                detail: "No enabled account_groups found.".into(),
                fix: Some(
                    "Add at least one enabled account group with auth_key + script_ids (deployment IDs).".into(),
                ),
            });
        } else {
            let total_ids: usize = groups
                .iter()
                .map(|g| g.script_ids.clone().into_vec().len())
                .sum();
            items.push(DoctorItem {
                id: "account_groups",
                level: DoctorLevel::Ok,
                title: "Apps Script accounts".into(),
                detail: format!(
                    "Enabled accounts: {}  Deployments: {}",
                    groups.len(),
                    total_ids
                ),
                fix: None,
            });
        }
    } else if mode == Mode::VercelEdge {
        if config.vercel.base_url.trim().is_empty() || config.vercel.auth_key.trim().is_empty() {
            items.push(DoctorItem {
                id: "vercel_edge",
                level: DoctorLevel::Fail,
                title: "Serverless JSON relay".into(),
                detail: "vercel.base_url and vercel.auth_key are required.".into(),
                fix: Some(
                    "Deploy tools/vercel-json-relay or tools/netlify-json-relay, set AUTH_KEY, then copy the deployment URL and key into config.json."
                        .into(),
                ),
            });
        } else {
            items.push(DoctorItem {
                id: "vercel_edge",
                level: DoctorLevel::Ok,
                title: "Serverless JSON relay".into(),
                detail: format!(
                    "Endpoint: {}{}",
                    config.vercel.base_url.trim_end_matches('/'),
                    config.vercel.relay_path
                ),
                fix: None,
            });
        }
    }

    // 4) MITM CA readiness / trust. Not needed for full mode.
    if mode == Mode::Full {
        items.push(DoctorItem {
            id: "mitm_ca",
            level: DoctorLevel::Ok,
            title: "MITM certificate".into(),
            detail: "Full mode: MITM CA not required.".into(),
            fix: None,
        });
    } else {
        let base = crate::data_dir::data_dir();
        match MitmCertManager::new_in(&base) {
            Ok(_) => {
                let ca_path = base.join(CA_CERT_FILE);
                let trusted = is_ca_trusted(&ca_path);
                if trusted {
                    items.push(DoctorItem {
                        id: "mitm_ca",
                        level: DoctorLevel::Ok,
                        title: "MITM certificate".into(),
                        detail: format!(
                            "CA file exists and appears trusted: {}",
                            ca_path.display()
                        ),
                        fix: None,
                    });
                } else {
                    items.push(DoctorItem {
                        id: "mitm_ca",
                        level: DoctorLevel::Warn,
                        title: "MITM certificate".into(),
                        detail: format!("CA file exists but does not appear trusted: {}", ca_path.display()),
                        fix: Some("Run `mhrv-f --install-cert` (may require admin) or import ca/ca.crt into your OS trust store.".into()),
                    });
                }
            }
            Err(e) => items.push(DoctorItem {
                id: "mitm_ca",
                level: DoctorLevel::Fail,
                title: "MITM certificate".into(),
                detail: format!("Failed to initialize local CA files: {e}"),
                fix: Some("Ensure the user-data directory is writable and try again.".into()),
            }),
        }
    }

    // 5) End-to-end relay probe (same as `test`), only when relay exists.
    if matches!(mode, Mode::AppsScript | Mode::VercelEdge) {
        let ok = test_cmd::run(config).await;
        items.push(DoctorItem {
            id: "relay_probe",
            level: if ok { DoctorLevel::Ok } else { DoctorLevel::Fail },
            title: "Relay probe".into(),
            detail: if ok {
                "PASS: end-to-end relay verified.".into()
            } else {
                "FAIL: relay probe failed. See logs above for the detailed reason.".into()
            },
            fix: if ok {
                None
            } else {
                Some("Common fixes: verify AUTH_KEY matches; for Apps Script, replace dead deployment IDs/re-deploy as 'Anyone'/scan a different google_ip; for serverless JSON, remove protection/routing pages and redeploy after env var changes.".into())
            },
        });
    } else if mode == Mode::Full {
        items.push(DoctorItem {
            id: "relay_probe",
            level: DoctorLevel::Warn,
            title: "Relay probe".into(),
            detail: "`mhrv-f test` is intentionally skipped in full mode; verify full mode by browsing through the tunnel and checking the tunnel-node public IP.".into(),
            fix: None,
        });
    }

    DoctorReport { items }
}

#[derive(Clone, Debug)]
pub struct FixOutcome {
    pub id: &'static str,
    pub ok: bool,
    pub detail: String,
}

/// Best-effort one-click fixes. Only performs actions that are:
/// - local-only
/// - safe to attempt automatically
/// - reversible by the user
///
/// This does NOT mutate config.json (except indirectly if OS CA installers do so).
pub fn apply_one_click_fixes(config: &Config) -> Vec<FixOutcome> {
    let mut out = Vec::new();
    let mode = config.mode_kind().ok();

    // Fix 1: ensure CA files exist + attempt to install into OS trust store
    // when needed (apps_script / direct; full mode doesn't use MITM).
    if mode != Some(Mode::Full) {
        let base = crate::data_dir::data_dir();
        match MitmCertManager::new_in(&base) {
            Ok(_) => {
                let ca_path = base.join(CA_CERT_FILE);
                if is_ca_trusted(&ca_path) {
                    out.push(FixOutcome {
                        id: "fix_install_ca",
                        ok: true,
                        detail: "CA already appears trusted; nothing to do.".into(),
                    });
                } else {
                    match install_ca(&ca_path) {
                        Ok(()) => out.push(FixOutcome {
                            id: "fix_install_ca",
                            ok: true,
                            detail: format!(
                                "Installed CA into OS trust store: {}",
                                ca_path.display()
                            ),
                        }),
                        Err(e) => out.push(FixOutcome {
                            id: "fix_install_ca",
                            ok: false,
                            detail: format!(
                                "Failed to install CA automatically (may require admin): {e}"
                            ),
                        }),
                    }
                }
            }
            Err(e) => out.push(FixOutcome {
                id: "fix_init_ca",
                ok: false,
                detail: format!("Failed to initialize CA files: {e}"),
            }),
        }
    }

    out
}

/// Run doctor, apply one-click fixes, then run doctor again.
pub async fn run_with_fixes(config: &Config) -> (DoctorReport, Vec<FixOutcome>, DoctorReport) {
    let before = run(config).await;
    let fixes = apply_one_click_fixes(config);
    let after = run(config).await;
    (before, fixes, after)
}
