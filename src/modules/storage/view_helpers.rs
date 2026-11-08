//! Storage view helpers split from view for the 280-line guard.
use crate::modules::storage::model::{DiskVolumeInfo, StorageSnapshot};
use crate::theme::{card_frame, colors};
use eframe::egui;
fn rt(s: impl Into<String>, sz: f32, c: egui::Color32) -> egui::RichText {
    egui::RichText::new(s).monospace().size(sz).color(c)
}
#[allow(dead_code)]
pub(crate) fn render_blackbox(ui: &mut egui::Ui, snap: &StorageSnapshot) {
    ui.vertical(|ui| {
        ui.label(rt("BLACKBOX", 10.0, colors::TEXT_SECONDARY));
        if let Some(spike) = snap.last_io_spike.as_ref() {
            let sub = format!(
                "{} {:.1} MB/s ({}s ago)",
                spike.process_name, spike.peak_mb_s, spike.ticks_ago
            );
            let tip = format!(
                "PID {} at {} - R {:.1} / W {:.1} MB/s - peak {:.1} MB/s",
                spike.pid, spike.timestamp_hms, spike.read_mb_s, spike.write_mb_s, spike.peak_mb_s
            );
            let resp = ui.label(rt("SPIKE ALERT", 13.0, colors::ACCENT_RED));
            resp.on_hover_text(tip);
            ui.label(rt(truncate(&sub, 28), 9.0, colors::TEXT_SECONDARY));
        } else {
            ui.label(rt("ARMED", 13.0, crate::theme::OPTIMAL_EMERALD));
        }
    });
}
#[allow(dead_code)]
pub(crate) fn col_kpi(ui: &mut egui::Ui, label: &str, value: String, color: egui::Color32) {
    ui.vertical(|ui| {
        ui.label(rt(label, 10.0, colors::TEXT_SECONDARY));
        ui.label(rt(value, 13.0, color));
    });
}
pub(super) fn truncate(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_owned();
    }
    let mut out: String = text.chars().take(max_chars.saturating_sub(1)).collect();
    out.push('~');
    out
}
/// Scope label like `Disk 0` or `Disk [L:]` (max 14 chars).
pub(crate) fn scope_label_for(
    snap: &StorageSnapshot,
    disk: Option<u32>,
    key: impl Fn(&DiskVolumeInfo) -> Option<u32>,
) -> Option<String> {
    disk.map(|d| {
        if d >= 0x8000_0000 {
            let ch = ((d & 0xFF) as u8) as char;
            return format!("Disk [{ch}:]");
        }
        let m: Vec<String> = snap
            .volumes
            .iter()
            .filter(|v| key(v) == Some(d))
            .map(|v| v.mount_point.to_string())
            .collect();
        let full = if m.len() == 1 { format!("Disk {d} ({})", m[0]) } else { format!("Disk {d}") };
        truncate(&full, 14)
    })
}
/// Fixed 4-column KPI strip with top padding.
#[allow(dead_code)]
pub(crate) fn render_status_and_kpis(
    ui: &mut egui::Ui,
    snap: &StorageSnapshot,
    kpi_read: f32,
    kpi_write: f32,
    _scope_disk: Option<u32>,
) {
    let (status_text, status_color, tip) = if let Some(spike) = snap.last_io_spike.as_ref() {
        let tip = format!(
            "PID {} at {} - R {:.1} / W {:.1} MB/s - peak {:.1} MB/s",
            spike.pid, spike.timestamp_hms, spike.read_mb_s, spike.write_mb_s, spike.peak_mb_s
        );
        ("SPIKE ALERT", colors::ACCENT_RED, Some(tip))
    } else {
        ("ARMED", crate::theme::OPTIMAL_EMERALD, None)
    };
    let read_s = format!("{kpi_read:.1}");
    let write_s = format!("{kpi_write:.1}");
    let top = snap
        .top_io_process
        .as_ref()
        .map(|t| truncate(&format!("{} {:.1}", t.1, t.2), 28))
        .unwrap_or_else(|| "-".to_owned());
    card_frame().inner_margin(egui::Margin::symmetric(10.0, 5.0)).show(ui, |ui| {
        ui.set_width(ui.available_width());
        let full_w = ui.available_width().max(280.0);
        let (rect, _) = ui.allocate_exact_size(egui::vec2(full_w, 34.0), egui::Sense::hover());
        let c_w = (full_w * 0.19).clamp(72.0, 105.0);
        let x0 = rect.left();
        let x1 = x0 + c_w;
        let x2 = x1 + c_w;
        let x3 = x2 + c_w;
        let top_y = rect.top() + 2.0;
        let val_y = rect.top() + 17.0;
        let hd = egui::FontId::proportional(10.0);
        let vd = egui::FontId::monospace(13.0);
        let hc = crate::theme::colors::TEXT_SECONDARY;
        let p = ui.painter();
        p.text(egui::pos2(x0, top_y), egui::Align2::LEFT_TOP, "I/O BLACKBOX", hd.clone(), hc);
        p.text(egui::pos2(x1, top_y), egui::Align2::LEFT_TOP, "READ MB/s", hd.clone(), hc);
        p.text(egui::pos2(x2, top_y), egui::Align2::LEFT_TOP, "WRITE MB/s", hd.clone(), hc);
        p.text(egui::pos2(x3, top_y), egui::Align2::LEFT_TOP, "TOP I/O", hd, hc);
        p.text(
            egui::pos2(x0, val_y),
            egui::Align2::LEFT_TOP,
            status_text,
            vd.clone(),
            status_color,
        );
        p.text(
            egui::pos2(x1, val_y),
            egui::Align2::LEFT_TOP,
            read_s,
            vd.clone(),
            colors::ACCENT_CYAN,
        );
        p.text(
            egui::pos2(x2, val_y),
            egui::Align2::LEFT_TOP,
            write_s,
            vd.clone(),
            colors::ACCENT_AMBER,
        );
        p.text(egui::pos2(x3, val_y), egui::Align2::LEFT_TOP, top, vd, colors::TEXT_PRIMARY);
        if let Some(t) = tip {
            let col_rect =
                egui::Rect::from_min_size(egui::pos2(x0, rect.top()), egui::vec2(c_w, 34.0));
            ui.interact(col_rect, egui::Id::new("kpi_blackbox_tip"), egui::Sense::hover())
                .on_hover_text(t);
        }
    });
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::storage::model::{DiskMediaKind, DiskVolumeInfo};
    use std::sync::Arc;
    #[test]
    fn synthetic_disk_decodes_letter() {
        let vol = DiskVolumeInfo::new(
            Arc::from("L:\\"),
            Arc::from("L"),
            Arc::from("NTFS"),
            DiskMediaKind::Unknown,
            100,
            50,
        );
        let snap = StorageSnapshot {
            volumes: Arc::from(vec![vol]),
            read_mb_s: 0.0,
            write_mb_s: 0.0,
            top_io_process: None,
            last_io_spike: None,
        };
        let key = 0x8000_0000 | (b'L' as u32);
        let label = scope_label_for(&snap, Some(key), |_| Some(key)).unwrap();
        assert_eq!(label, "Disk [L:]");
        assert!(!label.contains("214748"));
    }
}
