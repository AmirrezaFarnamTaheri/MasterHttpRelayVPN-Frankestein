use eframe::egui;
use mhrv_jni::readiness;

use crate::ui_style::{
    help_muted, ACCENT, ACCENT_MINT, ACCENT_WARM, ERR_RED, OK_GREEN, TEXT_LABEL, TEXT_MAIN,
    TEXT_MUTED,
};
use crate::{AccountGroupForm, FormState, UiTab};

pub(crate) fn mode_summary(mode: &str) -> (&'static str, &'static str, &'static str, &'static str) {
    match mode {
        "full" => (
            "Full tunnel",
            "Apps Script tunnel channel plus your tunnel-node server.",
            "Needs Apps Script full deployment, tunnel-node VPS, and server logs for verification.",
            "No local MITM CA; verify by checking that browsing exits through the tunnel-node IP.",
        ),
        "vercel_edge" => (
            "Serverless JSON",
            "Native no-VPS JSON/base64 fetch relay hosted on Vercel or Netlify.",
            "Deploy tools/vercel-json-relay or tools/netlify-json-relay, set AUTH_KEY, paste Base URL.",
            "Needs local MITM CA for HTTPS clients, same as Apps Script mode.",
        ),
        "direct" | "google_only" => (
            "Direct fronting",
            "SNI-rewrite path only: Google edge plus configured fronting groups.",
            "No Apps Script credentials; useful for bootstrap or limited CDN-fronted targets.",
            "Not a full tunnel; unmatched traffic goes raw/direct.",
        ),
        _ => (
            "Apps Script",
            "Classic no-VPS relay through your own Google Apps Script deployments.",
            "Deploy Code.gs or CodeCloudflareWorker.gs, then add account groups with AUTH_KEY and IDs.",
            "Needs local MITM CA for HTTPS clients; quotas scale with deployment/account pools.",
        ),
    }
}

pub(crate) fn mode_summary_panel(ui: &mut egui::Ui, mode: &str) {
    let (title, path, setup, trust) = mode_summary(mode);
    ui.add_space(6.0);
    ui.separator();
    egui::Grid::new("mode_summary_grid")
        .num_columns(2)
        .spacing([12.0, 6.0])
        .show(ui, |ui| {
            ui.label(egui::RichText::new("Selected").color(ACCENT).strong());
            ui.label(egui::RichText::new(title).strong());
            ui.end_row();
            ui.label(egui::RichText::new("Path").color(egui::Color32::from_gray(170)));
            help_muted(ui, path);
            ui.end_row();
            ui.label(egui::RichText::new("Setup").color(egui::Color32::from_gray(170)));
            help_muted(ui, setup);
            ui.end_row();
            ui.label(egui::RichText::new("Trust").color(egui::Color32::from_gray(170)));
            help_muted(ui, trust);
            ui.end_row();
        });
}

