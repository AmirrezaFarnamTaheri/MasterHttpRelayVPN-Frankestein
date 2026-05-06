use std::sync::mpsc::{Receiver, TryRecvError};
use std::time::Duration;

use eframe::egui;
use mhrv_jni::xhttp_cloud_deploy::{self, XhttpDeployWorkerMsg};

use crate::ui_style::{
    form_row, help_muted, help_subheading, ACCENT, CARD_STROKE, TEXT_LABEL, TEXT_MUTED,
};

const NETLIFY_XHTTP_CANDIDATES: &[&str] = &[
    "kubernetes.io",
    "helm.sh",
    "letsencrypt.org",
    "docs.helm.sh",
    "kubectl.docs.kubernetes.io",
    "blog.helm.sh",
    "kind.sigs.k8s.io",
    "cluster-api.sigs.k8s.io",
    "krew.sigs.k8s.io",
    "gateway-api.sigs.k8s.io",
    "scheduler-plugins.sigs.k8s.io",
    "kustomize.sigs.k8s.io",
    "image-builder.sigs.k8s.io",
];

const VERCEL_XHTTP_CANDIDATES: &[&str] = &[
    "community.vercel.com",
    "analytics.vercel.com",
    "botid.vercel.com",
    "blog.vercel.com",
    "app.vercel.com",
    "api.vercel.com",
    "ai.vercel.com",
    "cursor.com",
    "nextjs.org",
    "react.dev",
];

#[derive(Clone)]
pub(crate) struct XhttpGeneratorForm {
    pub(crate) platform: String,
    pub(crate) uuid: String,
    pub(crate) relay_host: String,
    pub(crate) target_domain: String,
    pub(crate) path: String,
    pub(crate) name_prefix: String,
    pub(crate) allow_insecure: bool,
    pub(crate) candidates: String,
    pub(crate) output: String,
    pub(crate) deploy_notes: String,
    /// `manual` | `vercel_api` | `netlify_api`
    pub(crate) deploy_tab: String,
    pub(crate) deploy_api_token: String,
    pub(crate) show_deploy_api_token: bool,
    pub(crate) randomize_bundle_names: bool,
    pub(crate) deploy_log: String,
    pub(crate) deploy_last_host: String,
}

impl Default for XhttpGeneratorForm {
    fn default() -> Self {
        Self {
            platform: "netlify".into(),
            uuid: String::new(),
            relay_host: String::new(),
            target_domain: String::new(),
            path: "/p4r34m".into(),
            name_prefix: "netlify-xhttp".into(),
            allow_insecure: true,
            candidates: NETLIFY_XHTTP_CANDIDATES.join("\n"),
            output: String::new(),
            deploy_notes: String::new(),
            deploy_tab: "manual".into(),
            deploy_api_token: String::new(),
            show_deploy_api_token: false,
            randomize_bundle_names: false,
            deploy_log: String::new(),
            deploy_last_host: String::new(),
        }
    }
}

#[derive(Default)]
pub(crate) struct XhttpDeployPipe {
    rx: Option<Receiver<XhttpDeployWorkerMsg>>,
    busy: bool,
}

pub(crate) fn xhttp_platform_defaults(
    platform: &str,
) -> (
    &'static str,
    &'static str,
    &'static str,
    &'static [&'static str],
) {
    match platform {
        "vercel" => (
            "your-project.vercel.app",
            "/yourpath",
            "vercel-xhttp",
            VERCEL_XHTTP_CANDIDATES,
        ),
        _ => (
            "your-site.netlify.app",
            "/p4r34m",
            "netlify-xhttp",
            NETLIFY_XHTTP_CANDIDATES,
        ),
    }
}

fn normalize_xhttp_host(value: &str) -> String {
    let trimmed = value.trim();
    let without_scheme = trimmed
        .strip_prefix("https://")
        .or_else(|| trimmed.strip_prefix("http://"))
        .unwrap_or(trimmed);
    without_scheme
        .split('/')
        .next()
        .unwrap_or_default()
        .trim()
        .to_string()
}

