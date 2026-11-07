use super::table_context_menu::{handle_double_click, render_context_menu};
use super::table_name_cell::render_name_cell;
use super::table_sort::{sort_header, toggle_sort};
use super::table_state::MemoryTableState;
use super::table_widgets::header_labels;
use crate::modules::memory::action_bar::MemoryActionBarState;
use crate::modules::memory::constants::HEAT_BAR_FULL_SCALE_PCT;
use crate::modules::memory::process_item::ProcessMemoryItem;
use crate::modules::memory::process_model::MemorySortCol;
use crate::theme::colors;
use crate::ui::primitives::draw_cell_heat_bar;
use eframe::egui;
use egui_extras::{Column, TableBuilder};

/// High-contrast text colors for table cells with heat bars
const HOG_TEXT_COLOR: egui::Color32 = egui::Color32::from_rgb(241, 245, 249); // ice-white for top consumers
const NORMAL_TEXT_COLOR: egui::Color32 = egui::Color32::from_rgb(203, 213, 225); // bright slate for normal rows

#[allow(clippy::too_many_arguments)]
pub(super) fn render_table(
    ui: &mut egui::Ui,
    rows: &[ProcessMemoryItem],
    state: &mut MemoryTableState,
    height: f32,
    action_state: &mut MemoryActionBarState,
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
                        egui::RichText::new(&*item.pid_text)
                            .monospace()
                            .size(11.0)
                            .color(colors::TEXT_SECONDARY),
                    );
                });
                clicked |= resp.clicked();
                let (_, resp) = row.col(|ui| {
                    // Physical RAM column with heat bar - add 8px left padding for text
                    let cell_rect = ui.max_rect();
                    let ram_ratio = (item.ram_pct / HEAT_BAR_FULL_SCALE_PCT).clamp(0.0, 1.0);
                    let is_alert = item.is_hog;
                    draw_cell_heat_bar(ui, cell_rect, ram_ratio, colors::ACCENT_CYAN, is_alert);
                    // Add 8px left padding so text doesn't overlap the 2px accent strip
                    ui.add_space(8.0);
                    let ram_text_color = if is_alert { HOG_TEXT_COLOR } else { NORMAL_TEXT_COLOR };
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(&*item.ram_text)
                                .monospace()
                                .size(11.0)
                                .color(ram_text_color),
                        );
                        if item.delta_bytes_per_sec.abs() >= 1_048_576 {
                            ui.add_space(4.0); // clear gap after ram_text
                            ui.label(
                                egui::RichText::new(&*item.delta_text).monospace().size(9.0).color(
                                    if item.delta_bytes_per_sec > 0 {
                                        colors::ACCENT_AMBER
                                    } else {
                                        egui::Color32::from_rgb(0x00, 0xE0, 0x80)
                                    },
                                ),
                            );
                        }
                    });
                });
                clicked |= resp.clicked();
                let (_, resp) = row.col(|ui| {
                    ui.label(
                        egui::RichText::new(&*item.virt_text)
                            .monospace()
                            .size(11.0)
                            .color(colors::TEXT_PRIMARY),
                    );
                });
                clicked |= resp.clicked();
                let (_, resp) = row.col(|ui| {
                    // RAM % column with heat bar - add 8px left padding for text
                    let cell_rect = ui.max_rect();
                    let pct_ratio = (item.ram_pct / 100.0).clamp(0.0, 1.0);
                    let is_alert = item.is_hog;
                    draw_cell_heat_bar(ui, cell_rect, pct_ratio, colors::ACCENT_CYAN, is_alert);
                    // Add 8px left padding so text doesn't overlap the 2px accent strip
                    ui.add_space(8.0);
                    let pct_text_color = if is_alert { HOG_TEXT_COLOR } else { NORMAL_TEXT_COLOR };
                    ui.label(
                        egui::RichText::new(&*item.pct_text)
                            .monospace()
                            .size(11.0)
                            .color(pct_text_color),
                    );
                });
                clicked |= resp.clicked();
                if clicked || row.response().clicked() {
                    state.toggle_selection(item);
                }
                if row.response().double_clicked() {
                    handle_double_click(item, action_state);
                }
                row.response().context_menu(|ui| {
                    render_context_menu(ui, item, state, action_state);
                });
            });
        });
}
