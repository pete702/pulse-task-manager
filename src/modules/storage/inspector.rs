//! Bottom split: selected disk details (left) + process I/O (right).

use std::sync::Arc;

use eframe::egui;

use crate::modules::cpu::model::RawProcessCpuRow;
use crate::modules::memory::formatters::format_bytes;
use crate::modules::storage::inspector_group::{cached_consumers, filter_on, last_scan};
use crate::modules::storage::model::{DiskVolumeInfo, StorageSnapshot};
use crate::modules::storage::physical_disk::drive_letter;
use crate::modules::storage::volume_cards::health_label;
use crate::theme::{card_frame, colors};

const GIB_F: f64 = 1_073_741_824.0;

/// Bottom 2-column split: selected disk details + top process activity.
pub fn render_selection_split(
    ui: &mut egui::Ui,
    snap: &StorageSnapshot,
    raw_processes: &[RawProcessCpuRow],
    selected: &Option<Arc<str>>,
) {
    let vol = snap.volumes.iter().find(|v| selected.as_ref() == Some(&v.mount_point)).cloned();
    // All partitions sharing the selected physical disk (or just the volume).
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
    ui.columns(2, |cols| {
        render_details_card(&mut cols[0], snap, vol.as_ref(), &disk_volumes);
        render_activity_card(&mut cols[1], raw_processes, &letters, selected);
    });
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

fn render_details_card(
    ui: &mut egui::Ui,
    snap: &StorageSnapshot,
    vol: Option<&DiskVolumeInfo>,
    disk_volumes: &[DiskVolumeInfo],
) {
    card_frame().show(ui, |ui| {
        let is_disk = vol.is_some_and(|v| v.physical_disk.is_some());
        ui.label(
            egui::RichText::new(if is_disk {
                "SELECTED DISK DETAILS"
            } else {
                "SELECTED VOLUME DETAILS"
            })
            .monospace()
            .size(12.0)
            .color(colors::TEXT_PRIMARY),
        );
        ui.add_space(4.0);
        if let Some(v) = vol {
            if let Some(disk) = v.physical_disk {
                // Single per-disk numbers once, then one line per partition.
                line(ui, "Physical Drive", &format!("Disk {disk} ({})", v.kind.badge()));
                line(
                    ui,
                    "Live I/O",
                    &format!("R: {:.1} MB/s | W: {:.1} MB/s", v.read_mb_s, v.write_mb_s),
                );
                let mounts: Vec<String> =
                    disk_volumes.iter().map(|x| x.mount_point.to_string()).collect();
                line(ui, "Partitions", &mounts.join(", "));
                let worst = disk_volumes.iter().max_by(|a, b| a.used_pct.total_cmp(&b.used_pct));
                if let Some(w) = worst {
                    let (health, color) = health_label(w);
                    ui.label(egui::RichText::new(health).monospace().size(11.0).color(color));
                }
                for part in disk_volumes {
                    let used = part.total_bytes.saturating_sub(part.available_bytes) as f64 / GIB_F;
                    let free = part.available_bytes as f64 / GIB_F;
                    let total = part.total_bytes as f64 / GIB_F;
                    let key = format!("{} ({})", part.mount_point, part.fs_type);
                    let value = format!(
                        "{used:.1} / {total:.1} GiB ({:.0}%) Free {free:.1} GiB",
                        part.used_pct
                    );
                    line(ui, &key, &value);
                }
            } else {
                // No disk mapping: single-volume details.
                let used = v.total_bytes.saturating_sub(v.available_bytes) as f64 / GIB_F;
                let free = v.available_bytes as f64 / GIB_F;
                let total = v.total_bytes as f64 / GIB_F;
                let (health, color) = health_label(v);
                line(ui, "Mount", &format!("{}  {}", v.mount_point, v.name));
                line(ui, "Media", &format!("{} / {}", v.kind.badge(), v.fs_type));
                line(ui, "Used", &format!("{used:.2} GiB"));
                line(ui, "Free", &format!("{free:.2} GiB"));
                line(ui, "Total", &format!("{total:.2} GiB"));
                line(ui, "Used %", &format!("{:.1}%", v.used_pct));
                ui.label(egui::RichText::new(health).monospace().size(11.0).color(color));
            }
            let _ = snap;
        } else {
            ui.label(
                egui::RichText::new("No volumes detected")
                    .monospace()
                    .size(11.0)
                    .color(colors::TEXT_SECONDARY),
            );
        }
    });
}

fn render_activity_card(
    ui: &mut egui::Ui,
    raw: &[RawProcessCpuRow],
    letters: &[char],
    selected: &Option<Arc<str>>,
) {
    card_frame().show(ui, |ui| {
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
                    let on_label = format!("[ On {} ]", scope.join(","));
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
                            egui::RichText::new("[ All Drives ]")
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

fn render_consumers(ui: &mut egui::Ui, raw: &[RawProcessCpuRow], filter: Option<&[char]>) {
    let top = cached_consumers(raw, filter);
    let active = top.iter().any(|g| g.total >= 0.05);
    if !active {
        let ago = last_scan()
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .map(|t| t.elapsed().as_secs_f64())
            .unwrap_or(-1.0);
        let ago_txt = if ago < 0.0 { "just now".to_owned() } else { format!("{ago:.0}s ago") };
        let empty = format!("No active process disk I/O (>= 0.05 MB/s). Last scan: {ago_txt} (monitoring {} processes)", raw.len());
        ui.label(egui::RichText::new(empty).monospace().size(11.0).color(colors::TEXT_SECONDARY));
        return;
    }
    ui.columns(4, |cols| {
        cols[0].label(
            egui::RichText::new("PROCESS").monospace().size(10.0).color(colors::TEXT_SECONDARY),
        );
        cols[1].label(
            egui::RichText::new("READ MB/s").monospace().size(10.0).color(colors::TEXT_SECONDARY),
        );
        cols[2].label(
            egui::RichText::new("WRITE MB/s").monospace().size(10.0).color(colors::TEXT_SECONDARY),
        );
        cols[3]
            .label(egui::RichText::new("RAM").monospace().size(10.0).color(colors::TEXT_SECONDARY));
    });
    for g in top.iter() {
        let label = if g.count > 1 { format!("{} (x{})", g.name, g.count) } else { g.name.clone() };
        let badge = g.drive.map(|d| format!(" [{d}:]")).unwrap_or_default();
        let short = truncate(&format!("{label}{badge}"), 22);
        ui.columns(4, |cols| {
            cols[0].label(
                egui::RichText::new(short).monospace().size(11.0).color(colors::TEXT_PRIMARY),
            );
            cols[1].label(
                egui::RichText::new(format!("{:.1}", g.read))
                    .monospace()
                    .size(11.0)
                    .color(colors::ACCENT_CYAN),
            );
            cols[2].label(
                egui::RichText::new(format!("{:.1}", g.write))
                    .monospace()
                    .size(11.0)
                    .color(colors::ACCENT_AMBER),
            );
            cols[3].label(
                egui::RichText::new(format_bytes(g.ram_bytes))
                    .monospace()
                    .size(11.0)
                    .color(colors::TEXT_SECONDARY),
            );
        });
    }
}

fn line(ui: &mut egui::Ui, key: &str, value: &str) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(key).monospace().size(10.0).color(colors::TEXT_SECONDARY));
        ui.label(egui::RichText::new(value).monospace().size(11.0).color(colors::TEXT_PRIMARY));
    });
}

fn truncate(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_owned();
    }
    let mut out: String = text.chars().take(max_chars.saturating_sub(1)).collect();
    out.push('~');
    out
}
