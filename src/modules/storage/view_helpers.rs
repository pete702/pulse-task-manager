//! Storage view helpers split from view for the 280-line guard.
use crate::modules::storage::model::{DiskVolumeInfo, StorageSnapshot};
use crate::theme::{card_frame, colors};
use eframe::egui;

#[derive(Clone, Default)]
struct KpiCache {
    read_bits: u32,
    write_bits: u32,
    top_key: u64,
    read_s: String,
    write_s: String,
    top: String,
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
            return format!("VOLUME [{ch}:]");
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
    let cache_id = egui::Id::new("storage_kpi_cache");
    let (read_s, write_s, top) = ui.ctx().data_mut(|d| {
        let c = d.get_temp_mut_or_insert_with(cache_id, KpiCache::default);
        let read_bits = kpi_read.to_bits();
        let write_bits = kpi_write.to_bits();
        let top_key = snap
            .top_io_process
            .as_ref()
            .map(|t| (t.0 as u64) ^ (t.2.to_bits() as u64))
            .unwrap_or(0);
        if c.read_bits != read_bits || c.write_bits != write_bits || c.top_key != top_key {
            c.read_bits = read_bits;
            c.write_bits = write_bits;
            c.top_key = top_key;
            c.read_s = format!("{kpi_read:.1}");
            c.write_s = format!("{kpi_write:.1}");
            c.top = snap
                .top_io_process
                .as_ref()
                .map(|t| truncate(&format!("{} {:.1} MB/s", t.1, t.2), 28))
                .unwrap_or_else(|| "-".to_owned());
        }
        (c.read_s.clone(), c.write_s.clone(), c.top.clone())
    });
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
        assert_eq!(label, "VOLUME [L:]");
        assert!(!label.contains("214748"));
    }
}
