//! Virtualized table body for the memory process list.
use super::table_state::MemoryTableState;
use super::table_widgets::header_labels;
use crate::modules::memory::formatters::format_bytes;
use crate::modules::memory::process_model::{MemorySortCol, ProcessMemoryItem};
use crate::theme::colors;
use eframe::egui;
use egui_extras::{Column, TableBuilder};

pub(super) fn render_table(
    ui: &mut egui::Ui,
    rows: &[ProcessMemoryItem],
    state: &mut MemoryTableState,
    height: f32,
) {
    let labels = header_labels();
    let max_body_height = (ui.available_height() - 48.0).max(60.0);
    TableBuilder::new(ui)
        .striped(true)
        .resizable(true)
        .sense(egui::Sense::click())
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .auto_shrink([false, false])
        .min_scrolled_height(height)
        .max_scroll_height(max_body_height)
        .column(Column::remainder().at_least(180.0))
        .column(Column::initial(75.0))
        .column(Column::initial(120.0))
        .column(Column::initial(120.0))
        .column(Column::initial(95.0))
        .header(26.0, |mut header| {
            for (idx, col) in [
                MemorySortCol::Name,
                MemorySortCol::Pid,
                MemorySortCol::RamBytes,
                MemorySortCol::VirtualBytes,
                MemorySortCol::RamPct,
            ]
            .into_iter()
            .enumerate()
            {
                header.col(|ui| {
                    let active = state.sort_col == col;
                    if sort_header(ui, labels[idx], active, state.sort_asc) {
                        toggle_sort(state, col);
                    }
                });
            }
        })
        .body(|body| {
            body.rows(24.0, rows.len(), |mut row| {
                let item = &rows[row.index()];
                let mut clicked = false;
                row.set_selected(state.is_row_selected(item));
                let (_, resp) = row.col(|ui| render_name_cell(ui, item));
                clicked |= resp.clicked();
                let (_, resp) = row.col(|ui| {
                    ui.label(
                        egui::RichText::new(item.pid.to_string())
                            .monospace()
                            .size(11.0)
                            .color(colors::TEXT_SECONDARY),
                    );
                });
                clicked |= resp.clicked();
                let (_, resp) = row.col(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(format_bytes(item.ram_bytes))
                                .monospace()
                                .size(11.0)
                                .color(crate::theme::load_color(item.ram_pct)),
                        );
                        if item.delta_bytes_per_sec.abs() >= 1_048_576 {
                            let delta_mib = item.delta_bytes_per_sec as f64 / 1_048_576.0;
                            let (symbol, color) = if item.delta_bytes_per_sec > 0 {
                                ("▲", colors::ACCENT_AMBER)
                            } else {
                                ("▼", egui::Color32::from_rgb(0x00, 0xE0, 0x80))
                            };
                            ui.label(
                                egui::RichText::new(format!(" {}{:+.1} M/s", symbol, delta_mib))
                                    .monospace()
                                    .size(9.0)
                                    .color(color),
                            );
                        }
                    });
                });
                clicked |= resp.clicked();
                let (_, resp) = row.col(|ui| {
                    ui.label(
                        egui::RichText::new(format_bytes(item.virtual_bytes))
                            .monospace()
                            .size(11.0)
                            .color(colors::TEXT_PRIMARY),
                    );
                });
                clicked |= resp.clicked();
                let (_, resp) = row.col(|ui| {
                    ui.label(
                        egui::RichText::new(format!("{:.1}%", item.ram_pct))
                            .monospace()
                            .size(11.0)
                            .color(crate::theme::load_color(item.ram_pct)),
                    );
                });
                clicked |= resp.clicked();
                // Any of the five cells selects the row; the row union is a fallback.
                if clicked || row.response().clicked() {
                    state.toggle_selection(item);
                }
            });
        });
}

fn render_name_cell(ui: &mut egui::Ui, item: &ProcessMemoryItem) {
    ui.label(
        egui::RichText::new(&item.name).monospace().size(12.0).strong().color(colors::TEXT_PRIMARY),
    );
    if item.process_count > 1 {
        ui.label(
            egui::RichText::new(format!("[{}x]", item.process_count))
                .monospace()
                .size(10.0)
                .color(colors::ACCENT_CYAN)
                .background_color(colors::ACCENT_CYAN.gamma_multiply(0.12)),
        );
    } else if item.is_hog {
        egui::Frame::none()
            .fill(egui::Color32::from_rgb(0x3A, 0x10, 0x1D))
            .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(0xE0, 0x2A, 0x54)))
            .rounding(egui::Rounding::same(3.0))
            .inner_margin(egui::Margin::symmetric(5.0, 1.0))
            .show(ui, |ui| {
                ui.label(
                    egui::RichText::new("▲ HOG")
                        .monospace()
                        .size(9.0)
                        .color(egui::Color32::from_rgb(0xE0, 0x2A, 0x54)),
                );
            });
    }
}

fn sort_header(ui: &mut egui::Ui, text: &str, active: bool, asc: bool) -> bool {
    let label = if active {
        format!("{text} {}", if asc { "▲" } else { "▼" })
    } else {
        text.to_owned()
    };
    ui.add(
        egui::Button::new(egui::RichText::new(label).monospace().size(10.0).color(if active {
            colors::ACCENT_CYAN
        } else {
            colors::TEXT_SECONDARY
        }))
        .frame(false),
    )
    .clicked()
}

pub(super) fn toggle_sort(state: &mut MemoryTableState, col: MemorySortCol) {
    if state.sort_col == col {
        state.sort_asc = !state.sort_asc;
    } else {
        state.sort_col = col;
        state.sort_asc = matches!(col, MemorySortCol::Name | MemorySortCol::Pid);
    }
}
