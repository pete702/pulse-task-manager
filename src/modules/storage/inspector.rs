//! Bottom: full-width top process disk activity.
use super::view_helpers::truncate;
use crate::modules::cpu::model::RawProcessCpuRow;
use crate::modules::memory::formatters::format_bytes;
use crate::modules::storage::inspector_group::{cached_consumers, filter_on, last_scan};
use crate::modules::storage::model::{DiskVolumeInfo, StorageSnapshot};
use crate::modules::storage::physical_disk::drive_letter;
use crate::theme::{card_frame, colors};
use eframe::egui;
use std::sync::Arc;
/// Full-width activity across 100% of the pane at all times.
pub fn render_selection_split(
    ui: &mut egui::Ui,
    snap: &StorageSnapshot,
    raw_processes: &[RawProcessCpuRow],
    selected: &Option<Arc<str>>,
) {
    let vol = snap.volumes.iter().find(|v| selected.as_ref() == Some(&v.mount_point)).cloned();
    let disk_volumes: Vec<DiskVolumeInfo> = match vol.as_ref() {
        Some(v) => match v.physical_disk {
            Some(d) => {
                snap.volumes.iter().filter(|x| x.physical_disk == Some(d)).cloned().collect()
            }
            None => vec![v.clone()],
        },
        None => Vec::new(),
    };
    let letters = disk_letters(&disk_volumes);
    render_activity_card(ui, raw_processes, &letters, selected);
}
fn disk_letters(volumes: &[DiskVolumeInfo]) -> Vec<char> {
    let mut out: Vec<char> = Vec::new();
    for v in volumes {
        if let Some(letter) = drive_letter(&v.mount_point) {
            if !out.contains(&letter) {
                out.push(letter);
            }
        }
    }
    out
}
fn render_activity_card(
    ui: &mut egui::Ui,
    raw: &[RawProcessCpuRow],
    letters: &[char],
    selected: &Option<Arc<str>>,
) {
    card_frame().show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.set_min_height((ui.available_height() - 4.0).max(150.0));
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("TOP PROCESS DISK ACTIVITY")
                    .monospace()
                    .size(12.0)
                    .color(colors::TEXT_PRIMARY),
            );
            if selected.is_some() && !letters.is_empty() {
                let scope: Vec<String> = letters.iter().map(|c| format!("{c}:")).collect();
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let filter = *filter_on().lock().unwrap_or_else(|e| e.into_inner());
                    let on_label = format!("On {}", scope.join(","));
                    let on_color =
                        if filter { colors::ACCENT_CYAN } else { colors::TEXT_SECONDARY };
                    if ui
                        .add(egui::Button::new(
                            egui::RichText::new(on_label).monospace().size(10.0).color(on_color),
                        ))
                        .clicked()
                    {
                        *filter_on().lock().unwrap_or_else(|e| e.into_inner()) = !filter;
                    }
                    let all_color =
                        if filter { colors::TEXT_SECONDARY } else { colors::ACCENT_CYAN };
                    if ui
                        .add(egui::Button::new(
                            egui::RichText::new("All Drives")
                                .monospace()
                                .size(10.0)
                                .color(all_color),
                        ))
                        .clicked()
                    {
                        *filter_on().lock().unwrap_or_else(|e| e.into_inner()) = false;
                    }
                });
            }
        });
        ui.add_space(4.0);
        let filter_on_flag = *filter_on().lock().unwrap_or_else(|e| e.into_inner());
        let active: Option<&[char]> = if filter_on_flag && selected.is_some() && !letters.is_empty()
        {
            Some(letters)
        } else {
            None
        };
        render_consumers(ui, raw, active);
    });
}
fn draw_table_row(
    ui: &mut egui::Ui,
    widths: [f32; 4],
    cells: [(&str, egui::Color32); 4],
    is_header: bool,
) {
    ui.horizontal(|ui| {
        ui.style_mut().spacing.item_spacing.x = 8.0;
        for (i, ((text, col), col_w)) in cells.iter().zip(widths.iter()).enumerate() {
            let layout = if i == 0 {
                egui::Layout::left_to_right(egui::Align::Center)
            } else {
                egui::Layout::right_to_left(egui::Align::Center)
            };
            ui.allocate_ui_with_layout(egui::vec2(*col_w, 18.0), layout, |ui| {
                ui.set_min_width(*col_w);
                ui.set_max_width(*col_w);
                let sz = if is_header { 10.0 } else { 11.0 };
                ui.label(egui::RichText::new(*text).monospace().size(sz).color(*col));
            });
        }
    });
}
fn render_consumers(ui: &mut egui::Ui, raw: &[RawProcessCpuRow], filter: Option<&[char]>) {
    ui.set_min_height(150.0);
    let top = cached_consumers(raw, filter);
    let active = top.iter().any(|g| g.total >= 0.05);
    if !active {
        let ago = last_scan()
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .map(|t| t.elapsed().as_secs_f64())
            .unwrap_or(-1.0);
        let ago_txt = if ago < 0.0 { "just now".to_owned() } else { format!("{ago:.0}s ago") };
        let empty = format!(
            "No active process disk I/O (>= 0.05 MB/s). Last scan: {ago_txt} (monitoring {} processes)",
            raw.len()
        );
        ui.label(egui::RichText::new(empty).monospace().size(11.0).color(colors::TEXT_SECONDARY));
        return;
    }
    let row_w = ui.available_width().max(320.0);
    let w_read = 88.0;
    let w_write = 88.0;
    let w_ram = 92.0;
    let w_proc = (row_w - w_read - w_write - w_ram - 24.0).max(140.0);
    let widths = [w_proc, w_read, w_write, w_ram];
    draw_table_row(
        ui,
        widths,
        [
            ("PROCESS", colors::TEXT_SECONDARY),
            ("READ MB/s", colors::TEXT_SECONDARY),
            ("WRITE MB/s", colors::TEXT_SECONDARY),
            ("RAM", colors::TEXT_SECONDARY),
        ],
        true,
    );
    let (sep, _) =
        ui.allocate_exact_size(egui::vec2(row_w, 1.0), egui::Sense::focusable_noninteractive());
    ui.painter().rect_filled(sep, 0.0, crate::theme::colors::BORDER_SUBTLE);
    ui.add_space(4.0);
    for g in top.iter() {
        let label = if g.count > 1 { format!("{} (x{})", g.name, g.count) } else { g.name.clone() };
        let badge = g.drive.map(|d| format!(" {d}:")).unwrap_or_default();
        let short = truncate(&format!("{label}{badge}"), 38);
        let read_s = format!("{:.1}", g.read);
        let write_s = format!("{:.1}", g.write);
        let ram_s = format_bytes(g.ram_bytes);
        draw_table_row(
            ui,
            widths,
            [
                (short.as_str(), colors::TEXT_PRIMARY),
                (read_s.as_str(), colors::ACCENT_CYAN),
                (write_s.as_str(), colors::ACCENT_AMBER),
                (ram_s.as_str(), colors::TEXT_SECONDARY),
            ],
            false,
        );
    }
}
