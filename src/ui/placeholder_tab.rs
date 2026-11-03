//! Placeholder tab rendering for PULSE.

use eframe::egui;

use crate::theme::{card_frame, colors};

/// Renders a placeholder card for unimplemented tabs.
/// Takes `target_version` explicitly (e.g., "v0.2.0") to avoid locale-dependent matching.
pub fn render_placeholder_tab(ui: &mut egui::Ui, tab_label: &str, target_version: &'static str) {
    card_frame().show(ui, |ui| {
        ui.label(
            egui::RichText::new(format!("{} — Planned for {}", tab_label, target_version))
                .color(colors::TEXT_SECONDARY)
                .size(14.0),
        );
    });
}
