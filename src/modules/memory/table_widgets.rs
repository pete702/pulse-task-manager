//! Shared widgets for the virtualized memory process table.

use eframe::egui;

use crate::modules::memory::formatters::format_bytes;
use crate::modules::memory::process_model::MemorySpikeEvent;
use crate::theme::colors;

use super::table_state::MemoryTableState;

/// Renders the amber/red "black box" high RAM pressure banner.
pub(super) fn render_spike_banner(ui: &mut egui::Ui, spike: &MemorySpikeEvent) {
    egui::Frame::none()
        .fill(colors::ACCENT_RED.gamma_multiply(0.12))
        .stroke(egui::Stroke::new(1.0_f32, colors::ACCENT_RED))
        .rounding(egui::Rounding::same(colors::RADIUS))
        .inner_margin(egui::Margin::same(9.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("⚠️ HIGH RAM PRESSURE")
                        .monospace()
                        .size(12.0)
                        .strong()
                        .color(colors::ACCENT_RED),
                );
                ui.label(
                    egui::RichText::new(format!("{:.1}%", spike.peak_ram_pct))
                        .monospace()
                        .size(14.0)
                        .color(colors::ACCENT_AMBER),
                );
                ui.label(
                    egui::RichText::new(format!("at: {}", spike.timestamp))
                        .monospace()
                        .size(10.0)
                        .color(colors::TEXT_SECONDARY),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new(format!(
                            "Largest consumer: {} [PID {}] • {}",
                            spike.culprit_name,
                            spike.culprit_pid,
                            format_bytes(spike.culprit_bytes)
                        ))
                        .monospace()
                        .size(11.0)
                        .color(colors::TEXT_PRIMARY),
                    );
                });
            });
        });
}

/// Renders search, clone grouping, and the visible/total counter.
pub(super) fn render_toolbar(
    ui: &mut egui::Ui,
    total: usize,
    shown: usize,
    clone_groups: usize,
    state: &mut MemoryTableState,
) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("🔍").size(14.0));
        ui.add(
            egui::TextEdit::singleline(&mut state.search_query)
                .desired_width(220.0)
                .hint_text("Search: name or PID"),
        );
        if !state.search_query.is_empty() && ui.button("✕").clicked() {
            state.search_query.clear();
        }
        ui.separator();
        ui.checkbox(&mut state.group_clones, format!("Group clones [{clone_groups}x]"));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                egui::RichText::new(format!("Shown: {shown}/{total}"))
                    .monospace()
                    .size(11.0)
                    .color(colors::TEXT_SECONDARY),
            );
        });
    });
}

/// Empty-state text.
pub(super) fn empty_label() -> &'static str {
    "No processes match this query"
}

/// Column header labels: name, PID, physical RAM, virtual memory, RAM percent.
pub(super) fn header_labels() -> [&'static str; 5] {
    ["Process", "PID", "Physical RAM", "Virtual", "RAM %"]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_labels_localized() {
        assert_eq!(header_labels()[0], "Process");
        assert_eq!(header_labels()[4], "RAM %");
        assert_eq!(empty_label(), "No processes match this query");
    }
}
