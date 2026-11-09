//! Unified process table body with sortable headers, bars, and row actions.
use crate::modules::cpu::model::ProcessCpuItem;
use crate::modules::cpu::table_cols::{col_widths, compute_col_x, draw_separators, truncate_name};
use crate::modules::cpu::table_menu::{render_sort_button, show_row_menu};
use crate::modules::cpu::table_state::{CpuSortCol, CpuTableState};
use crate::theme::colors;
use crate::ui::primitives::draw_cell_heat_bar;
use eframe::egui;

const DELTA_ALERT_BYTES: i64 = 50 * 1024 * 1024;
const EMERALD: egui::Color32 = egui::Color32::from_rgb(0x00, 0xE0, 0x80);
const HDR: [(&str, CpuSortCol); 7] = [
    ("PID", CpuSortCol::Pid),
    ("PROCESS NAME", CpuSortCol::Name),
    ("CLONES", CpuSortCol::CloneCount),
    ("CPU %", CpuSortCol::CpuPct),
    ("MEMORY (RAM)", CpuSortCol::RamBytes),
    ("10s DELTA", CpuSortCol::Delta),
    ("DISK I/O", CpuSortCol::DiskIo),
];

fn cell_r(col_x: &[f32; 7], col_w: &[f32; 7], i: usize, top: f32, bot: f32) -> egui::Rect {
    egui::Rect::from_min_max(
        egui::pos2(col_x[i] + 6.0, top),
        egui::pos2(col_x[i] + col_w[i] - 6.0, bot),
    )
}
fn txt(ui: &egui::Ui, r: egui::Rect, s: &str, c: egui::Color32) {
    ui.painter().with_clip_rect(r).text(
        r.left_center(),
        egui::Align2::LEFT_CENTER,
        s,
        egui::FontId::monospace(11.0),
        c,
    );
}