fn normalize_xhttp_path(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        "/p4r34m".into()
    } else if trimmed.starts_with('/') {
        trimmed.to_string()
    } else {
        format!("/{trimmed}")
    }
}

fn encode_uri_component(input: &str) -> String {
    let mut out = String::new();
    for b in input.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

pub(crate) fn generate_xhttp_vless_links(form: &XhttpGeneratorForm) -> Result<String, String> {
    let uuid = form.uuid.trim();
    if uuid.is_empty() {
        return Err("Paste the UUID from your real Xray/V2Ray backend first.".into());
    }
    let relay_host = normalize_xhttp_host(&form.relay_host);
    if relay_host.is_empty() {
        return Err("Paste your deployed Vercel/Netlify relay hostname first.".into());
    }
    let path = normalize_xhttp_path(&form.path);
    let encoded_path = encode_uri_component(&path);
    let allow = if form.allow_insecure { "1" } else { "0" };
    let prefix = form.name_prefix.trim();
    let prefix = if prefix.is_empty() { "xhttp" } else { prefix };
    let mut links = Vec::new();
    for raw in form.candidates.lines().flat_map(|line| line.split(',')) {
        let candidate = normalize_xhttp_host(raw);
        if candidate.is_empty()
            || links
                .iter()
                .any(|link: &String| link.contains(&format!("@{candidate}:443?")))
        {
            continue;
        }
        let tag = encode_uri_component(&format!("{prefix}-{candidate}"));
        links.push(format!(
            "vless://{uuid}@{candidate}:443?mode=auto&path={encoded_path}&security=tls&encryption=none&insecure={allow}&host={relay_host}&type=xhttp&allowInsecure={allow}&sni={candidate}&alpn=h2%2Chttp%2F1.1&fp=chrome#{tag}"
        ));
    }
    if links.is_empty() {
        Err("Add at least one Address/SNI candidate.".into())
    } else {
        Ok(links.join("\n"))
    }
}

pub(crate) fn generate_xhttp_deploy_notes(form: &XhttpGeneratorForm) -> Result<String, String> {
    let target = form.target_domain.trim();
    if target.is_empty() {
        return Err("Paste TARGET_DOMAIN first, for example https://xray.example.com:2096.".into());
    }
    if !(target.starts_with("https://") || target.starts_with("http://")) {
        return Err("TARGET_DOMAIN must include http:// or https:// and any required port.".into());
    }
    let notes = if form.platform == "vercel" {
        format!(
            "Vercel XHTTP helper\n\nManual / CLI:\n1. Open tools/vercel-xhttp-relay.\n2. Deploy with: vercel --prod\n3. In Vercel project settings, add environment variable:\n   TARGET_DOMAIN={target}\n4. Redeploy after setting the variable.\n5. Disable Deployment Protection for this relay project if Vercel put a login/protection page in front.\n6. Put the produced *.vercel.app hostname into the generator Relay Host field.\n7. Generate VLESS links with the Vercel preset and test one candidate at a time.\n\nOptional: Setup tab -> XHTTP -> Deploy assistant -> Vercel API deploys the same Edge relay from this app (token stays in RAM until exit).\n\nSee docs/vercel-xhttp-relay.md for dashboard import."
        )
    } else {
        format!(
            "Netlify XHTTP helper\n\nManual / CLI:\n1. Open tools/netlify-xhttp-relay.\n2. Deploy with: netlify deploy --prod\n3. In Netlify site settings, add environment variable:\n   TARGET_DOMAIN={target}\n4. Redeploy after setting the variable.\n5. Confirm Edge Function logs show relay activity for /p4r34m.\n6. Put the produced *.netlify.app hostname into the generator Relay Host field.\n7. Generate VLESS links with the Netlify preset and test one candidate at a time.\n\nOptional: Deploy assistant → Netlify API uploads a ZIP with the backend URL baked into the edge script (no dashboard env step).\n\nDashboard flow: import tools/netlify-xhttp-relay in Netlify, publish directory public."
        )
    };
    Ok(notes)
}

pub(crate) fn xhttp_vless_generator(
    ui: &mut egui::Ui,
    form: &mut XhttpGeneratorForm,
    deploy_pipe: &mut XhttpDeployPipe,
) -> Option<String> {
    let mut toast = None;
    help_muted(
        ui,
        "Generate external Xray/V2Ray VLESS + XHTTP links in-app. Native mhrv-f modes are unchanged. Provider API tokens for cloud deploy are kept in RAM only (never saved to config.json).",
    );
    ui.horizontal_wrapped(|ui| {
        ui.label(egui::RichText::new("Preset").color(egui::Color32::from_gray(200)));
        egui::ComboBox::from_id_source("xhttp_generator_platform")
            .selected_text(if form.platform == "vercel" {
                "Vercel XHTTP"
            } else {
                "Netlify XHTTP"
            })
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut form.platform, "netlify".into(), "Netlify XHTTP");
                ui.selectable_value(&mut form.platform, "vercel".into(), "Vercel XHTTP");
            });
        if ui
            .small_button("load preset")
            .on_hover_text(
                "Reset path, name prefix, and Address/SNI candidates for the selected platform.",
            )
            .clicked()
        {
            let (_, path, prefix, candidates) = xhttp_platform_defaults(&form.platform);
            form.path = path.into();
            form.name_prefix = prefix.into();
            form.candidates = candidates.join("\n");
            form.output.clear();
            toast = Some("XHTTP preset loaded.".into());
        }
    });
    let (host_hint, _, _, _) = xhttp_platform_defaults(&form.platform);
    form_row(
        ui,
        "UUID",
        Some("The UUID configured on your real backend Xray/V2Ray VLESS inbound."),
        |ui| {
            ui.add(egui::TextEdit::singleline(&mut form.uuid).desired_width(f32::INFINITY));
        },
    );
    form_row(
        ui,
        "Relay Host",
        Some("Your deployed Vercel or Netlify hostname. This becomes the XHTTP Host value."),
        |ui| {
            ui.add(
                egui::TextEdit::singleline(&mut form.relay_host)
                    .hint_text(host_hint)
                    .desired_width(f32::INFINITY),
            );
        },
    );
    ui.horizontal(|ui| {
        ui.add_sized(
            [120.0, 20.0],
            egui::Label::new(egui::RichText::new("XHTTP").color(egui::Color32::from_gray(200))),
        );
        ui.label(egui::RichText::new("Path").small());
        ui.add(egui::TextEdit::singleline(&mut form.path).desired_width(150.0));
        ui.add_space(8.0);
        ui.label(egui::RichText::new("Name").small());
        ui.add(egui::TextEdit::singleline(&mut form.name_prefix).desired_width(150.0));
    });
    ui.horizontal(|ui| {
        ui.add_space(120.0 + 8.0);
        ui.checkbox(
            &mut form.allow_insecure,
            "allowInsecure=1 for mismatched Address/SNI/Host testing",
        )
        .on_hover_text("Use false when Address, SNI, and Host all match your own relay domain. Use true only when deliberately testing front candidates.");
    });
    form_row(
        ui,
        "Candidates",
        Some("One Address/SNI candidate per line. Host remains the deployed relay hostname."),
        |ui| {
            ui.add(
                egui::TextEdit::multiline(&mut form.candidates)
                    .font(egui::TextStyle::Monospace)
                    .desired_width(f32::INFINITY)
                    .desired_rows(6),
            );
        },
    );
    ui.horizontal_wrapped(|ui| {
        ui.add_space(120.0 + 8.0);
        if ui.button("Generate VLESS links").clicked() {
            match generate_xhttp_vless_links(form) {
                Ok(output) => {
                    form.output = output;
                    toast = Some("Generated XHTTP VLESS links.".into());
                }
                Err(e) => toast = Some(e),
            }
        }
        if ui
            .small_button("copy")
            .on_hover_text("Copy the generated links.")
            .clicked()
        {
            match generate_xhttp_vless_links(form) {
                Ok(output) => {
                    ui.ctx().copy_text(output.clone());
                    form.output = output;
                    toast = Some("Copied generated XHTTP links.".into());
                }
                Err(e) => toast = Some(e),
            }
        }
    });
    if !form.output.is_empty() {
        form_row(
            ui,
            "Output",
            Some("Paste one generated link into v2rayN/v2rayNG or another Xray-compatible client."),
            |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut form.output)
                        .font(egui::TextStyle::Monospace)
                        .desired_width(f32::INFINITY)
                        .desired_rows(5),
                );
            },
        );
    }
    ui.separator();
    help_subheading(ui, "Deploy assistant");
    form_row(ui, "TARGET_DOMAIN", Some("Backend origin for the relay (scheme + host + port). Required for manual steps and API deploy."), |ui| {
        ui.add(
            egui::TextEdit::singleline(&mut form.target_domain)
                .hint_text("https://xray.example.com:2096")
                .desired_width(f32::INFINITY),
        );
    });
    ui.horizontal(|ui| {
        ui.add_space(120.0 + 8.0);
        egui::Frame::none()
            .fill(ACCENT.linear_multiply(0.065))
            .stroke(egui::Stroke::new(1.0, ACCENT.linear_multiply(0.28)))
            .rounding(10.0)
            .inner_margin(egui::Margin::symmetric(11.0, 9.0))
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing.x = 10.0;
                    ui.label(egui::RichText::new("Deploy via").small().color(TEXT_MUTED));
                    for (tab_id, label) in [
                        ("manual", "Manual / CLI"),
                        ("vercel_api", "Vercel API"),
                        ("netlify_api", "Netlify API"),
                    ] {
                        let sel = form.deploy_tab == tab_id;
                        let mut rt = egui::RichText::new(label).size(13.0);
                        rt = if sel {
                            rt.strong().color(egui::Color32::WHITE)
                        } else {
                            rt.color(TEXT_LABEL)
                        };
                        if ui.add(egui::SelectableLabel::new(sel, rt)).clicked() {
                            form.deploy_tab = tab_id.into();
                        }
                    }
                });
            });
    });

    if form.deploy_tab == "manual" {
        help_muted(ui, "CLI or dashboard only — no token stored.");
        ui.horizontal_wrapped(|ui| {
            ui.add_space(120.0 + 8.0);
            if ui.button("Generate deploy steps").clicked() {
                match generate_xhttp_deploy_notes(form) {
                    Ok(notes) => {
                        form.deploy_notes = notes;
                        toast = Some("Generated XHTTP deployment steps.".into());
                    }
                    Err(e) => toast = Some(e),
                }
            }
            if ui.small_button("copy steps").clicked() {
                match generate_xhttp_deploy_notes(form) {
                    Ok(notes) => {
                        ui.ctx().copy_text(notes.clone());
                        form.deploy_notes = notes;
                        toast = Some("Copied deployment steps.".into());
                    }
                    Err(e) => toast = Some(e),
                }
            }
        });
        if !form.deploy_notes.is_empty() {
            form_row(
                ui,
                "Steps",
                Some("Manual checklist for tools/vercel-xhttp-relay or tools/netlify-xhttp-relay."),
                |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut form.deploy_notes)
                            .font(egui::TextStyle::Monospace)
                            .desired_width(f32::INFINITY)
                            .desired_rows(7),
                    );
                },
            );
        }
    } else {
        let plat_name = if form.deploy_tab == "vercel_api" {
            "Vercel"
        } else {
            "Netlify"
        };
        let api_hint = format!(
            "{plat_name} token is sent only to {plat_name}'s API from this process and is never saved to config.json. Keep the token short-lived, then clear it after deployment."
        );
        help_muted(ui, &api_hint);
        form_row(
            ui,
            "API token",
            Some("Paste a token with deploy scope. Never committed to disk by this app."),
            |ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut form.deploy_api_token)
                        .password(!form.show_deploy_api_token)
                        .desired_width(f32::INFINITY),
                );
            },
        );
        ui.horizontal(|ui| {
            ui.add_space(120.0 + 8.0);
            ui.checkbox(&mut form.show_deploy_api_token, "Show token");
            ui.checkbox(
                &mut form.randomize_bundle_names,
                "Randomize project internals",
            )
            .on_hover_text("Optional hygiene: randomizes generated project/route/env names where the platform allows it. It does not change relay behavior.");
        });
        ui.horizontal_wrapped(|ui| {
            ui.add_space(120.0 + 8.0);
            let can_go = !deploy_pipe.busy && deploy_pipe.rx.is_none();
            let label = if deploy_pipe.busy {
                "Deploying…"
            } else {
                "Deploy to cloud"
            };
            let base = egui::Button::new(
                egui::RichText::new(label)
                    .strong()
                    .color(egui::Color32::WHITE),
            )
            .rounding(8.0)
            .min_size(egui::vec2(172.0, 34.0));
            let btn = if deploy_pipe.busy {
                base.fill(egui::Color32::from_rgb(72, 68, 62))
                    .stroke(egui::Stroke::new(1.0, CARD_STROKE))
            } else if can_go {
                base.fill(ACCENT.linear_multiply(0.82))
                    .stroke(egui::Stroke::new(1.0, ACCENT.linear_multiply(1.05)))
            } else {
                base
            };
            if ui.add_enabled(can_go && !deploy_pipe.busy, btn).clicked() {
                if let Err(e) = generate_xhttp_deploy_notes(form).map(|_| ()) {
                    toast = Some(e);
                } else {
                    let (tx, rx) = std::sync::mpsc::channel();
                    deploy_pipe.rx = Some(rx);
                    deploy_pipe.busy = true;
                    form.deploy_log.clear();
                    let token = form.deploy_api_token.clone();
                    let target = form.target_domain.clone();
                    let randomize = form.randomize_bundle_names;
                    let which = form.deploy_tab.clone();
                    std::thread::spawn(move || {
                        let res = match which.as_str() {
                            "vercel_api" => xhttp_cloud_deploy::deploy_vercel_xhttp(
                                &token, &target, randomize, &tx,
                            ),
                            "netlify_api" => xhttp_cloud_deploy::deploy_netlify_xhttp(
                                &token, &target, randomize, &tx,
                            ),
                            _ => Err("unknown deploy tab".into()),
                        };
                        let _ = tx.send(XhttpDeployWorkerMsg::Done(res));
                    });
                    toast = Some("Cloud deploy started — watch log below.".into());
                }
            }
            if ui.small_button("clear log").clicked() {
                form.deploy_log.clear();
            }
            if !form.deploy_last_host.is_empty()
                && ui
                    .small_button("copy relay host")
                    .on_hover_text("Copy last successful deploy hostname.")
                    .clicked()
            {
                ui.ctx().copy_text(form.deploy_last_host.clone());
                toast = Some("Copied relay host.".into());
            }
            if ui
                .small_button("clear token")
                .on_hover_text(
                    "Remove the deploy API token from RAM after you are done. Does not undo or delete the remote deployment.",
                )
                .clicked()
            {
                form.deploy_api_token.clear();
                form.show_deploy_api_token = false;
                toast = Some("Deploy API token cleared from memory.".into());
            }
        });
        if !form.deploy_log.is_empty() {
            form_row(ui, "Deploy log", None, |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut form.deploy_log)
                        .font(egui::TextStyle::Monospace)
                        .desired_width(f32::INFINITY)
                        .desired_rows(6),
                );
            });
        }
        if !form.deploy_last_host.is_empty() {
            form_row(ui, "Last deploy host", Some("Copied into Relay Host on success. Use Clear token when finished pasting credentials."), |ui| {
                ui.label(egui::RichText::new(&form.deploy_last_host).monospace());
            });
        }
    }
    toast
}

