//! Placeholder tab rendering for PULSE.

use eframe::egui;

use crate::theme::{card_frame, colors};

/// Renders a placeholder card for unimplemented tabs.
pub fn render_placeholder_tab(ui: &mut egui::Ui, tab_label: &str) {
    card_frame().show(ui, |ui| {
        ui.label(
            egui::RichText::new(format!("{} — Coming in v0.2.0", tab_label))
                .color(colors::TEXT_SECONDARY)
                .size(14.0),
        );
    });
}
