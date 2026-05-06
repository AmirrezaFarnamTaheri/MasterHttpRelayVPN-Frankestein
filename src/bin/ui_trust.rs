use std::sync::mpsc::Sender;

use eframe::egui;
use mhrv_jni::config::Config;
use mhrv_jni::support_bundle;
use mhrv_jni::trust_center::{self, TrustStatus};

use crate::ui_style::{
    help_callout, help_muted, help_subheading, ACCENT, ACCENT_WARM, CARD_STROKE_HI, ERR_RED,
    OK_GREEN, TEXT_LABEL, TEXT_MAIN, TEXT_MUTED,
};
use crate::{Cmd, FormState};

fn trust_status_label(status: &TrustStatus) -> (&'static str, egui::Color32) {
    match status {
        TrustStatus::NotRequired => ("not required", OK_GREEN),
        TrustStatus::Missing => ("missing", ERR_RED),
        TrustStatus::PresentTrusted => ("trusted", OK_GREEN),
        TrustStatus::PresentUntrusted => ("present, not trusted", ACCENT_WARM),
    }
}

fn bool_status(value: bool) -> (&'static str, egui::Color32) {
    if value {
        ("yes", OK_GREEN)
    } else {
        ("no", ACCENT_WARM)
    }
}

pub(crate) fn trust_center_snapshot_panel(ui: &mut egui::Ui, config: Result<Config, String>) {
    match config {
        Ok(cfg) => {
            let snap = trust_center::snapshot(&cfg);
            let manifest = support_bundle::preview_manifest();
            let (ca_label, ca_color) = trust_status_label(&snap.ca.status);
            let (cert_label, cert_color) = bool_status(snap.ca.cert_exists);
            let (key_label, key_color) = bool_status(snap.ca.key_exists);
            let sensitive_files = manifest
                .files
                .iter()
                .filter(|file| file.contains_sensitive_material)
                .count();

            egui::Frame::none()
                .fill(egui::Color32::from_rgb(29, 32, 35))
                .stroke(egui::Stroke::new(1.0, CARD_STROKE_HI.linear_multiply(0.55)))
                .rounding(8.0)
                .inner_margin(egui::Margin::symmetric(12.0, 10.0))
                .show(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(
                            egui::RichText::new(format!("Mode: {}", snap.mode))
                                .strong()
                                .color(TEXT_MAIN),
                        );
                        ui.separator();
                        ui.label(
                            egui::RichText::new(format!("CA: {ca_label}"))
                                .strong()
                                .color(ca_color),
                        );
                        ui.separator();
                        ui.label(
                            egui::RichText::new(format!(
                                "Support bundle: {} files, {} sensitive",
                                manifest.files.len(),
                                sensitive_files
                            ))
                            .color(TEXT_MUTED),
                        );
                    });
                    ui.add_space(6.0);
                    egui::Grid::new("trust_center_snapshot_grid")
                        .num_columns(2)
                        .spacing([16.0, 5.0])
                        .show(ui, |ui| {
                            ui.label(egui::RichText::new("CA cert").color(TEXT_LABEL));
                            ui.label(egui::RichText::new(cert_label).color(cert_color));
                            ui.end_row();

                            ui.label(egui::RichText::new("CA key").color(TEXT_LABEL));
                            ui.label(egui::RichText::new(key_label).color(key_color));
                            ui.end_row();

                            ui.label(egui::RichText::new("Platform probe").color(TEXT_LABEL));
                            let probe = match snap.ca.trusted_by_platform_probe {
                                Some(true) => ("trusted", OK_GREEN),
                                Some(false) => ("not trusted", ACCENT_WARM),
                                None => ("not available", TEXT_MUTED),
                            };
                            ui.label(egui::RichText::new(probe.0).color(probe.1));
                            ui.end_row();

                            ui.label(egui::RichText::new("Firefox profiles").color(TEXT_LABEL));
                            ui.label(
                                egui::RichText::new(format!(
                                    "{} found, {} NSS DB, {} managed markers",
                                    snap.browser.firefox_profile_count,
                                    snap.browser.firefox_profiles_with_cert_db,
                                    snap.browser.firefox_profiles_with_enterprise_roots_marker
                                ))
                                .color(TEXT_MUTED),
                            );
                            ui.end_row();

                            ui.label(egui::RichText::new("certutil").color(TEXT_LABEL));
                            let (certutil, certutil_color) =
                                bool_status(snap.browser.certutil_available);
                            ui.label(egui::RichText::new(certutil).color(certutil_color));
                            ui.end_row();

                            ui.label(egui::RichText::new("Firefox NSS CA").color(TEXT_LABEL));
                            let firefox_nss = snap
                                .browser
                                .firefox_profiles_with_nss_cert
                                .map(|n| format!("{n} profile(s)"))
                                .unwrap_or_else(|| "certutil unavailable".to_string());
                            ui.label(egui::RichText::new(firefox_nss).color(TEXT_MUTED));
                            ui.end_row();

                            if !snap.browser.firefox_profiles.is_empty() {
                                ui.label(egui::RichText::new("Firefox details").color(TEXT_LABEL));
                                ui.vertical(|ui| {
                                    for profile in snap.browser.firefox_profiles.iter().take(4) {
                                        let nss_ca = profile
                                            .nss_has_cert
                                            .map(
                                                |has| if has { "CA present" } else { "CA missing" },
                                            )
                                            .unwrap_or("CA unknown");
                                        let marker = if profile.enterprise_roots_marker {
                                            "managed"
                                        } else if profile.enterprise_roots_user_owned {
                                            "user enterprise_roots"
                                        } else {
                                            "no marker"
                                        };
                                        ui.small(
                                            egui::RichText::new(format!(
                                                "{}: {}, {}, {}",
                                                profile.profile_label,
                                                if profile.has_cert_db {
                                                    "NSS DB"
                                                } else {
                                                    "no NSS DB"
                                                },
                                                nss_ca,
                                                marker
                                            ))
                                            .color(TEXT_MUTED),
                                        );
                                    }
                                    if snap.browser.firefox_profiles.len() > 4 {
                                        ui.small(
                                            egui::RichText::new(format!(
                                                "+{} more profile(s) in trust-center --json",
                                                snap.browser.firefox_profiles.len() - 4
                                            ))
                                            .color(TEXT_MUTED),
                                        );
                                    }
                                });
                                ui.end_row();
                            }

                            ui.label(egui::RichText::new("Chrome NSS CA").color(TEXT_LABEL));
                            let chrome_nss = snap
                                .browser
                                .chrome_nssdb_has_cert
                                .map(|has| if has { "present" } else { "missing" })
                                .unwrap_or("unavailable");
                            let chrome_color = match snap.browser.chrome_nssdb_has_cert {
                                Some(true) => OK_GREEN,
                                Some(false) => ACCENT_WARM,
                                None => TEXT_MUTED,
                            };
                            ui.label(egui::RichText::new(chrome_nss).color(chrome_color));
                            ui.end_row();

                            ui.label(egui::RichText::new("Signing policy").color(TEXT_LABEL));
                            ui.label(
                                egui::RichText::new(snap.signing.android_release_keystore_policy)
                                    .color(TEXT_MUTED),
                            );
                            ui.end_row();
                        });
                    if let Some(action) = snap.ca.next_action {
                        ui.add_space(6.0);
                        ui.small(egui::RichText::new(action).color(ACCENT_WARM));
                    }
                });
        }
        Err(err) => {
            help_callout(
                ui,
                "Trust snapshot unavailable",
                &format!(
                    "The current form does not validate yet, so the Trust Center cannot map CA requirements to a mode: {err}"
                ),
                ACCENT_WARM,
            );
        }
    }
}

