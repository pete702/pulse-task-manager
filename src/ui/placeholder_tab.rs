//! Placeholder tab rendering for PULSE.

use eframe::egui;

use crate::theme::{card_frame, colors};

/// Renders a placeholder card for unimplemented tabs.
pub fn render_placeholder_tab(ui: &mut egui::Ui, tab_label: &str) {
    let version = match tab_label {
        "▣ Processor (CPU)" => "v0.2.0",
        "◧ Storage (Disks)" => "v0.3.0",
        "⇅ Network & Ping" => "v0.4.0",
        "⚙ System & GPU" => "v0.5.0",
        _ => "v0.2.0",
    };
    card_frame().show(ui, |ui| {
        ui.label(
            egui::RichText::new(format!("{} — Planned for {}", tab_label, version))
                .color(colors::TEXT_SECONDARY)
                .size(14.0),
        );
    });
}
