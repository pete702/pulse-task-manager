//! RAM card rendering for PULSE memory HUD.
use crate::i18n::Key;
use crate::modules::memory::history::MemoryHistory;
use crate::modules::memory::model::MemorySnapshot;
use crate::theme::{card_frame, colors};
use crate::ui::primitives::draw_segmented_led_bar;
use eframe::egui;

/// Renders the Physical RAM card.
pub fn render_ram_card(ui: &mut egui::Ui, mem: &MemorySnapshot, history: &MemoryHistory) {
    card_frame().show(ui, |ui| {
        ui.vertical(|ui| {
            // Header
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(Key::PhysicalRamTitle.t())
                        .monospace()
                        .size(13.0)
                        .color(colors::TEXT_PRIMARY),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let pct = mem.ram_usage_percent();
                    let color = crate::theme::load_color(pct);
                    ui.label(
                        egui::RichText::new(format!("{:.1}%", pct))
                            .monospace()
                            .size(22.0)
                            .color(color),
                    );
                });
            });
            ui.add_space(8.0);

            // Segmented LED bar (28 segments) — hardware telemetry style
            let bar_w = ui.available_width().max(80.0);
            let pct = mem.ram_usage_percent() / 100.0;
            let (rect, resp) = ui.allocate_exact_size(egui::vec2(bar_w, 16.0), egui::Sense::hover());
            resp.on_hover_text("Physical RAM usage: In-use memory vs total installed RAM. >85% = critical pressure, 70-85% = elevated, <70% = optimal");
            draw_segmented_led_bar(ui, rect, pct, 28, colors::ACCENT_CYAN);
            ui.add_space(12.0);

            // Grid of values (4 rows to match Swap/Commit card)
            egui::Grid::new("ram_grid").num_columns(2).spacing([16.0, 8.0]).show(ui, |ui| {
                // Used / Total
                ui.label(
                    egui::RichText::new(Key::UsedTotal.t())
                        .monospace()
                        .size(12.0)
                        .color(colors::TEXT_SECONDARY),
                );
                ui.label(
                    egui::RichText::new(format!(
                        "{:.2} / {:.2} GiB",
                        mem.ram_used_gib(),
                        mem.ram_total_gib()
                    ))
                    .monospace()
                    .size(13.0)
                    .color(colors::TEXT_PRIMARY),
                );
                ui.end_row();

                // Available RAM
                ui.label(
                    egui::RichText::new("Available RAM")
                        .monospace()
                        .size(12.0)
                        .color(colors::TEXT_SECONDARY),
                );
                ui.label(
                    egui::RichText::new(format!(
                        "{:.2} GiB ({:.1}%)",
                        mem.ram_available_gib(),
                        if mem.ram_total_bytes > 0 {
                            (mem.ram_available_bytes as f64 / mem.ram_total_bytes as f64) * 100.0
                        } else {
                            0.0
                        }
                    ))
                    .monospace()
                    .size(13.0)
                    .color(colors::TEXT_PRIMARY),
                );
                ui.end_row();

                // System Cache (OS)
                ui.label(
                    egui::RichText::new("System Cache (OS)")
                        .monospace()
                        .size(12.0)
                        .color(colors::TEXT_SECONDARY),
                );
                ui.label(
                    egui::RichText::new(format!("{:.2} GiB", mem.ram_cached_gib()))
                        .monospace()
                        .size(13.0)
                        .color(colors::ACCENT_CYAN),
                );
                ui.end_row();

                // Peak
                ui.label(
                    egui::RichText::new(Key::PeakSession.t())
                        .monospace()
                        .size(12.0)
                        .color(colors::TEXT_SECONDARY),
                );
                ui.label(
                    egui::RichText::new(format!("{:.1}%", history.peak_ram_percent))
                        .monospace()
                        .size(13.0)
                        .color(colors::ACCENT_RED),
                );
                ui.end_row();
            });
        });
    });
}
