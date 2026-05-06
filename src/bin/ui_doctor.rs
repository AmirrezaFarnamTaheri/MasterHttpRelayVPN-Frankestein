use std::time::Instant;

use eframe::egui;
use mhrv_jni::doctor::{DoctorItem, DoctorLevel, DoctorReport};

use crate::ui_format::fmt_duration;
use crate::ui_style::{section, ACCENT_WARM, ERR_RED, OK_GREEN, TEXT_LABEL, TEXT_MAIN, TEXT_MUTED};

pub(crate) fn doctor_level_label(level: &DoctorLevel) -> &'static str {
    match level {
        DoctorLevel::Ok => "OK",
        DoctorLevel::Warn => "WARN",
        DoctorLevel::Fail => "FAIL",
    }
}

fn doctor_level_color(level: &DoctorLevel) -> egui::Color32 {
    match level {
        DoctorLevel::Ok => OK_GREEN,
        DoctorLevel::Warn => ACCENT_WARM,
        DoctorLevel::Fail => ERR_RED,
    }
}

fn doctor_counts(report: &DoctorReport) -> (usize, usize, usize) {
    report
        .items
        .iter()
        .fold((0, 0, 0), |(ok, warn, fail), item| match item.level {
            DoctorLevel::Ok => (ok + 1, warn, fail),
            DoctorLevel::Warn => (ok, warn + 1, fail),
            DoctorLevel::Fail => (ok, warn, fail + 1),
        })
}

pub(crate) fn render_doctor_summary_card(
    ui: &mut egui::Ui,
    report: Option<&DoctorReport>,
    at: Option<Instant>,
) {
    section(ui, "Doctor summary", |ui| {
        let Some(report) = report else {
            ui.label(
                egui::RichText::new("Run Doctor to populate structured diagnostics.")
                    .color(TEXT_MUTED)
                    .italics(),
            );
            return;
        };

        let (ok, warn, fail) = doctor_counts(report);
        let status = if report.ok() { "OK" } else { "Needs attention" };
        let status_color = if report.ok() { OK_GREEN } else { ERR_RED };
        ui.horizontal_wrapped(|ui| {
            ui.label(
                egui::RichText::new(status)
                    .strong()
                    .color(status_color)
                    .size(15.0),
            );
            ui.label(
                egui::RichText::new(format!("{} ok · {} warn · {} fail", ok, warn, fail))
                    .monospace()
                    .color(TEXT_LABEL),
            );
            if let Some(at) = at {
                ui.label(
                    egui::RichText::new(format!("updated {} ago", fmt_duration(at.elapsed())))
                        .color(TEXT_MUTED),
                );
            }
        });

        ui.add_space(7.0);
        let mut shown = 0usize;
        let non_ok: Vec<&DoctorItem> = report
            .items
            .iter()
            .filter(|item| !matches!(item.level, DoctorLevel::Ok))
            .collect();
        let items: Vec<&DoctorItem> = if non_ok.is_empty() {
            report.items.iter().take(5).collect()
        } else {
            non_ok.into_iter().take(5).collect()
        };

        for item in items {
            shown += 1;
            ui.horizontal_wrapped(|ui| {
                ui.label(
                    egui::RichText::new(doctor_level_label(&item.level))
                        .monospace()
                        .strong()
                        .color(doctor_level_color(&item.level)),
                );
                ui.label(
                    egui::RichText::new(format!("{} · {}", item.id, item.title))
                        .strong()
                        .color(TEXT_MAIN),
                );
            });
            if !item.detail.trim().is_empty() {
                ui.small(egui::RichText::new(item.detail.trim()).color(TEXT_MUTED));
            }
            if let Some(fix) = &item.fix {
                ui.small(egui::RichText::new(format!("Fix: {}", fix)).color(ACCENT_WARM));
            }
            ui.add_space(5.0);
        }

        if report.items.len() > shown {
            ui.small(
                egui::RichText::new(format!(
                    "{} more items are still available in the Doctor log.",
                    report.items.len() - shown
                ))
                .color(TEXT_MUTED),
            );
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(level: DoctorLevel) -> DoctorItem {
        DoctorItem {
            id: "test.item",
            level,
            title: "Title".into(),
            detail: "Detail".into(),
            fix: None,
        }
    }

    #[test]
    fn doctor_level_labels_are_stable() {
        assert_eq!(doctor_level_label(&DoctorLevel::Ok), "OK");
        assert_eq!(doctor_level_label(&DoctorLevel::Warn), "WARN");
        assert_eq!(doctor_level_label(&DoctorLevel::Fail), "FAIL");
    }

    #[test]
    fn doctor_counts_groups_levels() {
        let report = DoctorReport {
            items: vec![
                item(DoctorLevel::Ok),
                item(DoctorLevel::Warn),
                item(DoctorLevel::Warn),
                item(DoctorLevel::Fail),
            ],
        };
        assert_eq!(doctor_counts(&report), (1, 2, 1));
    }

    #[test]
    fn doctor_level_colors_match_status_tokens() {
        assert_eq!(doctor_level_color(&DoctorLevel::Ok), OK_GREEN);
        assert_eq!(doctor_level_color(&DoctorLevel::Warn), ACCENT_WARM);
        assert_eq!(doctor_level_color(&DoctorLevel::Fail), ERR_RED);
    }
}
