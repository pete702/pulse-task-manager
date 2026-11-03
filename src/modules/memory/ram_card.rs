//! RAM card rendering for PULSE memory HUD.

use eframe::egui;

use crate::i18n::Key;
use crate::modules::memory::history::MemoryHistory;
use crate::modules::memory::model::MemorySnapshot;
use crate::theme::{card_frame, colors};

/// Renders the Physical RAM card.
pub fn render_ram_card(ui: &mut egui::Ui, mem: &MemorySnapshot, history: &MemoryHistory) {
    card_frame().show(ui, |ui| {
        ui.set_width(ui.available_width());
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

            // Progress bar
            let pct = mem.ram_usage_percent() / 100.0;
            let bar_color = crate::theme::load_color(mem.ram_usage_percent());
            ui.add(
                egui::ProgressBar::new(pct)
                    .fill(bar_color)
                    .animate(false)
                    .rounding(egui::Rounding::same(3.0)),
            );
            ui.add_space(12.0);

            // Grid of values (4 rows to match Swap/Commit card)
            egui::Grid::new("ram_grid")
                .num_columns(2)
                .spacing([16.0, 8.0])
                .show(ui, |ui| {
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

                    // Available / Free
                    ui.label(
                        egui::RichText::new(Key::AvailableFree.t())
                            .monospace()
                            .size(12.0)
                            .color(colors::TEXT_SECONDARY),
                    );
                    ui.label(
                        egui::RichText::new(format!(
                            "{:.2} GiB / {:.2} GiB",
                            mem.ram_available_gib(),
                            mem.ram_free_bytes as f64 / 1_073_741_824.0
                        ))
                        .monospace()
                        .size(13.0)
                        .color(colors::TEXT_PRIMARY),
                    );
                    ui.end_row();

                    // Cached
                    ui.label(
                        egui::RichText::new(Key::CachedStandby.t())
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