struct ModeReadinessItem {
    id: readiness::ReadinessId,
    label: String,
    ok: bool,
    detail: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct DesktopRepairAction {
    label: &'static str,
    target: &'static str,
    anchor: Option<&'static str>,
    tab: UiTab,
    hint: &'static str,
}

struct ModeDashboard {
    title: &'static str,
    subtitle: &'static str,
    color: egui::Color32,
    chips: Vec<String>,
    capabilities: Vec<&'static str>,
    requirements: Vec<ModeReadinessItem>,
    cautions: Vec<&'static str>,
}

fn mode_dashboard(form: &FormState) -> ModeDashboard {
    let enabled_groups: Vec<&AccountGroupForm> =
        form.account_groups.iter().filter(|g| g.enabled).collect();
    let deployment_count: usize = enabled_groups
        .iter()
        .flat_map(|g| g.script_ids.lines().flat_map(|line| line.split(',')))
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .count();
    let groups_with_auth = enabled_groups
        .iter()
        .filter(|g| !g.auth_key.trim().is_empty())
        .count();
    let listen_ready =
        form.listen_port.trim().parse::<u16>().is_ok() && !form.listen_host.trim().is_empty();
    let relay_path_ready = form.vercel_relay_path.trim().starts_with('/');
    let serverless_base_ready = form.vercel_base_url.trim().starts_with("http://")
        || form.vercel_base_url.trim().starts_with("https://");
    let mut dashboard = match form.mode.as_str() {
        "vercel_edge" => ModeDashboard {
            title: "Serverless JSON",
            subtitle: "No-VPS JSON/base64 fetch relay on Vercel or Netlify.",
            color: ACCENT_MINT,
            chips: vec![
                "needs local CA".into(),
                "no Google quota pool".into(),
                format!("max body {} MiB", form.vercel_max_body_mb.max(1)),
            ],
            capabilities: vec![
                "HTTPS browsing through the local MITM proxy",
                "Vercel/Netlify deployment with a single AUTH_KEY",
                "Optional JSON batch envelope when the helper supports it",
                "Same local HTTP/SOCKS listener model as Apps Script",
            ],
            requirements: vec![
                ModeReadinessItem {
                    id: readiness::VERCEL_BASE_URL,
                    label: "Relay origin".into(),
                    ok: serverless_base_ready,
                    detail: if serverless_base_ready {
                        form.vercel_base_url.trim().to_string()
                    } else {
                        "Paste https://your-project.vercel.app or Netlify site URL".into()
                    },
                },
                ModeReadinessItem {
                    id: readiness::VERCEL_RELAY_PATH,
                    label: "Relay path".into(),
                    ok: relay_path_ready,
                    detail: if relay_path_ready {
                        form.vercel_relay_path.trim().to_string()
                    } else {
                        "Path must start with /, usually /api/api".into()
                    },
                },
                ModeReadinessItem {
                    id: readiness::VERCEL_AUTH_KEY,
                    label: "AUTH_KEY".into(),
                    ok: !form.vercel_auth_key.trim().is_empty(),
                    detail: if form.vercel_auth_key.trim().is_empty() {
                        "Must match the serverless deployment environment variable".into()
                    } else {
                        "Configured".into()
                    },
                },
                ModeReadinessItem {
                    id: readiness::LOCAL_LISTENER,
                    label: "Local listener".into(),
                    ok: listen_ready,
                    detail: format!("{}:{}", form.listen_host.trim(), form.listen_port.trim()),
                },
            ],
            cautions: vec![
                "Platform protection pages must not sit in front of the relay endpoint.",
                "This mode still needs local CA trust for HTTPS clients.",
            ],
        },
        "direct" | "google_only" => ModeDashboard {
            title: "Direct fronting",
            subtitle: "No-relay SNI rewrite for Google edge and configured fronting groups.",
            color: ACCENT_WARM,
            chips: vec![
                "no relay credentials".into(),
                "bootstrap friendly".into(),
                "limited coverage".into(),
            ],
            capabilities: vec![
                "Reach script.google.com to deploy or repair Apps Script",
                "Use tested Google/fronting group edge targets without relay quota",
                "Bypass relay auth/key setup for narrow bootstrap workflows",
                "Keep HTTP/SOCKS listeners available for proxy-aware clients",
            ],
            requirements: vec![
                ModeReadinessItem {
                    id: readiness::DIRECT_GOOGLE_IP,
                    label: "Google edge IP".into(),
                    ok: form.google_ip.trim().is_empty()
                        || form.google_ip.trim().parse::<std::net::IpAddr>().is_ok(),
                    detail: if form.google_ip.trim().is_empty() {
                        "Auto-detected on start when possible".into()
                    } else {
                        form.google_ip.trim().to_string()
                    },
                },
                ModeReadinessItem {
                    id: readiness::DIRECT_FRONT_DOMAIN,
                    label: "Front SNI".into(),
                    ok: form.front_domain.trim().is_empty()
                        || form.front_domain.trim().parse::<std::net::IpAddr>().is_err(),
                    detail: if form.front_domain.trim().is_empty() {
                        "Defaults to www.google.com on start".into()
                    } else if form.front_domain.trim().parse::<std::net::IpAddr>().is_ok() {
                        "Use a hostname for SNI, not an IP address".into()
                    } else {
                        form.front_domain.trim().to_string()
                    },
                },
                ModeReadinessItem {
                    id: readiness::LOCAL_LISTENER,
                    label: "Local listener".into(),
                    ok: listen_ready,
                    detail: format!("{}:{}", form.listen_host.trim(), form.listen_port.trim()),
                },
            ],
            cautions: vec![
                "This is not a full tunnel; unmatched traffic is not carried by a relay.",
                "Use Apps Script, Serverless JSON, or Full when you need broad browsing coverage.",
            ],
        },
        "full" => ModeDashboard {
            title: "Full tunnel",
            subtitle: "Apps Script control channel plus your remote tunnel-node server.",
            color: egui::Color32::from_rgb(170, 145, 225),
            chips: vec![
                "no local CA".into(),
                "needs tunnel-node".into(),
                format!("{deployment_count} deployment ID(s)"),
            ],
            capabilities: vec![
                "Carry TCP/UDP-style tunnel batches without local HTTPS interception",
                "Use tunnel-node health/logs as the main remote verification surface",
                "Tune full batch timeout, timeout strikes, cooldown, and QUIC blocking",
                "Keep Apps Script account groups for the full-mode control channel",
            ],
            requirements: vec![
                ModeReadinessItem {
                    id: readiness::ACCOUNT_GROUPS_ENABLED,
                    label: "Enabled groups".into(),
                    ok: !enabled_groups.is_empty(),
                    detail: format!("{} enabled group(s)", enabled_groups.len()),
                },
                ModeReadinessItem {
                    id: readiness::ACCOUNT_GROUPS_SCRIPT_IDS,
                    label: "Deployment IDs".into(),
                    ok: deployment_count > 0,
                    detail: format!("{deployment_count} deployment ID(s)"),
                },
                ModeReadinessItem {
                    id: readiness::ACCOUNT_GROUPS_AUTH_KEY,
                    label: "Group AUTH_KEY".into(),
                    ok: groups_with_auth == enabled_groups.len() && !enabled_groups.is_empty(),
                    detail: format!("{groups_with_auth}/{} enabled group(s)", enabled_groups.len()),
                },
                ModeReadinessItem {
                    id: readiness::LOCAL_LISTENER,
                    label: "Local listener".into(),
                    ok: listen_ready,
                    detail: format!("{}:{}", form.listen_host.trim(), form.listen_port.trim()),
                },
            ],
            cautions: vec![
                "Verify remote tunnel-node auth, version, and limits outside the local CA flow.",
                "Full mode can be healthy even when local CA checks are irrelevant.",
            ],
        },
        _ => ModeDashboard {
            title: "Apps Script",
            subtitle: "Classic no-VPS relay through your own Google Apps Script deployments.",
            color: ACCENT,
            chips: vec![
                "needs local CA".into(),
                format!("{} enabled group(s)", enabled_groups.len()),
                format!("{deployment_count} deployment ID(s)"),
            ],
            capabilities: vec![
                "Full local HTTPS browsing through the Apps Script relay",
                "Multiple deployment IDs per Google account for fallback",
                "Multiple account groups with weights for quota resilience",
                "Optional Cloudflare Worker exit via the Apps Script helper",
            ],
            requirements: vec![
                ModeReadinessItem {
                    id: readiness::ACCOUNT_GROUPS_ENABLED,
                    label: "Enabled groups".into(),
                    ok: !enabled_groups.is_empty(),
                    detail: format!("{} enabled group(s)", enabled_groups.len()),
                },
                ModeReadinessItem {
                    id: readiness::ACCOUNT_GROUPS_SCRIPT_IDS,
                    label: "Deployment IDs".into(),
                    ok: deployment_count > 0,
                    detail: format!("{deployment_count} deployment ID(s)"),
                },
                ModeReadinessItem {
                    id: readiness::ACCOUNT_GROUPS_AUTH_KEY,
                    label: "Group AUTH_KEY".into(),
                    ok: groups_with_auth == enabled_groups.len() && !enabled_groups.is_empty(),
                    detail: format!("{groups_with_auth}/{} enabled group(s)", enabled_groups.len()),
                },
                ModeReadinessItem {
                    id: readiness::LOCAL_LISTENER,
                    label: "Local listener".into(),
                    ok: listen_ready,
                    detail: format!("{}:{}", form.listen_host.trim(), form.listen_port.trim()),
                },
            ],
            cautions: vec![
                "Install and trust the local CA before using HTTPS clients.",
                "Quota pressure is solved first by more deployments/accounts, then by careful tuning.",
            ],
        },
    };
    append_operational_readiness(form, &mut dashboard.requirements);
    dashboard
}

fn append_operational_readiness(form: &FormState, requirements: &mut Vec<ModeReadinessItem>) {
    if matches!(form.mode.as_str(), "full") {
        requirements.push(ModeReadinessItem {
            id: readiness::FULL_CODEFULL_DEPLOYMENT,
            label: "CodeFull deployment".into(),
            ok: false,
            detail: "Verify every full-mode deployment uses CodeFull.gs".into(),
        });
        requirements.push(ModeReadinessItem {
            id: readiness::FULL_TUNNEL_NODE_URL,
            label: "Tunnel-node URL".into(),
            ok: false,
            detail: "CodeFull.gs must point at the public tunnel-node origin".into(),
        });
        requirements.push(ModeReadinessItem {
            id: readiness::FULL_TUNNEL_AUTH,
            label: "Tunnel auth".into(),
            ok: false,
            detail: "TUNNEL_AUTH_KEY must match between CodeFull.gs and tunnel-node".into(),
        });
        requirements.push(ModeReadinessItem {
            id: readiness::FULL_UDP_SUPPORT,
            label: "UDP/SOCKS5 path".into(),
            ok: !form.socks5_port.trim().is_empty(),
            detail: if form.socks5_port.trim().is_empty() {
                "Set SOCKS5 port if clients need UDP ASSOCIATE".into()
            } else {
                format!("SOCKS5 listener configured on {}", form.socks5_port.trim())
            },
        });
        requirements.push(ModeReadinessItem {
            id: readiness::FULL_TUNNEL_HEALTH,
            label: "Tunnel health".into(),
            ok: false,
            detail: "Check /healthz, tunnel-node logs, and public-IP verification".into(),
        });
    }

    if !matches!(form.mode.as_str(), "full") {
        requirements.push(ModeReadinessItem {
            id: readiness::CA_TRUST,
            label: "Local CA trust".into(),
            ok: false,
            detail: "Install and trust the generated CA before routing HTTPS clients".into(),
        });
        requirements.push(ModeReadinessItem {
            id: readiness::ANDROID_APP_CA_TRUST,
            label: "Android app CA trust".into(),
            ok: false,
            detail: "Android apps may ignore user CAs unless they opt in".into(),
        });
    }

    let host = form.listen_host.trim();
    if matches!(host, "0.0.0.0" | "::") {
        let has_token = form
            .lan_token
            .as_deref()
            .map(|token| !token.trim().is_empty())
            .unwrap_or(false);
        let allowlist_count = form
            .lan_allowlist
            .as_ref()
            .map(|allowlist| {
                allowlist
                    .iter()
                    .filter(|entry| !entry.trim().is_empty())
                    .count()
            })
            .unwrap_or(0);
        let has_allowlist = allowlist_count > 0;

        requirements.push(ModeReadinessItem {
            id: readiness::LAN_EXPOSURE,
            label: "LAN exposure".into(),
            ok: false,
            detail: format!("Proxy is reachable on {host} when firewall rules allow"),
        });
        requirements.push(ModeReadinessItem {
            id: readiness::LAN_TOKEN,
            label: "LAN access control".into(),
            ok: has_token || has_allowlist,
            detail: if has_token {
                "HTTP/CONNECT token configured".into()
            } else if has_allowlist {
                format!("{allowlist_count} allowlist entrie(s) configured")
            } else {
                "Set lan_token or lan_allowlist before sharing on LAN".into()
            },
        });
        if !form.socks5_port.trim().is_empty() {
            requirements.push(ModeReadinessItem {
                id: readiness::LAN_ALLOWLIST,
                label: "SOCKS5 LAN allowlist".into(),
                ok: has_allowlist,
                detail: if has_allowlist {
                    format!("{allowlist_count} allowlist entrie(s) configured")
                } else {
                    "SOCKS5 has no token header; add lan_allowlist".into()
                },
            });
        }
    }
}

fn repair_tab_for_target(target: &str) -> UiTab {
    if target.starts_with("advanced.") || target.starts_with("setup.account_groups") {
        UiTab::Advanced
    } else if target.starts_with("setup.direct")
        || target.starts_with("setup.local_listener")
        || target.starts_with("network.")
    {
        UiTab::Network
    } else if target.starts_with("help.") {
        UiTab::Help
    } else if target.starts_with("setup.ca_trust") {
        UiTab::Trust
    } else {
        UiTab::Setup
    }
}

fn repair_hint_for_tab(tab: UiTab) -> &'static str {
    match tab {
        UiTab::Setup => "Jump to the setup fields for this blocker.",
        UiTab::Network => "Jump to network/listener fields for this blocker.",
        UiTab::Advanced => "Jump to advanced configuration fields for this blocker.",
        UiTab::Monitor => "Jump to monitor diagnostics for this blocker.",
        UiTab::Trust => "Jump to certificate, signing, and support-bundle trust state.",
        UiTab::Help => "Jump to help and documentation for this blocker.",
    }
}