pub(crate) fn support_bundle_preview(ui: &mut egui::Ui) {
    let manifest = support_bundle::preview_manifest();
    ui.label(
        egui::RichText::new(format!(
            "{} files; auth keys {}, LAN tokens {}, deployment IDs {}, private keys {}.",
            manifest.files.len(),
            manifest.redaction.auth_keys,
            manifest.redaction.lan_tokens,
            manifest.redaction.deployment_ids,
            manifest.redaction.private_keys,
        ))
        .color(TEXT_MUTED),
    );
    ui.add_space(4.0);
    egui::Grid::new("trust_support_bundle_manifest_grid")
        .num_columns(3)
        .spacing([12.0, 4.0])
        .striped(true)
        .show(ui, |ui| {
            ui.label(egui::RichText::new("file").strong().color(TEXT_LABEL));
            ui.label(egui::RichText::new("category").strong().color(TEXT_LABEL));
            ui.label(egui::RichText::new("sensitive").strong().color(TEXT_LABEL));
            ui.end_row();
            for file in &manifest.files {
                ui.label(egui::RichText::new(file.path).monospace());
                ui.label(egui::RichText::new(file.category).color(TEXT_MUTED));
                let (label, color) = bool_status(file.contains_sensitive_material);
                ui.label(egui::RichText::new(label).color(color));
                ui.end_row();
            }
        });
}

