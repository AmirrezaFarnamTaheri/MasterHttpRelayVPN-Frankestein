use eframe::egui;

/// Saturated blue used for primary actions, links, and focus rings.
pub(crate) const ACCENT: egui::Color32 = egui::Color32::from_rgb(102, 178, 255);
pub(crate) const ACCENT_WARM: egui::Color32 = egui::Color32::from_rgb(235, 182, 108);
pub(crate) const ACCENT_MINT: egui::Color32 = egui::Color32::from_rgb(94, 206, 164);
pub(crate) const OK_GREEN: egui::Color32 = egui::Color32::from_rgb(76, 196, 118);
pub(crate) const ERR_RED: egui::Color32 = egui::Color32::from_rgb(242, 122, 122);
pub(crate) const TEXT_MAIN: egui::Color32 = egui::Color32::from_rgb(237, 235, 230);
/// Form labels are slightly brighter than body copy for scanability.
pub(crate) const TEXT_LABEL: egui::Color32 = egui::Color32::from_rgb(222, 219, 212);
pub(crate) const TEXT_MUTED: egui::Color32 = egui::Color32::from_rgb(172, 168, 160);
pub(crate) const CARD_FILL: egui::Color32 = egui::Color32::from_rgb(34, 33, 31);
pub(crate) const CARD_STROKE: egui::Color32 = egui::Color32::from_rgb(58, 56, 52);
/// Subtle highlight mixed into section outlines.
pub(crate) const CARD_STROKE_HI: egui::Color32 = egui::Color32::from_rgb(72, 82, 96);
pub(crate) const PANEL_FILL: egui::Color32 = egui::Color32::from_rgb(22, 21, 20);
pub(crate) const SURFACE_SHADOW: egui::Shadow = egui::Shadow {
    offset: egui::vec2(0.0, 6.0),
    blur: 22.0,
    spread: 0.0,
    color: egui::Color32::from_black_alpha(72),
};
pub(crate) const HEADER_SHADOW: egui::Shadow = egui::Shadow {
    offset: egui::vec2(0.0, 8.0),
    blur: 28.0,
    spread: 0.0,
    color: egui::Color32::from_black_alpha(90),
};
const FORM_LABEL_WIDTH: f32 = 150.0;
const FORM_GAP: f32 = 12.0;

pub(crate) fn apply_ui_theme(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = egui::Color32::from_rgb(17, 16, 15);
    visuals.window_fill = PANEL_FILL;
    visuals.window_rounding = egui::Rounding::same(11.0);
    visuals.window_shadow = egui::Shadow {
        offset: egui::vec2(0.0, 14.0),
        blur: 36.0,
        spread: 0.0,
        color: egui::Color32::from_black_alpha(110),
    };
    visuals.window_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(48, 54, 62));
    visuals.popup_shadow = egui::Shadow {
        offset: egui::vec2(0.0, 8.0),
        blur: 18.0,
        spread: 0.0,
        color: egui::Color32::from_black_alpha(100),
    };
    visuals.extreme_bg_color = egui::Color32::from_rgb(14, 13, 12);
    visuals.faint_bg_color = egui::Color32::from_rgb(38, 36, 33);
    visuals.code_bg_color = egui::Color32::from_rgb(22, 21, 20);
    visuals.hyperlink_color = ACCENT;
    visuals.selection.bg_fill = ACCENT.linear_multiply(0.38);
    visuals.selection.stroke = egui::Stroke::new(1.0, ACCENT.linear_multiply(0.85));

    let wr = egui::Rounding::same(8.0);
    visuals.widgets.noninteractive.bg_fill = CARD_FILL;
    visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, CARD_STROKE);
    visuals.widgets.noninteractive.rounding = wr;
    visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(44, 41, 38);
    visuals.widgets.inactive.weak_bg_fill = egui::Color32::from_rgb(50, 46, 42);
    visuals.widgets.inactive.rounding = wr;
    visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(54, 50, 46);
    visuals.widgets.hovered.weak_bg_fill = egui::Color32::from_rgb(58, 54, 49);
    visuals.widgets.hovered.rounding = wr;
    visuals.widgets.active.bg_fill = ACCENT.linear_multiply(0.58);
    visuals.widgets.active.rounding = wr;
    visuals.widgets.open.bg_fill = egui::Color32::from_rgb(48, 44, 40);
    visuals.widgets.open.rounding = wr;

    visuals.collapsing_header_frame = true;
    visuals.button_frame = true;
    visuals.indent_has_left_vline = true;
    ctx.set_visuals(visuals);

    ctx.style_mut(|s| {
        s.text_styles
            .insert(egui::TextStyle::Heading, egui::FontId::proportional(23.0));
        s.text_styles
            .insert(egui::TextStyle::Body, egui::FontId::proportional(14.9));
        s.text_styles
            .insert(egui::TextStyle::Button, egui::FontId::proportional(14.1));
        s.text_styles
            .insert(egui::TextStyle::Small, egui::FontId::proportional(12.9));
        s.text_styles
            .insert(egui::TextStyle::Monospace, egui::FontId::monospace(13.0));
        s.spacing.item_spacing = egui::vec2(10.0, 9.0);
        s.spacing.button_padding = egui::vec2(15.0, 8.0);
        s.spacing.interact_size = egui::vec2(40.0, 34.0);
        s.spacing.combo_width = 268.0;
        s.spacing.text_edit_width = 348.0;
        s.spacing.tooltip_width = 440.0;
        s.spacing.window_margin = egui::Margin::same(10.0);
    });
}

