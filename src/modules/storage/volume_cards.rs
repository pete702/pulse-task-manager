//! Physical disk cards grouped by Disk N for `[03] Storage`.

use std::sync::Arc;

use crate::modules::memory::formatters::format_bytes;
use crate::modules::storage::model::{group_physical_disks, DiskVolumeInfo, PhysicalDiskGroup};
use crate::theme::{card_frame, colors};
use eframe::egui;

/// Renders physical disk cards grouped by `physical_disk`.
/// Each card shows live per-disk R/W and lists its partitions.
pub fn render_volume_cards(
    ui: &mut egui::Ui,
    volumes: &[DiskVolumeInfo],
    selected_mount: &mut Option<Arc<str>>,
) {
    if volumes.is_empty() {
        card_frame().show(ui, |ui| {
            ui.label(
                egui::RichText::new("No volumes detected")
                    .monospace()
                    .size(12.0)
                    .color(colors::TEXT_SECONDARY),
            );
        });
        return;
    }
    let groups = group_physical_disks(volumes);
    if ui.available_width() >= 520.0 {
        for pair in groups.chunks(2) {
            ui.columns(2, |cols| {
                render_disk_card(&mut cols[0], &pair[0], selected_mount);
                if let Some(second) = pair.get(1) {
                    render_disk_card(&mut cols[1], second, selected_mount);
                }
            });
            ui.add_space(8.0);
        }
    } else {
        for group in &groups {
            render_disk_card(ui, group, selected_mount);
        }
    }
}

/// Honest 3-tier health: critical -> LOW (red), >=80% -> HIGH (amber),
/// else HEALTHY (emerald). Text + color both change.
/// The badge always carries the used percent so cards and details agree.
pub fn health_label(vol: &DiskVolumeInfo) -> (String, egui::Color32) {
    let pct = vol.used_pct.round() as u32;
    if vol.is_critical_space {
        (format!("[ LOW {pct}% ]"), colors::ACCENT_RED)
    } else if vol.used_pct >= 80.0 {
        (format!("[ HIGH {pct}% ]"), colors::ACCENT_AMBER)
    } else {
        (format!("[ HEALTHY {pct}% ]"), crate::theme::OPTIMAL_EMERALD)
    }
}

fn bar_color(vol: &DiskVolumeInfo) -> egui::Color32 {
    if vol.used_pct >= 90.0 || vol.is_critical_space {
        colors::ACCENT_RED
    } else if vol.used_pct >= 80.0 {
        colors::ACCENT_AMBER
    } else {
        colors::ACCENT_CYAN
    }
}

fn disk_title(group: &PhysicalDiskGroup) -> String {
    match group.disk_index {
        Some(d) => format!("DISK {d} - {}", group.kind.badge()),
        None => {
            let mount = group.volumes.first().map(|v| v.mount_point.as_ref()).unwrap_or("?");
            format!("{mount} - {}", group.kind.badge())
        }
    }
}

fn render_disk_card(
    ui: &mut egui::Ui,
    group: &PhysicalDiskGroup,
    selected_mount: &mut Option<Arc<str>>,
) {
    let is_selected = selected_mount
        .as_ref()
        .is_some_and(|sel| group.volumes.iter().any(|v| &v.mount_point == sel));
    let frame = if is_selected {
        card_frame().stroke(egui::Stroke::new(2.0_f32, colors::ACCENT_CYAN))
    } else {
        card_frame()
    };
    let mut clicked_mount: Option<Arc<str>> = None;
    let resp_inner = frame.show(ui, |ui| {
        render_disk_header(ui, group, is_selected);
        ui.add_space(4.0);
        for vol in &group.volumes {
            let clicked = render_partition(ui, vol, selected_mount);
            if clicked {
                clicked_mount = Some(Arc::clone(&vol.mount_point));
            }
        }
    });
    let id = match group.disk_index {
        Some(d) => egui::Id::new(format!("disk_card_{d}")),
        None => egui::Id::new(format!("disk_card_{}", group.volumes[0].mount_point)),
    };
    let resp = ui
        .interact(resp_inner.response.rect, id, egui::Sense::click())
        .on_hover_cursor(egui::CursorIcon::PointingHand);
    if let Some(mount) = clicked_mount {
        *selected_mount = Some(mount);
    } else if resp.clicked() {
        if let Some(first) = group.volumes.first() {
            *selected_mount = Some(Arc::clone(&first.mount_point));
        }
    }
}

