//! Storage view helpers split from view for the 280-line guard.
use eframe::egui;

use crate::modules::storage::model::StorageSnapshot;
use crate::theme::colors;

pub(crate) fn render_blackbox(ui: &mut egui::Ui, snap: &StorageSnapshot) {
    ui.vertical(|ui| {
        ui.label(
            egui::RichText::new("I/O BLACKBOX").monospace().size(9.0).color(colors::TEXT_SECONDARY),
        );
        if let Some(spike) = snap.last_io_spike.as_ref() {
            let sub = format!(
                "{} {:.1} MB/s ({}s ago)",
                spike.process_name, spike.peak_mb_s, spike.ticks_ago
            );
            let tip = format!(
                "PID {} at {} - R {:.1} / W {:.1} MB/s - peak {:.1} MB/s",
                spike.pid, spike.timestamp_hms, spike.read_mb_s, spike.write_mb_s, spike.peak_mb_s
            );
            let resp = ui.label(
                egui::RichText::new("SPIKE ALERT").monospace().size(13.0).color(colors::ACCENT_RED),
            );
            resp.on_hover_text(tip);
            ui.label(
                egui::RichText::new(truncate(&sub, 28))
                    .monospace()
                    .size(9.0)
                    .color(colors::TEXT_SECONDARY),
            );
        } else {
            ui.label(
                egui::RichText::new("ARMED")
                    .monospace()
                    .size(13.0)
                    .color(crate::theme::OPTIMAL_EMERALD),
            );
        }
    });
}

pub(crate) fn col_kpi(ui: &mut egui::Ui, label: &str, value: String, color: egui::Color32) {
    ui.vertical(|ui| {
        ui.label(egui::RichText::new(label).monospace().size(9.0).color(colors::TEXT_SECONDARY));
        ui.label(egui::RichText::new(value).monospace().size(13.0).color(color));
    });
}

pub(crate) fn truncate(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_owned();
    }
    let mut out: String = text.chars().take(max_chars.saturating_sub(1)).collect();
    out.push('~');
    out
}