/// Section title with a thin accent rail for readability and visual rhythm.
pub(crate) fn section_title_bar(ui: &mut egui::Ui, title: &str) {
    ui.add_space(14.0);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 11.0;
        let (rect, _) = ui.allocate_exact_size(egui::vec2(5.0, 20.0), egui::Sense::hover());
        let shine = ACCENT.linear_multiply(1.08).gamma_multiply(1.05);
        ui.painter()
            .rect_filled(rect, egui::Rounding::same(3.0), shine);
        ui.label(
            egui::RichText::new(title)
                .size(15.8)
                .color(TEXT_MAIN)
                .strong(),
        );
    });
    ui.add_space(9.0);
}

/// Draw a rounded section frame grouping related controls.
pub(crate) fn section(ui: &mut egui::Ui, title: &str, body: impl FnOnce(&mut egui::Ui)) {
    section_title_bar(ui, title);
    let frame = egui::Frame::none()
        .fill(CARD_FILL)
        .stroke(egui::Stroke::new(
            1.0,
            egui::Color32::from_rgb(
                (CARD_STROKE.r() + CARD_STROKE_HI.r()) / 2,
                (CARD_STROKE.g() + CARD_STROKE_HI.g()) / 2,
                (CARD_STROKE.b() + CARD_STROKE_HI.b()) / 2,
            ),
        ))
        .rounding(11.0)
        .shadow(SURFACE_SHADOW)
        .inner_margin(egui::Margin::symmetric(21.0, 18.0));
    frame.show(ui, body);
}

pub(crate) fn help_subheading(ui: &mut egui::Ui, text: &str) {
    ui.add_space(6.0);
    ui.label(
        egui::RichText::new(text)
            .strong()
            .color(ACCENT.linear_multiply(1.02))
            .size(13.7),
    );
    ui.add_space(5.0);
}

pub(crate) fn help_muted(ui: &mut egui::Ui, text: &str) {
    ui.label(
        egui::RichText::new(text)
            .size(13.0)
            .line_height(Some(18.0))
            .color(TEXT_MUTED),
    );
}

pub(crate) fn help_callout(ui: &mut egui::Ui, title: &str, body: &str, color: egui::Color32) {
    egui::Frame::none()
        .fill(color.linear_multiply(0.14))
        .stroke(egui::Stroke::new(1.0, color.linear_multiply(0.5)))
        .rounding(10.0)
        .shadow(egui::Shadow {
            offset: egui::vec2(0.0, 3.0),
            blur: 12.0,
            spread: 0.0,
            color: egui::Color32::from_black_alpha(48),
        })
        .inner_margin(egui::Margin::symmetric(14.0, 11.0))
        .show(ui, |ui| {
            ui.label(egui::RichText::new(title).strong().color(color).size(13.2));
            ui.add_space(4.0);
            help_muted(ui, body);
        });
}

pub(crate) fn mode_goal_card(ui: &mut egui::Ui, title: &str, body: &str, color: egui::Color32) {
    let width = ((ui.available_width() - 14.0) / 2.0).max(240.0);
    egui::Frame::none()
        .fill(egui::Color32::from_rgb(40, 37, 34))
        .stroke(egui::Stroke::new(1.0, color.linear_multiply(0.48)))
        .rounding(10.0)
        .shadow(egui::Shadow {
            offset: egui::vec2(0.0, 2.0),
            blur: 10.0,
            spread: 0.0,
            color: egui::Color32::from_black_alpha(40),
        })
        .inner_margin(egui::Margin::symmetric(14.0, 12.0))
        .show(ui, |ui| {
            ui.set_min_width(width);
            ui.label(egui::RichText::new(title).strong().color(color).size(13.6));
            ui.add_space(4.0);
            help_muted(ui, body);
        });
}

/// A primary accent-filled button for headline row actions.
pub(crate) fn primary_button(text: &str) -> egui::Button<'_> {
    egui::Button::new(
        egui::RichText::new(text)
            .color(egui::Color32::WHITE)
            .strong(),
    )
    .fill(ACCENT.linear_multiply(0.95))
    .stroke(egui::Stroke::new(1.0, ACCENT.linear_multiply(1.15)))
    .min_size(egui::vec2(130.0, 34.0))
    .rounding(8.0)
}

/// A compact form row: label on the left, widget on the right.
pub(crate) fn form_row(
    ui: &mut egui::Ui,
    label: &str,
    hover: Option<&str>,
    widget: impl FnOnce(&mut egui::Ui),
) {
    ui.horizontal(|ui| {
        let resp = ui.add_sized(
            [FORM_LABEL_WIDTH, 24.0],
            egui::Label::new(egui::RichText::new(label).color(TEXT_LABEL).strong()),
        );
        if let Some(h) = hover {
            resp.on_hover_text(h);
        }
        ui.add_space(FORM_GAP);
        widget(ui);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn style_tokens_keep_expected_palette() {
        assert_eq!(ACCENT, egui::Color32::from_rgb(102, 178, 255));
        assert_eq!(ACCENT_WARM, egui::Color32::from_rgb(235, 182, 108));
        assert_eq!(OK_GREEN, egui::Color32::from_rgb(76, 196, 118));
        assert_eq!(ERR_RED, egui::Color32::from_rgb(242, 122, 122));
        assert_eq!(TEXT_LABEL, egui::Color32::from_rgb(222, 219, 212));
        assert_eq!(TEXT_MUTED, egui::Color32::from_rgb(172, 168, 160));
    }

    #[test]
    fn form_row_metrics_stay_stable() {
        assert_eq!(FORM_LABEL_WIDTH, 150.0);
        assert_eq!(FORM_GAP, 12.0);
    }

    #[test]
    fn shadows_keep_operational_depth() {
        assert_eq!(SURFACE_SHADOW.blur, 22.0);
        assert_eq!(HEADER_SHADOW.blur, 28.0);
        assert_eq!(PANEL_FILL, egui::Color32::from_rgb(22, 21, 20));
    }
}