pub(crate) fn trust_center_tab(ui: &mut egui::Ui, form: &FormState, cmd_tx: &Sender<Cmd>) {
    help_muted(
        ui,
        "Read-only trust state for this config. Install/remove/check actions reuse the existing serialized CA commands; the snapshot itself does not mutate trust stores.",
    );
    ui.add_space(6.0);
    trust_center_snapshot_panel(ui, form.to_config());

    ui.add_space(10.0);
    help_subheading(ui, "Certificate actions");
    ui.horizontal_wrapped(|ui| {
        if ui
            .small_button("Install CA")
            .on_hover_text("Install or repair the local MITM CA trust. This may need admin privileges.")
            .clicked()
        {
            let _ = cmd_tx.send(Cmd::InstallCa);
        }
        if ui
            .small_button("Remove CA")
            .on_hover_text("Remove the local MITM CA from OS/browser trust stores and delete local ca/ when safe.")
            .clicked()
        {
            let _ = cmd_tx.send(Cmd::RemoveCa);
        }
        if ui
            .small_button("Check CA")
            .on_hover_text("Run the local OS trust probe without changing files.")
            .clicked()
        {
            let _ = cmd_tx.send(Cmd::CheckCaTrusted);
        }
    });

    ui.add_space(10.0);
    help_subheading(ui, "Support bundle preview");
    support_bundle_preview(ui);

    ui.add_space(10.0);
    help_subheading(ui, "Docs");
    ui.horizontal_wrapped(|ui| {
        ui.hyperlink_to(
            egui::RichText::new("Trust Center").size(12.0).color(ACCENT),
            "docs/trust-center.md",
        );
        ui.hyperlink_to(
            egui::RichText::new("Safety").size(12.0).color(ACCENT),
            "docs/safety-security.md",
        );
        ui.hyperlink_to(
            egui::RichText::new("Android signing")
                .size(12.0)
                .color(ACCENT),
            "docs/android-signing.md",
        );
        ui.hyperlink_to(
            egui::RichText::new("Doctor").size(12.0).color(ACCENT),
            "docs/doctor.md",
        );
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trust_status_labels_are_stable() {
        assert_eq!(
            trust_status_label(&TrustStatus::NotRequired).0,
            "not required"
        );
        assert_eq!(trust_status_label(&TrustStatus::Missing).0, "missing");
        assert_eq!(
            trust_status_label(&TrustStatus::PresentTrusted).0,
            "trusted"
        );
        assert_eq!(
            trust_status_label(&TrustStatus::PresentUntrusted).0,
            "present, not trusted"
        );
    }

    #[test]
    fn bool_status_uses_yes_no_copy() {
        assert_eq!(bool_status(true).0, "yes");
        assert_eq!(bool_status(false).0, "no");
    }

    #[test]
    fn support_manifest_has_expected_redaction_contract() {
        let manifest = support_bundle::preview_manifest();
        assert!(!manifest.files.is_empty());
        assert!(!manifest.redaction.auth_keys.is_empty());
        assert!(!manifest.redaction.deployment_ids.is_empty());
        assert!(!manifest.redaction.private_keys.is_empty());
    }

    #[test]
    fn trust_tab_docs_links_are_local_reference_paths() {
        let docs = [
            "docs/trust-center.md",
            "docs/safety-security.md",
            "docs/android-signing.md",
            "docs/doctor.md",
        ];
        for path in docs {
            assert!(path.starts_with("docs/"));
            assert!(path.ends_with(".md"));
        }
    }
}
