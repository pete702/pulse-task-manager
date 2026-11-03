//! Swap & Commit card rendering for PULSE memory HUD.

use eframe::egui;

use crate::i18n::Key;
use crate::modules::memory::formatters::format_bytes;
use crate::modules::memory::model::MemorySnapshot;
use crate::theme::{card_frame, colors};

/// Renders the Swap & Commit card.
pub fn render_swap_commit_card(ui: &mut egui::Ui, mem: &MemorySnapshot) {
    card_frame().show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.vertical(|ui| {
            // Header
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(Key::SwapCommitTitle.t())
                        .monospace()
                        .size(13.0)
                        .color(colors::TEXT_PRIMARY),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Use Commit % if swap is 0, otherwise swap %
                    let (pct, color) = if mem.swap_total_bytes > 0 {
                        (
                            mem.swap_usage_percent(),
                            crate::theme::load_color(mem.swap_usage_percent()),
                        )
                    } else {
                        let commit_pct = if mem.commit_limit_bytes > 0 {
                            (mem.commit_total_bytes as f64 / mem.commit_limit_bytes as f64 * 100.0)
                                as f32
                        } else {
                            0.0
                        };
                        (commit_pct, crate::theme::load_color(commit_pct))
                    };
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
            let pct = if mem.swap_total_bytes > 0 {
                mem.swap_usage_percent() / 100.0
            } else if mem.commit_limit_bytes > 0 {
                (mem.commit_total_bytes as f64 / mem.commit_limit_bytes as f64) as f32
            } else {
                0.0
            };
            ui.add(
                egui::ProgressBar::new(pct)
                    .fill(colors::ACCENT_PURPLE)
                    .animate(false)
                    .rounding(egui::Rounding::same(3.0)),
            );
            ui.add_space(12.0);

            // Grid of values (4 rows to match RAM card)
            egui::Grid::new("swap_commit_grid")
                .num_columns(2)
                .spacing([16.0, 8.0])
                .show(ui, |ui| {
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

                    // Commit (Used / Limit)
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
                    let avail_commit = mem
                        .commit_limit_bytes
                        .saturating_sub(mem.commit_total_bytes);
                    ui.label(
                        egui::RichText::new(format_bytes(avail_commit))
                            .monospace()
                            .size(13.0)
                            .color(colors::ACCENT_CYAN),
                    );
                    ui.end_row();

                    // Commit Load
                    ui.label(
                        egui::RichText::new(Key::CommitPercent.t())
                            .monospace()
                            .size(12.0)
                            .color(colors::TEXT_SECONDARY),
                    );
                    let commit_pct = if mem.commit_limit_bytes > 0 {
                        (mem.commit_total_bytes as f64 / mem.commit_limit_bytes as f64 * 100.0)
                            as f32
                    } else {
                        0.0
                    };
                    ui.label(
                        egui::RichText::new(format!("{:.1}%", commit_pct))
                            .monospace()
                            .size(13.0)
                            .color(crate::theme::load_color(commit_pct)),
                    );
                    ui.end_row();
                });
        });
    });
}
