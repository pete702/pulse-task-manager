//! Unified process table body with sortable headers, bars, and row actions.
use crate::modules::cpu::model::ProcessCpuItem;
use crate::modules::cpu::table_menu::{render_sort_button, show_row_menu};
use crate::modules::cpu::table_state::{CpuSortCol, CpuTableState};
use crate::theme::colors;
use crate::ui::primitives::draw_cell_heat_bar;
use eframe::egui;
use egui_extras::{Column, TableBuilder};

const DELTA_ALERT_BYTES: i64 = 50 * 1024 * 1024;
const EMERALD: egui::Color32 = egui::Color32::from_rgb(0x00, 0xE0, 0x80);

/// Renders the full-width virtualized process table.
pub fn render_virtualized_table(
    ui: &mut egui::Ui,
    rows: &[ProcessCpuItem],
    state: &mut CpuTableState,
    height: f32,
) {
    ui.style_mut().interaction.selectable_labels = false;
    let row_height = if state.comfort_density { 28.0 } else { 22.0 };
    let max_body = (ui.available_height() - 48.0).max(60.0);
    let max_ram = rows.iter().map(|row| row.ram_bytes).max().unwrap_or(1).max(1);
    ui.set_width(ui.available_width());
    let avail_w = ui.available_width();
    let s = (avail_w / 820.0).clamp(0.72, 1.25);
    let r = |min: f32, max: f32, base: f32| Column::initial(base).range(min..=(max * s).max(min));
    TableBuilder::new(ui)
        .striped(true)
        .resizable(true)
        .auto_shrink([false, false])
        .sense(egui::Sense::click())
        .min_scrolled_height(height.min(max_body))
        .max_scroll_height(max_body)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(r(48.0, 110.0, 58.0).resizable(true).clip(true))
        .column(Column::remainder().at_least(96.0).clip(true))
        .column(r(52.0, 110.0, 64.0).resizable(true).clip(true))
        .column(r(62.0, 140.0, 76.0).resizable(true).clip(true))
        .column(r(108.0, 260.0, 136.0).resizable(true).clip(true))
        .column(r(80.0, 160.0, 98.0).resizable(true).clip(true))
        .header(24.0, |mut header| {
            for (label, col) in [
                ("PID", CpuSortCol::Pid),
                ("PROCESS NAME", CpuSortCol::Name),
                ("CLONES", CpuSortCol::CloneCount),
                ("CPU %", CpuSortCol::CpuPct),
                ("MEMORY (RAM)", CpuSortCol::RamBytes),
                ("10s DELTA", CpuSortCol::Delta),
            ] {
                header.col(|ui| {
                    let active = state.sort_col == col;
                    if render_sort_button(ui, label, active, state.sort_asc).clicked() {
                        state.set_sort(col);
                    }
                });
            }
        })
        .body(|body| {
            body.rows(row_height, rows.len(), |mut row| {
                let item = &rows[row.index()];
                let selected = state.selected_pid == Some(item.pid);
                row.set_selected(selected);

                row.col(|ui| {
                    ui.label(
                        egui::RichText::new(item.pid_text.as_ref()).monospace().size(11.0).color(
                            if selected { colors::ACCENT_CYAN } else { colors::TEXT_SECONDARY },
                        ),
                    );
                });

                row.col(|ui| {
                    ui.add_space(8.0);
                    ui.label(
                        egui::RichText::new(item.name.as_ref()).monospace().size(11.0).color(
                            if selected { colors::ACCENT_CYAN } else { colors::TEXT_PRIMARY },
                        ),
                    );
                    if item.is_cpu_hog {
                        ui.label(
                            egui::RichText::new("[HOG]")
                                .monospace()
                                .size(10.0)
                                .color(colors::ACCENT_RED),
                        );
                    }
                });

                row.col(|ui| {
                    let color = if item.clone_count > 1 {
                        colors::ACCENT_CYAN
                    } else {
                        colors::TEXT_SECONDARY
                    };
                    ui.label(
                        egui::RichText::new(item.clone_text.as_ref())
                            .monospace()
                            .size(11.0)
                            .color(color),
                    );
                });

                row.col(|ui| {
                    let rect_for_bar = ui.max_rect();
                    if item.cpu_pct >= 0.5 && ui.is_rect_visible(rect_for_bar) {
                        draw_cell_heat_bar(
                            ui,
                            rect_for_bar,
                            item.cpu_pct / 100.0,
                            colors::ACCENT_CYAN,
                            item.is_cpu_hog,
                        );
                    }
                    ui.add_space(8.0);
                    let color = if item.cpu_pct >= 35.0 {
                        colors::ACCENT_AMBER
                    } else if item.cpu_pct >= 10.0 {
                        colors::ACCENT_CYAN
                    } else if item.cpu_pct >= 0.1 {
                        colors::TEXT_PRIMARY
                    } else {
                        egui::Color32::from_rgb(92, 104, 122)
                    };
                    ui.label(
                        egui::RichText::new(item.cpu_text.as_ref())
                            .monospace()
                            .size(11.0)
                            .color(color),
                    );
                });

                row.col(|ui| {
                    let cap_ref = max_ram.min(4096 * 1024 * 1024);
                    let ratio = (item.ram_bytes as f32 / cap_ref as f32).clamp(0.0, 1.0);
                    let rect_for_bar = ui.max_rect();
                    if ratio * 100.0 >= 0.5 && ui.is_rect_visible(rect_for_bar) {
                        draw_cell_heat_bar(ui, rect_for_bar, ratio, colors::ACCENT_CYAN, false);
                    }
                    ui.add_space(8.0);
                    ui.label(
                        egui::RichText::new(item.ram_text.as_ref())
                            .monospace()
                            .size(11.0)
                            .color(colors::TEXT_PRIMARY),
                    );
                });

                row.col(|ui| {
                    let stable = item.ram_delta_10s.abs() < 512 * 1024;
                    let color = if stable {
                        colors::TEXT_SECONDARY
                    } else if item.ram_delta_10s < 0 {
                        EMERALD
                    } else if item.ram_delta_10s >= DELTA_ALERT_BYTES {
                        colors::ACCENT_RED
                    } else {
                        colors::ACCENT_AMBER
                    };
                    ui.label(
                        egui::RichText::new(item.delta_text.as_ref())
                            .monospace()
                            .size(11.0)
                            .color(color),
                    );
                });

                let response = row.response();
                if response.clicked() || response.secondary_clicked() {
                    state.selected_pid = Some(item.pid);
                    state.selected_name = Some(item.name.to_string());
                }
                response
                    .on_hover_cursor(egui::CursorIcon::Default)
                    .on_hover_text("Right-click for process actions")
                    .context_menu(|ui| {
                        show_row_menu(ui, item, state);
                    });
            });
        });
}
