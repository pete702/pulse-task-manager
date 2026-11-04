use super::table_context_menu::{handle_double_click, render_context_menu};
use super::table_name_cell::render_name_cell;
use super::table_sort::{sort_header, toggle_sort};
use super::table_state::MemoryTableState;
use super::table_widgets::header_labels;
use crate::modules::memory::formatters::format_bytes;
use crate::modules::memory::process_item::ProcessMemoryItem;
use crate::modules::memory::process_model::MemorySortCol;
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
    let avail_h = ui.available_height();
    let max_body_height = (avail_h - 48.0).max(60.0);
    let max_scroll_h = max_body_height;
    TableBuilder::new(ui)
        .striped(true)
        .resizable(true)
        .sense(egui::Sense::click())
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .auto_shrink([false, false])
        .min_scrolled_height(height)
        .max_scroll_height(max_scroll_h)
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
                if clicked || row.response().clicked() {
                    state.toggle_selection(item);
                }
                if row.response().double_clicked() {
                    handle_double_click(item);
                }
                row.response().context_menu(|ui| {
                    render_context_menu(ui, item, state);
                });
            });
        });
}
