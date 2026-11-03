//! Swap & Commit card rendering for PULSE memory HUD.

use eframe::egui;

use crate::i18n::Key;
use crate::modules::memory::formatters::format_bytes;
use crate::modules::memory::model::MemorySnapshot;
use crate::theme::{card_frame, colors};

/// Renders the Swap & Commit card with Commit % as the headline metric.
pub fn render_swap_commit_card(ui: &mut egui::Ui, mem: &MemorySnapshot) {
    card_frame().show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.vertical(|ui| {
            // Header: commit charge is always the headline (swap may legitimately be 0%).
            let commit_pct = mem.commit_charge_pct();
            let commit_color = crate::theme::load_color(commit_pct);
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(Key::SwapCommitTitle.t())
                        .monospace()
                        .size(13.0)
                        .color(colors::TEXT_PRIMARY),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new(format!("{:.1}%", commit_pct))
                            .monospace()
                            .size(22.0)
                            .color(commit_color),
                    )
                    .on_hover_text("Commit charge as a share of the system commit limit");
                });
            });
            ui.add_space(8.0);

            // Progress bar tracks commit charge, not the often-zero pagefile usage.
            ui.add(
                egui::ProgressBar::new(commit_pct / 100.0)
                    .fill(colors::ACCENT_PURPLE)
                    .animate(false)
                    .rounding(egui::Rounding::same(3.0)),
            );
            ui.add_space(12.0);

            // Grid of values (4 rows to match RAM card)
            egui::Grid::new("swap_commit_grid").num_columns(2).spacing([16.0, 8.0]).show(
                ui,
                |ui| {
                    // Commit (Used / Limit) — same metric the purple bar tracks.
                    ui.label(
                        egui::RichText::new(Key::CommitChargeLimit.t())
                            .monospace()
                            .size(12.0)
                            .color(colors::TEXT_SECONDARY),
                    );
                    ui.label(
                        egui::RichText::new(format!(
                            "{:.2} / {:.2} GiB",
                            mem.commit_total_gib(),
                            mem.commit_limit_gib()
                        ))
                        .monospace()
                        .size(13.0)
                        .color(colors::ACCENT_PURPLE),
                    );
                    ui.end_row();

                    // Available Commit
                    ui.label(
                        egui::RichText::new("Available Commit")
                            .monospace()
                            .size(12.0)
                            .color(colors::TEXT_SECONDARY),
                    );
                    let avail_commit =
                        mem.commit_limit_bytes.saturating_sub(mem.commit_total_bytes);
                    ui.label(
                        egui::RichText::new(format_bytes(avail_commit))
                            .monospace()
                            .size(13.0)
                            .color(colors::ACCENT_CYAN),
                    );
                    ui.end_row();

                    // Swap (Used / Total)
                    ui.label(
                        egui::RichText::new(Key::SwapUsedTotal.t())
                            .monospace()
                            .size(12.0)
                            .color(colors::TEXT_SECONDARY),
                    );
                    ui.label(
                        egui::RichText::new(format!(
                            "{:.2} / {:.2} GiB",
                            mem.swap_used_gib(),
                            mem.swap_total_gib()
                        ))
                        .monospace()
                        .size(13.0)
                        .color(colors::TEXT_PRIMARY),
                    );
                    ui.end_row();

                    // Swap Usage %
                    ui.label(
                        egui::RichText::new("Swap Usage %")
                            .monospace()
                            .size(12.0)
                            .color(colors::TEXT_SECONDARY),
                    );
                    ui.label(
                        egui::RichText::new(format!("{:.1}%", mem.swap_usage_percent()))
                            .monospace()
                            .size(13.0)
                            .color(colors::TEXT_PRIMARY),
                    );
                    ui.end_row();
                },
            );
        });
    });
}
