//! Compact 2-column volume cards for `[03] Storage`.

use std::sync::Arc;

use crate::modules::memory::formatters::format_bytes;
use crate::modules::storage::model::DiskVolumeInfo;
use crate::theme::{card_frame, colors};
use eframe::egui;

/// Renders volume cards: 2-column row-major grid on wide layouts, else
/// 1 column. Row-major keeps `C:\` top-left, `D:\` top-right.
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
    if ui.available_width() >= 520.0 {
        for pair in volumes.chunks(2) {
            ui.columns(2, |cols| {
                render_volume_card(&mut cols[0], &pair[0], selected_mount);
                if let Some(second) = pair.get(1) {
                    render_volume_card(&mut cols[1], second, selected_mount);
                }
            });
            ui.add_space(8.0);
        }
    } else {
        for vol in volumes {
            render_volume_card(ui, vol, selected_mount);
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

fn render_volume_card(
    ui: &mut egui::Ui,
    vol: &DiskVolumeInfo,
    selected_mount: &mut Option<Arc<str>>,
) {
    let is_selected = selected_mount.as_ref() == Some(&vol.mount_point);
    let frame = if is_selected {
        card_frame().stroke(egui::Stroke::new(2.0_f32, colors::ACCENT_CYAN))
    } else {
        card_frame()
    };
    let resp_inner = frame.show(ui, |ui| {
        render_header(ui, vol, is_selected);
        ui.add_space(4.0);
        render_bar(ui, vol);
    });
    let id = egui::Id::new(format!("vol_card_{}", vol.mount_point));
    let resp = ui
        .interact(resp_inner.response.rect, id, egui::Sense::click())
        .on_hover_cursor(egui::CursorIcon::PointingHand);
    if resp.clicked() {
        *selected_mount = Some(Arc::clone(&vol.mount_point));
    }
}

fn render_header(ui: &mut egui::Ui, vol: &DiskVolumeInfo, is_selected: bool) {
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(vol.mount_point.as_ref())
                .monospace()
                .size(13.0)
                .color(colors::TEXT_PRIMARY),
        );
        if let Some(disk) = vol.physical_disk {
            ui.label(
                egui::RichText::new(format!("[ Disk {disk} ]"))
                    .monospace()
                    .size(11.0)
                    .color(colors::ACCENT_CYAN),
            );
        }
        ui.label(
            egui::RichText::new(format!("[ {} ]", vol.kind.badge()))
                .monospace()
                .size(11.0)
                .color(colors::ACCENT_CYAN),
        );
        ui.label(
            egui::RichText::new(vol.fs_type.as_ref().to_owned())
                .monospace()
                .size(11.0)
                .color(colors::TEXT_SECONDARY),
        );
        ui.label(
            egui::RichText::new(format!("{:.0}%", vol.used_pct))
                .monospace()
                .size(11.0)
                .color(bar_color(vol)),
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
            let (health, color) = health_label(vol);
            ui.label(egui::RichText::new(health).monospace().size(11.0).color(color));
        });
    });
}

fn render_bar(ui: &mut egui::Ui, vol: &DiskVolumeInfo) {
    let used = vol.total_bytes.saturating_sub(vol.available_bytes);
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
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(format!(
                "{} / {}",
                format_bytes(used),
                format_bytes(vol.total_bytes)
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
}