fn render_disk_header(ui: &mut egui::Ui, group: &PhysicalDiskGroup, is_selected: bool) {
    let active = group.read_mb_s + group.write_mb_s > 0.05;
    let speed_color = if active { colors::ACCENT_CYAN } else { colors::TEXT_SECONDARY };
    // Worst partition drives the health badge so cards and details agree.
    let worst = group.volumes.iter().max_by(|a, b| a.used_pct.total_cmp(&b.used_pct));
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(disk_title(group))
                .monospace()
                .size(13.0)
                .color(colors::TEXT_PRIMARY),
        );
        ui.label(
            egui::RichText::new(format!(
                "R: {:.1} MB/s | W: {:.1} MB/s",
                group.read_mb_s, group.write_mb_s
            ))
            .monospace()
            .size(11.0)
            .color(speed_color),
        );
        if is_selected {
            ui.label(
                egui::RichText::new("[ SELECTED ]")
                    .monospace()
                    .size(11.0)
                    .color(colors::ACCENT_CYAN),
            );
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if let Some(v) = worst {
                let (health, color) = health_label(v);
                ui.label(egui::RichText::new(health).monospace().size(11.0).color(color));
            }
        });
    });
}

fn render_partition(
    ui: &mut egui::Ui,
    vol: &DiskVolumeInfo,
    selected_mount: &Option<Arc<str>>,
) -> bool {
    let is_selected = selected_mount.as_ref() == Some(&vol.mount_point);
    let used = vol.total_bytes.saturating_sub(vol.available_bytes);
    let mut clicked = false;
    ui.horizontal(|ui| {
        let label = if is_selected {
            format!("{} ({}) [ SELECTED ]", vol.mount_point, vol.fs_type)
        } else {
            format!("{} ({})", vol.mount_point, vol.fs_type)
        };
        let btn = egui::Button::new(
            egui::RichText::new(label).monospace().size(11.0).color(if is_selected {
                colors::ACCENT_CYAN
            } else {
                colors::TEXT_PRIMARY
            }),
        );
        if ui.add(btn).on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
            clicked = true;
        }
        ui.label(
            egui::RichText::new(format!(
                "{} / {} ({:.0}%)",
                format_bytes(used),
                format_bytes(vol.total_bytes),
                vol.used_pct
            ))
            .monospace()
            .size(10.0)
            .color(colors::TEXT_PRIMARY),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                egui::RichText::new(format!("Free {}", format_bytes(vol.available_bytes)))
                    .monospace()
                    .size(10.0)
                    .color(colors::TEXT_SECONDARY),
            );
        });
    });
    render_bar(ui, vol);
    clicked
}

fn render_bar(ui: &mut egui::Ui, vol: &DiskVolumeInfo) {
    let pct = vol.used_pct.clamp(0.0, 100.0) / 100.0;
    let bar_w = ui.available_width().max(100.0);
    let (rect, _) =
        ui.allocate_exact_size(egui::vec2(bar_w, 8.0), egui::Sense::focusable_noninteractive());
    ui.painter().rect_filled(rect, 2.0, colors::BORDER);
    if pct > 0.0 {
        let fill = rect.width() * pct;
        let filled =
            egui::Rect::from_min_max(rect.min, egui::Pos2::new(rect.min.x + fill, rect.max.y));
        ui.painter().rect_filled(filled, 2.0, bar_color(vol));
    }
    ui.add_space(6.0);
}
