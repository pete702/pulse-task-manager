//! CPU Total Processor Load card.

use crate::modules::cpu::model::CpuSnapshot;
use crate::theme::{card_frame, colors};
use crate::ui::primitives::{cyber_accent, draw_cyber_corner_brackets, draw_segmented_led_bar};
use eframe::egui;

/// Renders the Total Processor Load card.
pub fn render_total_load_card(ui: &mut egui::Ui, cpu: &CpuSnapshot) {
    let total_pct = cpu.total_pct;
    let resp = card_frame().show(ui, |ui| {
        ui.vertical(|ui| {
            // Header: title on its own line, brand below with ellipsis so long
            // Xeon strings never collide with the title on narrow windows.
            ui.add(
                egui::Label::new(
                    egui::RichText::new("TOTAL PROCESSOR LOAD")
                        .monospace()
                        .size(13.0)
                        .color(colors::TEXT_PRIMARY),
                )
                .wrap(false),
            );
            if !cpu.brand.is_empty() {
                ui.add(
                    egui::Label::new(
                        egui::RichText::new(cpu.brand.as_ref())
                            .monospace()
                            .size(10.0)
                            .color(colors::TEXT_SECONDARY),
                    )
                    .wrap(false)
                    .truncate(true),
                );
            }
            ui.add_space(8.0);

            // Primary metric: Total CPU % + precision bar
            let total_pct = cpu.total_pct;
            let color = crate::theme::load_color(total_pct);
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(format!("{total_pct:.1}%"))
                        .monospace()
                        .size(28.0)
                        .color(color),
                );
            });
            ui.add_space(4.0);

            // Precision bar (smooth continuous)
            let bar_w = ui.available_width().max(80.0);
            let (rect, resp) = ui.allocate_exact_size(egui::vec2(bar_w, 16.0), egui::Sense::hover());
            resp.on_hover_text("Total system CPU utilization across all logical threads. 100% = all threads saturated.");
            draw_segmented_led_bar(ui, rect, total_pct / 100.0, 28, colors::ACCENT_CYAN);
            ui.add_space(12.0);

            // Bottom row: 3 mini-tiles via Grid with no-wrap labels
            egui::Grid::new("cpu_total_grid")
                .num_columns(3)
                .spacing([20.0, 6.0])
                .show(ui, |ui| {
                    // LIVE CLOCK
                    ui.vertical(|ui| {
                        ui.add(
                            egui::Label::new(
                                egui::RichText::new("LIVE CLOCK")
                                    .monospace()
                                    .size(10.0)
                                    .color(colors::TEXT_SECONDARY),
                            )
                            .wrap(false),
                        );
                        let live_ghz = cpu.frequency_mhz as f64 / 1000.0;
                        ui.add(
                            egui::Label::new(
                                egui::RichText::new(format!("{live_ghz:.2} GHz"))
                                    .monospace()
                                    .size(12.0)
                                    .color(colors::ACCENT_CYAN),
                            )
                            .wrap(false),
                        )
                        .on_hover_text(format!("{} MHz (live)", cpu.frequency_mhz));
                    });
                    // BASE CLOCK
                    ui.vertical(|ui| {
                        ui.add(
                            egui::Label::new(
                                egui::RichText::new("BASE CLOCK")
                                    .monospace()
                                    .size(10.0)
                                    .color(colors::TEXT_SECONDARY),
                            )
                            .wrap(false),
                        );
                        let base_ghz = cpu.base_mhz as f64 / 1000.0;
                        ui.add(
                            egui::Label::new(
                                egui::RichText::new(format!("{base_ghz:.2} GHz"))
                                    .monospace()
                                    .size(12.0)
                                    .color(colors::TEXT_PRIMARY),
                            )
                            .wrap(false),
                        )
                        .on_hover_text(format!("{} MHz (base)", cpu.base_mhz));
                    });
                    // LOGICAL CORES / PROCS
                    ui.vertical(|ui| {
                        ui.add(
                            egui::Label::new(
                                egui::RichText::new("LOGICAL CORES / PROCS")
                                    .monospace()
                                    .size(10.0)
                                    .color(colors::TEXT_SECONDARY),
                            )
                            .wrap(false),
                        );
                        ui.add(
                            egui::Label::new(
                                egui::RichText::new(format!(
                                    "{} / {}",
                                    cpu.logical_cores, cpu.process_count
                                ))
                                .monospace()
                                .size(12.0)
                                .color(colors::TEXT_PRIMARY),
                            )
                            .wrap(false),
                        )
                        .on_hover_text(format!(
                            "{} logical cores / {} processes",
                            cpu.logical_cores, cpu.process_count
                        ));
                    });
                    ui.end_row();
                });
        });
    });
    draw_cyber_corner_brackets(ui, resp.response.rect, cyber_accent(total_pct));
}
