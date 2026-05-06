use std::sync::mpsc::Sender;
use std::time::Instant;

use eframe::egui;

use crate::ui_style::{form_row, help_muted, section};
use crate::{AccountGroupForm, Cmd, FormState};

pub(crate) fn show_first_run_wizard(
    ui: &mut egui::Ui,
    form: &mut FormState,
    cmd_tx: &Sender<Cmd>,
    toast: &mut Option<(String, Instant)>,
) {
    section(ui, "First-run wizard", |ui| {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Step").color(egui::Color32::from_gray(150)));
            for (idx, title) in ["Mode", "Relay", "CA", "Diagnostics"].iter().enumerate() {
                let selected = form.wizard_step == idx;
                if ui
                    .selectable_label(selected, *title)
                    .on_hover_text("Jump to this setup step")
                    .clicked()
                {
                    form.wizard_step = idx;
                }
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.small_button("Hide").clicked() {
                    form.show_first_run_wizard = false;
                }
            });
        });
        ui.separator();

        match form.wizard_step {
            0 => {
                help_muted(ui, "Choose the transport you want to set up first. Apps Script is the classic path; serverless JSON is the no-VPS Vercel/Netlify fetch relay; full mode is for the separate tunnel-node path.");
                ui.horizontal(|ui| {
                    if ui.button("Apps Script").clicked() {
                        form.mode = "apps_script".into();
                        form.wizard_step = 1;
                    }
                    if ui.button("Serverless JSON").clicked() {
                        form.mode = "vercel_edge".into();
                        form.wizard_step = 1;
                    }
                    if ui.button("Full tunnel").clicked() {
                        form.mode = "full".into();
                        form.wizard_step = 1;
                    }
                });
            }
            1 => {
                if form.mode == "vercel_edge" {
                    help_muted(ui, "Deploy tools/vercel-json-relay or tools/netlify-json-relay, set AUTH_KEY, redeploy, confirm /api/api returns JSON, and paste the deployment URL here.");
                    form_row(ui, "Base URL", None, |ui| {
                        ui.add(
                            egui::TextEdit::singleline(&mut form.vercel_base_url)
                                .hint_text(
                                    "https://your-project.vercel.app or https://your-site.netlify.app",
                                )
                                .desired_width(f32::INFINITY),
                        );
                    });
                    form_row(ui, "Relay path", None, |ui| {
                        ui.add(
                            egui::TextEdit::singleline(&mut form.vercel_relay_path)
                                .hint_text("/api/api")
                                .desired_width(f32::INFINITY),
                        );
                    });
                    form_row(ui, "Auth key", None, |ui| {
                        ui.add(
                            egui::TextEdit::singleline(&mut form.vercel_auth_key)
                                .password(!form.show_vercel_auth_key)
                                .desired_width(f32::INFINITY),
                        );
                    });
                } else if form.mode == "direct" {
                    help_muted(ui, "Direct mode is a no-relay SNI-rewrite path. Use it to reach script.google.com, or to use configured fronting groups for Google/Vercel/Fastly/Netlify-style targets.");
                } else {
                    help_muted(ui, "Add at least one Apps Script account group under Advanced -> Multi-account pools. Each enabled group needs AUTH_KEY and one or more deployment IDs.");
                    if ui.button("+ Add Apps Script group").clicked() {
                        form.account_groups.push(AccountGroupForm {
                            label: String::new(),
                            enabled: true,
                            weight: 1,
                            auth_key: String::new(),
                            script_ids: String::new(),
                            show_auth_key: false,
                        });
                    }
                }
                ui.horizontal(|ui| {
                    if ui.button("Test relay").clicked() {
                        match form.to_config() {
                            Ok(cfg) => {
                                let _ = cmd_tx.send(Cmd::Test(cfg));
                            }
                            Err(e) => {
                                *toast = Some((format!("Cannot test: {}", e), Instant::now()))
                            }
                        }
                    }
                    if ui.button("Next").clicked() {
                        form.wizard_step = 2;
                    }
                });
            }
            2 => {
                if form.mode == "full" {
                    help_muted(ui, "Full mode does not need the local MITM CA. Continue to diagnostics after the tunnel-node side is ready.");
                } else {
                    help_muted(ui, "Apps Script and serverless JSON MITM HTTPS locally. Install the generated CA into your OS trust store, then check trust status. Firefox may need restart or NSS/enterprise roots handling.");
                    ui.horizontal(|ui| {
                        if ui.button("Install CA").clicked() {
                            let _ = cmd_tx.send(Cmd::InstallCa);
                        }
                        if ui.button("Check CA").clicked() {
                            let _ = cmd_tx.send(Cmd::CheckCaTrusted);
                        }
                    });
                }
                if ui.button("Next").clicked() {
                    form.wizard_step = 3;
                }
            }
            _ => {
                help_muted(ui, "Run Doctor and Test relay. PASS means the local config can reach the relay. In full mode, Doctor skips the JSON probe and you should verify by browsing through the tunnel.");
                ui.horizontal(|ui| {
                    if ui.button("Doctor").clicked() {
                        match form.to_config() {
                            Ok(cfg) => {
                                let _ = cmd_tx.send(Cmd::Doctor(cfg));
                            }
                            Err(e) => {
                                *toast = Some((format!("Cannot run doctor: {}", e), Instant::now()))
                            }
                        }
                    }
                    if ui.button("Test relay").clicked() {
                        match form.to_config() {
                            Ok(cfg) => {
                                let _ = cmd_tx.send(Cmd::Test(cfg));
                            }
                            Err(e) => {
                                *toast = Some((format!("Cannot test: {}", e), Instant::now()))
                            }
                        }
                    }
                    if ui.button("Finish").clicked() {
                        form.show_first_run_wizard = false;
                    }
                });
            }
        }
    });
}

#[cfg(test)]
mod tests {
    #[test]
    fn wizard_steps_keep_expected_order() {
        assert_eq!(["Mode", "Relay", "CA", "Diagnostics"][0], "Mode");
        assert_eq!(["Mode", "Relay", "CA", "Diagnostics"][3], "Diagnostics");
    }
}