pub(crate) fn poll_xhttp_cloud_deploy(
    form: &mut XhttpGeneratorForm,
    deploy_pipe: &mut XhttpDeployPipe,
    ctx: &egui::Context,
) -> Option<String> {
    let rx = deploy_pipe.rx.take()?;
    loop {
        match rx.try_recv() {
            Ok(XhttpDeployWorkerMsg::Log(line)) => {
                form.deploy_log.push_str(&line);
                form.deploy_log.push('\n');
            }
            Ok(XhttpDeployWorkerMsg::Done(res)) => {
                deploy_pipe.busy = false;
                deploy_pipe.rx = None;
                return match res {
                    Ok(host) => {
                        form.relay_host = host.clone();
                        form.deploy_last_host = host.clone();
                        Some(format!("Cloud deploy complete: {host}"))
                    }
                    Err(e) => Some(format!("Cloud deploy failed: {e}")),
                };
            }
            Err(TryRecvError::Empty) => {
                deploy_pipe.rx = Some(rx);
                ctx.request_repaint_after(Duration::from_millis(150));
                return None;
            }
            Err(TryRecvError::Disconnected) => {
                deploy_pipe.busy = false;
                deploy_pipe.rx = None;
                return None;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_form() -> XhttpGeneratorForm {
        XhttpGeneratorForm {
            uuid: "11111111-1111-4111-8111-111111111111".into(),
            relay_host: "https://relay.example.com/p4r34m".into(),
            target_domain: "https://xray.example.com:2096".into(),
            candidates: "kubernetes.io\nhttps://kubernetes.io/path\nhelm.sh, react.dev".into(),
            ..Default::default()
        }
    }

    #[test]
    fn normalize_xhttp_host_strips_scheme_path_and_space() {
        assert_eq!(
            normalize_xhttp_host(" https://relay.example.com/p4r34m "),
            "relay.example.com"
        );
        assert_eq!(normalize_xhttp_host("http://example.test"), "example.test");
        assert_eq!(normalize_xhttp_host("plain.example/a/b"), "plain.example");
    }

    #[test]
    fn normalize_xhttp_path_defaults_and_prepends_slash() {
        assert_eq!(normalize_xhttp_path(""), "/p4r34m");
        assert_eq!(normalize_xhttp_path("abc"), "/abc");
        assert_eq!(normalize_xhttp_path("/abc"), "/abc");
    }

    #[test]
    fn encode_uri_component_encodes_reserved_bytes() {
        assert_eq!(encode_uri_component("/p4 r"), "%2Fp4%20r");
        assert_eq!(encode_uri_component("abc-_.~"), "abc-_.~");
        assert_eq!(encode_uri_component("h2,http/1.1"), "h2%2Chttp%2F1.1");
    }

    #[test]
    fn generate_xhttp_links_deduplicates_candidates() {
        let links = generate_xhttp_vless_links(&valid_form()).expect("links");
        assert_eq!(links.matches("vless://").count(), 3);
        assert!(links.contains("@kubernetes.io:443?"));
        assert!(links.contains("@helm.sh:443?"));
        assert!(links.contains("@react.dev:443?"));
        assert!(links.contains("host=relay.example.com"));
        assert!(links.contains("path=%2Fp4r34m"));
    }

    #[test]
    fn generate_xhttp_deploy_notes_requires_full_target_url() {
        let mut form = valid_form();
        form.target_domain = "xray.example.com:2096".into();
        let err = generate_xhttp_deploy_notes(&form).expect_err("scheme required");
        assert!(err.contains("http:// or https://"));
    }

    #[test]
    fn xhttp_platform_defaults_select_candidate_sets() {
        let (host, path, prefix, candidates) = xhttp_platform_defaults("vercel");
        assert_eq!(host, "your-project.vercel.app");
        assert_eq!(path, "/yourpath");
        assert_eq!(prefix, "vercel-xhttp");
        assert!(candidates.contains(&"nextjs.org"));
    }
}