/// Renders the full-width virtualized process table.
pub fn render_virtualized_table(
    ui: &mut egui::Ui,
    rows: &[ProcessCpuItem],
    state: &mut CpuTableState,
    height: f32,
) {
    let prev_selectable = ui.style().interaction.selectable_labels;
    ui.style_mut().interaction.selectable_labels = false;
    let row_h = if state.comfort_density { 28.0 } else { 22.0 };
    let max_body = (ui.available_height() - 48.0).max(60.0);
    let scroll_h = height.min(max_body);
    let raw_avail = ui.available_width();
    let mut widths = col_widths(ui, raw_avail);
    let fitted_w: f32 = widths.iter().sum();
    let (header_rect, _) = ui.allocate_exact_size(egui::vec2(fitted_w, 24.0), egui::Sense::hover());
    let col_x = compute_col_x(header_rect.left(), &widths);
    for (i, (label, sc)) in HDR.iter().enumerate() {
        let hr = cell_r(&col_x, &widths, i, header_rect.top(), header_rect.bottom());
        let mut hui = ui.child_ui(hr, egui::Layout::left_to_right(egui::Align::Center));
        if render_sort_button(&mut hui, label, state.sort_col == *sc, state.sort_asc).clicked() {
            state.set_sort(*sc);
        }
    }
    let tbl_r = egui::Rect::from_min_max(
        header_rect.min,
        egui::pos2(header_rect.right(), header_rect.bottom() + scroll_h),
    );
    let max_ram =
        rows.iter().map(|r| r.ram_bytes).max().unwrap_or(512 * 1024 * 1024).max(512 * 1024 * 1024)
            as f32;
    egui::ScrollArea::vertical().max_height(scroll_h).auto_shrink([false, false]).show_rows(
        ui,
        row_h,
        rows.len(),
        |ui, row_range| {
            for idx in row_range {
                let item = &rows[idx];
                let selected = state.selected_pid == Some(item.pid);
                let (rr, resp) =
                    ui.allocate_exact_size(egui::vec2(fitted_w, row_h), egui::Sense::click());
                if selected {
                    ui.painter().rect_filled(
                        rr,
                        egui::Rounding::same(2.0),
                        egui::Color32::from_rgba_unmultiplied(24, 92, 118, 120),
                    );
                    ui.painter().rect_stroke(
                        rr,
                        egui::Rounding::same(2.0),
                        egui::Stroke::new(1.0_f32, colors::ACCENT_CYAN),
                    );
                } else if resp.hovered() {
                    ui.painter().rect_filled(
                        rr,
                        egui::Rounding::same(2.0),
                        egui::Color32::from_rgba_unmultiplied(38, 52, 68, 95),
                    );
                } else if idx % 2 == 1 {
                    ui.painter().rect_filled(
                        rr,
                        egui::Rounding::same(2.0),
                        egui::Color32::from_rgba_unmultiplied(18, 24, 32, 70),
                    );
                }
                if resp.clicked() || resp.secondary_clicked() {
                    state.selected_pid = Some(item.pid);
                    state.selected_start_time = Some(item.start_time);
                    state.selected_name = Some(item.name.to_string());
                }
                let top = rr.top();
                let bot = rr.bottom();
                let c0 = cell_r(&col_x, &widths, 0, top, bot);
                let cc = if selected { colors::ACCENT_CYAN } else { colors::TEXT_SECONDARY };
                txt(ui, c0, &item.pid_text, cc);
                let c1 = cell_r(&col_x, &widths, 1, top, bot);
                let shown = truncate_name(&item.name, widths[1]);
                let nc = if selected { colors::ACCENT_CYAN } else { colors::TEXT_PRIMARY };
                txt(ui, c1, &shown, nc);
                if item.is_cpu_hog {
                    let hog = egui::Rect::from_min_max(
                        egui::pos2(c1.right() - 34.0, top),
                        egui::pos2(c1.right(), bot),
                    );
                    ui.painter().with_clip_rect(hog).text(
                        hog.right_center(),
                        egui::Align2::RIGHT_CENTER,
                        "[HOG]",
                        egui::FontId::monospace(10.0),
                        colors::ACCENT_RED,
                    );
                }
                let c2 = cell_r(&col_x, &widths, 2, top, bot);
                let ccc =
                    if item.clone_count > 1 { colors::ACCENT_CYAN } else { colors::TEXT_SECONDARY };
                txt(ui, c2, &item.clone_text, ccc);
                let c3 = cell_r(&col_x, &widths, 3, top, bot);
                if item.cpu_pct >= 0.5 {
                    draw_cell_heat_bar(
                        ui,
                        c3,
                        item.cpu_pct / 100.0,
                        colors::ACCENT_CYAN,
                        item.is_cpu_hog,
                    );
                }
                let cpu_c = if item.cpu_pct >= 35.0 {
                    colors::ACCENT_AMBER
                } else if item.cpu_pct >= 10.0 {
                    colors::ACCENT_CYAN
                } else if item.cpu_pct >= 0.1 {
                    colors::TEXT_PRIMARY
                } else {
                    egui::Color32::from_rgb(92, 104, 122)
                };
                txt(ui, c3, &item.cpu_text, cpu_c);
                let bw = (widths[4] - 14.0).max(20.0);
                let ram_bar = egui::Rect::from_min_max(
                    egui::pos2(col_x[4] + 6.0, top),
                    egui::pos2(col_x[4] + 6.0 + bw, bot),
                );
                let ratio = (item.ram_bytes as f32 / max_ram).clamp(0.0, 1.0);
                if ratio * 100.0 >= 0.5 {
                    draw_cell_heat_bar(ui, ram_bar, ratio, colors::ACCENT_CYAN, false);
                }
                let c4 = cell_r(&col_x, &widths, 4, top, bot);
                txt(ui, c4, &item.ram_text, colors::TEXT_PRIMARY);
                let c5 = cell_r(&col_x, &widths, 5, top, bot);
                let stable = item.ram_delta_10s.abs() < 512 * 1024;
                let dc = if stable {
                    colors::TEXT_SECONDARY
                } else if item.ram_delta_10s < 0 {
                    EMERALD
                } else if item.ram_delta_10s >= DELTA_ALERT_BYTES {
                    colors::ACCENT_RED
                } else {
                    colors::ACCENT_AMBER
                };
                txt(ui, c5, &item.delta_text, dc);
                let c6 = cell_r(&col_x, &widths, 6, top, bot);
                let total_io = item.disk_read_mb_s + item.disk_write_mb_s;
                let ic = if total_io >= 20.0 {
                    colors::ACCENT_AMBER
                } else if total_io >= 0.1 {
                    colors::ACCENT_CYAN
                } else {
                    egui::Color32::from_rgb(92, 104, 122)
                };
                txt(ui, c6, &item.disk_io_text, ic);
                let hov = resp.hovered();
                let resp = if hov {
                    let p = if !item.exe_path.is_empty() {
                        item.exe_path.to_string()
                    } else {
                        crate::modules::cpu::exe_cache::cached_exe_path(item.pid, item.start_time)
                    };
                    resp.on_hover_text(format!("{}\n{}", item.name, p))
                } else {
                    resp
                };
                resp.on_hover_cursor(egui::CursorIcon::Default).context_menu(|ui| {
                    show_row_menu(ui, item, state);
                });
            }
        },
    );
    draw_separators(ui, tbl_r, &mut widths, rows);
    ui.style_mut().interaction.selectable_labels = prev_selectable;
}