fn desktop_repair_action(item: &ModeReadinessItem) -> Option<DesktopRepairAction> {
    if item.ok {
        return None;
    }
    let repair = readiness::repair_for_id(item.id)?;
    let tab = repair_tab_for_target(repair.target);
    let anchor = readiness::repair_anchor_for_target(repair.target).map(|anchor| anchor.desktop);
    Some(DesktopRepairAction {
        label: repair.label,
        target: repair.target,
        anchor,
        tab,
        hint: repair_hint_for_tab(tab),
    })
}

pub(crate) fn mode_dashboard_panel(
    ui: &mut egui::Ui,
    form: &FormState,
    running: bool,
    dirty: bool,
    active_tab: &mut UiTab,
) {
    let dashboard = mode_dashboard(form);
    egui::Frame::none()
        .fill(dashboard.color.linear_multiply(0.1))
        .stroke(egui::Stroke::new(
            1.0,
            dashboard.color.linear_multiply(0.46),
        ))
        .rounding(11.0)
        .inner_margin(egui::Margin::symmetric(15.0, 13.0))
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(
                    egui::RichText::new(dashboard.title)
                        .strong()
                        .size(16.0)
                        .color(dashboard.color.linear_multiply(1.16)),
                );
                info_chip(
                    ui,
                    if running { "running" } else { "stopped" },
                    if running { OK_GREEN } else { ERR_RED },
                );
                if dirty {
                    info_chip(ui, "unsaved edits", ACCENT_WARM);
                }
                for chip in &dashboard.chips {
                    info_chip(ui, chip, dashboard.color);
                }
            });
            ui.add_space(5.0);
            help_muted(ui, dashboard.subtitle);

            ui.add_space(10.0);
            ui.columns(2, |columns| {
                columns[0].label(
                    egui::RichText::new("Readiness")
                        .strong()
                        .color(TEXT_LABEL)
                        .size(13.5),
                );
                columns[0].add_space(5.0);
                for item in &dashboard.requirements {
                    let warning = readiness::repair_for_id(item.id)
                        .map(|repair| {
                            repair.target.starts_with("network.")
                                || repair.target.starts_with("help.")
                                || repair.target.starts_with("setup.ca_trust")
                        })
                        .unwrap_or(false);
                    let status = if item.ok {
                        "ready"
                    } else if warning {
                        "check"
                    } else {
                        "missing"
                    };
                    let status_color = if item.ok {
                        OK_GREEN
                    } else if warning {
                        ACCENT_WARM
                    } else {
                        ERR_RED
                    };
                    columns[0].horizontal_wrapped(|ui| {
                        ui.label(
                            egui::RichText::new(status)
                                .strong()
                                .size(11.2)
                                .color(status_color),
                        );
                        ui.label(egui::RichText::new(&item.label).strong().color(TEXT_MAIN));
                        ui.small(egui::RichText::new(&item.detail).color(TEXT_MUTED));
                        if !item.ok {
                            ui.small(
                                egui::RichText::new(item.id)
                                    .monospace()
                                    .color(status_color.linear_multiply(1.08)),
                            );
                            if let Some(repair) = desktop_repair_action(item) {
                                if ui
                                    .small_button(repair.label)
                                    .on_hover_text(match repair.anchor {
                                        Some(anchor) => format!(
                                            "{}\nOpen: {}\nTarget: {}",
                                            repair.hint, anchor, repair.target
                                        ),
                                        None => format!("{} ({})", repair.hint, repair.target),
                                    })
                                    .clicked()
                                {
                                    *active_tab = repair.tab;
                                }
                            }
                        }
                    });
                }

                columns[1].label(
                    egui::RichText::new("Capabilities")
                        .strong()
                        .color(TEXT_LABEL)
                        .size(13.5),
                );
                columns[1].add_space(5.0);
                for capability in &dashboard.capabilities {
                    columns[1].small(egui::RichText::new(*capability).color(TEXT_MUTED));
                }
                if !dashboard.cautions.is_empty() {
                    columns[1].add_space(7.0);
                    for caution in &dashboard.cautions {
                        columns[1].small(
                            egui::RichText::new(*caution).color(ACCENT_WARM.linear_multiply(1.08)),
                        );
                    }
                }
            });
        });
}

