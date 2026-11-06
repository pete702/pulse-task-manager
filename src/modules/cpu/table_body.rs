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
    let avail_w = ui.available_width().max(480.0);
    let width_bucket = (avail_w / 8.0).round() as u32;
    ui.push_id(("proc_table", width_bucket), |ui| {
        TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .auto_shrink([false, false])
            .sense(egui::Sense::click())
            .min_scrolled_height(height.min(max_body))
            .max_scroll_height(max_body)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(
                Column::initial((avail_w * 0.11).clamp(64.0, 140.0))
                    .range(56.0..=200.0)
                    .resizable(true)
                    .clip(true),
            )
            .column(Column::remainder().at_least(110.0).resizable(true).clip(true))
            .column(
                Column::initial((avail_w * 0.24).clamp(155.0, 280.0))
                    .range(145.0..=340.0)
                    .resizable(true)
                    .clip(true),
            )
            .column(
                Column::initial((avail_w * 0.11).clamp(64.0, 140.0))
                    .range(56.0..=200.0)
                    .resizable(true)
                    .clip(true),
            )
            .column(
                Column::initial((avail_w * 0.13).clamp(74.0, 160.0))
                    .range(64.0..=220.0)
                    .resizable(true)
                    .clip(true),
            )
            .column(
                Column::initial((avail_w * 0.16).clamp(90.0, 190.0))
                    .range(80.0..=240.0)
                    .resizable(true)
                    .clip(true),
            )
            .column(
                Column::initial((avail_w * 0.13).clamp(80.0, 180.0))
                    .range(70.0..=260.0)
                    .resizable(true)
                    .clip(true),
            )
            .header(24.0, |mut header| {
                for (label, col) in [
                    ("PID", CpuSortCol::Pid),
                    ("PROCESS NAME", CpuSortCol::Name),
                    ("CLONES", CpuSortCol::CloneCount),
                    ("CPU %", CpuSortCol::CpuPct),
                    ("MEMORY (RAM)", CpuSortCol::RamBytes),
                    ("10s DELTA", CpuSortCol::Delta),
                    ("DISK I/O", CpuSortCol::DiskIo),
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
                            egui::RichText::new(item.pid_text.as_ref())
                                .monospace()
                                .size(11.0)
                                .color(if selected {
                                    colors::ACCENT_CYAN
                                } else {
                                    colors::TEXT_SECONDARY
                                }),
                        );
                    });

                    row.col(|ui| {
                        ui.add_space(8.0);
                        let resp = ui.label(
                            egui::RichText::new(item.name.as_ref()).monospace().size(11.0).color(
                                if selected { colors::ACCENT_CYAN } else { colors::TEXT_PRIMARY },
                            ),
                        );
                        if resp.hovered() {
                            let p = crate::modules::cpu::exe_cache::cached_exe_path(
                                item.pid,
                                item.start_time,
                            );
                            resp.on_hover_text(p);
                        }
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

                    row.col(|ui| {
                        let total_io = item.disk_read_mb_s + item.disk_write_mb_s;
                        let color = if total_io >= 20.0 {
                            colors::ACCENT_AMBER
                        } else if total_io >= 0.1 {
                            colors::ACCENT_CYAN
                        } else {
                            egui::Color32::from_rgb(92, 104, 122)
                        };
                        ui.label(
                            egui::RichText::new(item.disk_io_text.as_ref())
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
    });
}
