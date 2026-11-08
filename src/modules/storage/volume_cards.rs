//! Physical disk cards grouped by Disk N for `[03] Storage`.
use crate::modules::memory::formatters::format_bytes;
use crate::modules::storage::model::{group_physical_disks, DiskVolumeInfo, PhysicalDiskGroup};
use crate::theme::{card_frame, colors};
use eframe::egui;
use std::sync::Arc;
const LC: egui::Align2 = egui::Align2::LEFT_CENTER;
const RC: egui::Align2 = egui::Align2::RIGHT_CENTER;
fn pt(
    p: &egui::Painter,
    pos: egui::Pos2,
    a: egui::Align2,
    s: impl std::fmt::Display,
    sz: f32,
    c: egui::Color32,
) {
    p.text(pos, a, s, egui::FontId::monospace(sz), c);
}
/// Honest 3-tier health: critical -- LOW FREE, >=80% -- HIGH USE, else OK.
#[allow(dead_code)]
pub fn health_label(vol: &DiskVolumeInfo) -> (String, egui::Color32) {
    let pct = vol.used_pct;
    if vol.is_critical_space {
        (format!("LOW FREE ({pct:.0}% USED)"), crate::theme::TACTICAL_RED)
    } else if pct >= 80.0 {
        (format!("HIGH USE ({pct:.0}%)"), crate::theme::WARNING_AMBER)
    } else {
        (format!("OK ({pct:.0}% USED)"), crate::theme::EMERALD_GREEN)
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
fn total_gib(volumes: &[DiskVolumeInfo]) -> u64 {
    volumes.iter().map(|v| v.total_bytes).sum::<u64>() / (1024 * 1024 * 1024)
}
fn disk_title(group: &PhysicalDiskGroup) -> String {
    let gib = total_gib(&group.volumes);
    match group.disk_index {
        Some(d) => {
            let badge = group.kind.badge();
            if badge == "DISK" {
                format!("DISK {d} ({gib}G)")
            } else {
                format!("DISK {d} {badge} ({gib}G)")
            }
        }
        None => {
            let m = group.volumes.first().map(|v| v.mount_point.as_ref()).unwrap_or("?");
            format!("DISK [{}] ({gib}G)", m.trim_end_matches('\\'))
        }
    }
}
/// Full-width clickable ALL selector showing global throughput.
pub fn render_all_selector(
    ui: &mut egui::Ui,
    read: f32,
    write: f32,
    selected_mount: &mut Option<Arc<str>>,
) {
    let active = selected_mount.is_none();
    let frame = if active {
        card_frame()
            .stroke(egui::Stroke::new(2.0_f32, colors::ACCENT_CYAN))
            .inner_margin(egui::Margin::symmetric(8.0, 4.0))
    } else {
        card_frame().inner_margin(egui::Margin::symmetric(8.0, 4.0))
    };
    let speed_col = if read + write > 0.05 { colors::ACCENT_CYAN } else { colors::TEXT_SECONDARY };
    frame.show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        ui.spacing_mut().item_spacing.y = 2.0;
        let w = ui.available_width().max(120.0);
        let (rect, resp) = ui.allocate_exact_size(egui::vec2(w, 16.0), egui::Sense::click());
        let p = ui.painter();
        let spd = format!("R: {read:.1} | W: {write:.1} MB/s");
        pt(p, rect.left_center(), LC, "ALL STORAGE DISKS", 11.0, colors::TEXT_PRIMARY);
        pt(p, rect.right_center(), RC, spd, 10.0, speed_col);
        if resp.clicked() {
            *selected_mount = None;
        }
        if resp.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
    });
}
/// Renders physical disks as a single vertical master column.
pub fn render_volume_cards(
    ui: &mut egui::Ui,
    volumes: &[DiskVolumeInfo],
    selected_mount: &mut Option<Arc<str>>,
) {
    if volumes.is_empty() {
        card_frame().show(ui, |ui| {
            ui.set_min_width(ui.available_width());
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
    if ui.available_width() >= 440.0 {
        for pair in groups.chunks(2) {
            ui.columns(2, |cols| {
                render_disk_card(&mut cols[0], &pair[0], selected_mount);
                if let Some(second) = pair.get(1) {
                    render_disk_card(&mut cols[1], second, selected_mount);
                }
            });
            ui.add_space(4.0);
        }
    } else {
        for group in &groups {
            render_disk_card(ui, group, selected_mount);
            ui.add_space(4.0);
        }
    }
}
fn render_disk_card(
    ui: &mut egui::Ui,
    group: &PhysicalDiskGroup,
    selected_mount: &mut Option<Arc<str>>,
) {
    let is_selected =
        selected_mount.as_ref().is_some_and(|s| group.volumes.iter().any(|v| &v.mount_point == s));
    let frame = if is_selected {
        card_frame()
            .stroke(egui::Stroke::new(2.0_f32, colors::ACCENT_CYAN))
            .inner_margin(egui::Margin::symmetric(8.0, 4.0))
    } else {
        card_frame().inner_margin(egui::Margin::symmetric(8.0, 4.0))
    };
    let active = group.read_mb_s + group.write_mb_s > 0.05;
    let speed_col = if active { colors::ACCENT_CYAN } else { colors::TEXT_SECONDARY };
    let title = disk_title(group);
    let first_mount = group.volumes.first().map(|v| Arc::clone(&v.mount_point));
    frame.show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        ui.spacing_mut().item_spacing.y = 2.0;
        let w = ui.available_width().max(120.0);
        let (rect, resp) = ui.allocate_exact_size(egui::vec2(w, 16.0), egui::Sense::click());
        let p = ui.painter();
        let spd = format!("R:{:.1} W:{:.1} MB/s", group.read_mb_s, group.write_mb_s);
        pt(p, rect.left_center(), LC, title.clone(), 11.0, colors::TEXT_PRIMARY);
        pt(p, rect.right_center(), RC, spd, 10.5, speed_col);
        if resp.clicked() {
            if let Some(first) = first_mount.as_ref() {
                if selected_mount.as_ref() == Some(first) {
                    *selected_mount = None;
                } else {
                    *selected_mount = Some(Arc::clone(first));
                }
            }
        }
        if resp.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
        for vol in &group.volumes {
            render_partition(ui, vol, selected_mount);
            ui.add_space(2.0);
        }
    });
}
fn render_partition(
    ui: &mut egui::Ui,
    vol: &DiskVolumeInfo,
    selected_mount: &mut Option<Arc<str>>,
) {
    let is_sel = selected_mount.as_ref() == Some(&vol.mount_point);
    let free_gib = vol.available_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
    let used_gib =
        vol.total_bytes.saturating_sub(vol.available_bytes) as f64 / (1024.0 * 1024.0 * 1024.0);
    let total_gib = vol.total_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
    let narrow = ui.available_width() < 220.0;
    let pct_s = format!("{:.0}%", vol.used_pct);
    let cap_s = if narrow {
        format!("{used_gib:.1}/{total_gib:.1}G - ")
    } else {
        format!("{used_gib:.1}/{total_gib:.1}G (Free {free_gib:.1}G) ")
    };
    let bc = bar_color(vol);
    let left = format!("{} {}", vol.mount_point.trim_end_matches('\\'), vol.fs_type);
    let col = if is_sel { colors::ACCENT_CYAN } else { colors::TEXT_PRIMARY };
    let used = vol.total_bytes.saturating_sub(vol.available_bytes);
    let tip = format!(
        "Used {} / Total {} - Free {} - {}",
        format_bytes(used),
        format_bytes(vol.total_bytes),
        format_bytes(vol.available_bytes),
        vol.fs_type
    );
    let w = ui.available_width().max(120.0);
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(w, 15.0), egui::Sense::click());
    let p = ui.painter();
    pt(p, rect.left_center(), LC, left, 11.0, col);
    pt(p, rect.right_center(), RC, pct_s.clone(), 10.0, bc);
    let cap_x = rect.right() - (pct_s.len() as f32) * 6.5 - 2.0;
    pt(p, egui::pos2(cap_x, rect.center().y), RC, cap_s, 10.0, colors::TEXT_SECONDARY);
    let clicked = resp.clicked();
    let hovered = resp.hovered();
    if hovered {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        resp.on_hover_text(tip);
    }
    if clicked {
        if is_sel {
            *selected_mount = None;
        } else {
            *selected_mount = Some(Arc::clone(&vol.mount_point));
        }
    }
    render_bar(ui, vol);
}
fn render_bar(ui: &mut egui::Ui, vol: &DiskVolumeInfo) {
    let pct = vol.used_pct.clamp(0.0, 100.0) / 100.0;
    let bar_w = ui.available_width().max(60.0);
    let (rect, _) =
        ui.allocate_exact_size(egui::vec2(bar_w, 3.0), egui::Sense::focusable_noninteractive());
    ui.painter().rect_filled(rect, 1.5, colors::BORDER);
    if pct > 0.0 {
        let fill = rect.width() * pct;
        let filled =
            egui::Rect::from_min_max(rect.min, egui::Pos2::new(rect.min.x + fill, rect.max.y));
        ui.painter().rect_filled(filled, 2.0, bar_color(vol));
    }
}