pub(crate) fn ghost_action(label: &'static str) -> egui::Button<'static> {
    egui::Button::new(egui::RichText::new(label).color(TEXT_MAIN))
        .fill(egui::Color32::from_rgb(30, 34, 38))
        .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(70, 78, 86)))
        .min_size(egui::vec2(92.0, 30.0))
        .rounding(6.0)
}

pub(crate) fn info_chip(ui: &mut egui::Ui, label: impl Into<String>, color: egui::Color32) {
    egui::Frame::none()
        .fill(color.linear_multiply(0.22))
        .stroke(egui::Stroke::new(1.0, color.linear_multiply(0.58)))
        .rounding(14.0)
        .shadow(egui::Shadow {
            offset: egui::vec2(0.0, 1.0),
            blur: 6.0,
            spread: 0.0,
            color: egui::Color32::from_black_alpha(36),
        })
        .inner_margin(egui::Margin::symmetric(10.0, 4.0))
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new(label.into())
                    .size(11.6)
                    .strong()
                    .color(color.linear_multiply(1.22)),
            );
        });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::load_form;

    #[test]
    fn desktop_dashboard_uses_shared_readiness_ids() {
        let (mut form, _) = load_form();
        form.mode = "apps_script".into();
        form.listen_host = "127.0.0.1".into();
        form.listen_port = "8085".into();
        form.account_groups = vec![AccountGroupForm {
            label: "primary".into(),
            enabled: true,
            weight: 1,
            auth_key: "test-auth-key-please-change-32chars".into(),
            script_ids: "AKfycb_primary".into(),
            show_auth_key: false,
        }];

        let dashboard = mode_dashboard(&form);
        let ids: Vec<_> = dashboard.requirements.iter().map(|item| item.id).collect();
        assert_eq!(
            &ids[..4],
            vec![
                readiness::ACCOUNT_GROUPS_ENABLED,
                readiness::ACCOUNT_GROUPS_SCRIPT_IDS,
                readiness::ACCOUNT_GROUPS_AUTH_KEY,
                readiness::LOCAL_LISTENER,
            ]
        );
        assert!(ids.contains(&readiness::CA_TRUST));
        assert!(dashboard
            .requirements
            .iter()
            .filter(|item| !matches!(
                item.id,
                readiness::CA_TRUST | readiness::ANDROID_APP_CA_TRUST
            ))
            .all(|item| item.ok));
    }

    #[test]
    fn desktop_direct_dashboard_matches_auto_default_readiness() {
        let (mut form, _) = load_form();
        form.mode = "direct".into();
        form.google_ip.clear();
        form.front_domain.clear();
        form.listen_host = "127.0.0.1".into();
        form.listen_port = "8085".into();

        let dashboard = mode_dashboard(&form);
        let google = dashboard
            .requirements
            .iter()
            .find(|item| item.id == readiness::DIRECT_GOOGLE_IP)
            .expect("google ip readiness row");
        let front = dashboard
            .requirements
            .iter()
            .find(|item| item.id == readiness::DIRECT_FRONT_DOMAIN)
            .expect("front domain readiness row");

        assert!(google.ok);
        assert!(front.ok);
        assert!(google.detail.contains("Auto-detected"));
        assert!(front.detail.contains("Defaults"));
    }

    #[test]
    fn desktop_repair_actions_route_to_expected_tabs() {
        let (mut form, _) = load_form();
        form.mode = "apps_script".into();
        form.account_groups.clear();
        let dashboard = mode_dashboard(&form);
        let repair = desktop_repair_action(&dashboard.requirements[0])
            .expect("missing account group should have repair action");
        assert_eq!(repair.target, "setup.account_groups");
        assert_eq!(
            repair.anchor,
            Some("Advanced -> Multi-account pools -> Add group")
        );
        assert_eq!(repair.tab, UiTab::Advanced);

        form.mode = "vercel_edge".into();
        form.vercel_base_url.clear();
        let dashboard = mode_dashboard(&form);
        let repair = desktop_repair_action(
            dashboard
                .requirements
                .iter()
                .find(|item| item.id == readiness::VERCEL_BASE_URL)
                .expect("serverless base row"),
        )
        .expect("missing serverless base should have repair action");
        assert_eq!(repair.tab, UiTab::Setup);
        assert_eq!(
            repair.anchor,
            Some("Setup -> Serverless JSON relay -> Base URL")
        );

        form.mode = "direct".into();
        form.google_ip = "not-an-ip".into();
        let dashboard = mode_dashboard(&form);
        let repair = desktop_repair_action(
            dashboard
                .requirements
                .iter()
                .find(|item| item.id == readiness::DIRECT_GOOGLE_IP)
                .expect("direct IP row"),
        )
        .expect("bad direct IP should have repair action");
        assert_eq!(repair.tab, UiTab::Network);
        assert_eq!(repair.anchor, Some("Network -> Google IP"));

        form.listen_host = "0.0.0.0".into();
        form.socks5_port = "8086".into();
        form.lan_allowlist = None;
        let dashboard = mode_dashboard(&form);
        let repair = desktop_repair_action(
            dashboard
                .requirements
                .iter()
                .find(|item| item.id == readiness::LAN_ALLOWLIST)
                .expect("lan allowlist row"),
        )
        .expect("lan allowlist warning should have repair action");
        assert_eq!(repair.tab, UiTab::Network);
        assert_eq!(
            repair.anchor,
            Some("Network -> Sharing and per-app routing -> Allowed IPs")
        );

        form.listen_host = "127.0.0.1".into();
        form.mode = "apps_script".into();
        let dashboard = mode_dashboard(&form);
        let repair = desktop_repair_action(
            dashboard
                .requirements
                .iter()
                .find(|item| item.id == readiness::CA_TRUST)
                .expect("ca trust row"),
        )
        .expect("ca trust warning should have repair action");
        assert_eq!(repair.tab, UiTab::Trust);

        form.mode = "full".into();
        form.socks5_port.clear();
        let dashboard = mode_dashboard(&form);
        let repair = desktop_repair_action(
            dashboard
                .requirements
                .iter()
                .find(|item| item.id == readiness::FULL_TUNNEL_HEALTH)
                .expect("full tunnel health row"),
        )
        .expect("full tunnel health warning should have repair action");
        assert_eq!(repair.tab, UiTab::Help);
        assert_eq!(
            repair.anchor,
            Some("Help & docs -> Full tunnel -> health/details and IP-check smoke test")
        );
        let udp = dashboard
            .requirements
            .iter()
            .find(|item| item.id == readiness::FULL_UDP_SUPPORT)
            .expect("full UDP row");
        assert!(!udp.ok);
    }
}
