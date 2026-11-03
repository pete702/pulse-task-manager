//! RAMMap-style 3-segment Memory Composition Bar for PULSE.

use crate::i18n::Key;
use crate::modules::memory::formatters::format_bytes;
use crate::modules::memory::model::MemorySnapshot;
use crate::theme::{card_frame, colors, load_color};
use eframe::egui;

const GIB: f64 = 1_073_741_824.0;

/// Clamps the three segment widths so they always sum to exactly `total_px`.
fn segment_widths(total_px: f32, in_use: f64, standby: f64, total: f64) -> (f32, f32, f32) {
    let (px, total) = (total_px.max(0.0), total.max(1.0));
    let in_use_w = (px as f64 * (in_use.max(0.0) / total)).clamp(0.0, px as f64) as f32;
    let standby_w =
        (px as f64 * (standby.max(0.0) / total)).clamp(0.0, (px - in_use_w).max(0.0) as f64) as f32;
    (in_use_w, standby_w, (px - in_use_w - standby_w).max(0.0))
}

fn legend(ui: &mut egui::Ui, color: egui::Color32, text: &str) {
    ui.label(egui::RichText::new("■").monospace().size(12.0).color(color));
    ui.label(egui::RichText::new(text).monospace().size(11.0).color(colors::TEXT_PRIMARY));
    ui.add_space(16.0);
}

/// Renders the RAMMap-style 3-segment memory composition bar.
pub fn render_composition_bar(ui: &mut egui::Ui, snap: &MemorySnapshot) {
    let frame = card_frame().inner_margin(egui::Margin::symmetric(14.0, 10.0));
    let outer = egui::Margin { left: 0.0, right: 4.0, top: 0.0, bottom: 0.0 };
    frame.outer_margin(outer).show(ui, |ui| {
        let inner_width = (ui.available_width() - 16.0).max(80.0);
        ui.set_max_width(inner_width);
        ui.vertical(|ui| {
            let title = egui::RichText::new(Key::MemoryCompositionTitle.t()).monospace().size(13.0).color(colors::ACCENT_CYAN);
            ui.label(title);
            ui.add_space(6.0);
            let total = snap.ram_total_bytes.max(1) as f64;
            let in_use = (snap.ram_used_bytes as f64).clamp(0.0, total);
            let avail = (snap.ram_available_bytes as f64).clamp(0.0, (total - in_use).max(0.0));
            let standby = (snap.ram_cached_bytes() as f64).clamp(0.0, avail);
            let true_free = (avail - standby).max(0.0);
            let pct = |v: f64| (v / total * 100.0) as f32;
            let (in_use_pct, standby_pct, free_pct) = (pct(in_use), pct(standby), pct(true_free));
            let (rect, response) = ui.allocate_exact_size(egui::vec2(inner_width, 12.0), egui::Sense::hover());
            ui.painter().rect_filled(rect, egui::Rounding::same(3.0), egui::Color32::from_rgb(0x1E, 0x22, 0x30));
            let (in_use_w, standby_w, _free_w) = segment_widths(rect.width(), in_use, standby, total);
            let (x0, x1) = (rect.left(), rect.left() + in_use_w);
            let x2 = (x1 + standby_w).min(rect.right());
            if in_use_w > 0.0 {
                let r = egui::Rect::from_min_max(egui::pos2(x0, rect.top()), egui::pos2(x1, rect.bottom()));
                ui.painter().rect_filled(r, egui::Rounding::same(3.0), load_color(in_use_pct));
            }
            if standby_w > 0.0 {
                let r = egui::Rect::from_min_max(egui::pos2(x1, rect.top()), egui::pos2(x2, rect.bottom()));
                ui.painter().rect_filled(r, egui::Rounding::same(3.0), egui::Color32::from_rgb(0x00, 0x88, 0xAA));
            }
            ui.painter().rect_stroke(rect, egui::Rounding::same(3.0), egui::Stroke::new(1.0_f32, colors::BORDER));
            if response.hovered() {
                let tooltip = format!("In Use (Active): {:.2} GB ({:.1}%)\nStandby Cache (Reclaimable): {:.2} GB ({:.1}%)\nFree Memory: {:.2} GB ({:.1}%)\nCommitted / Limit: {:.2} / {:.2} GB ({:.1}%)",
                    in_use / GIB, in_use_pct, standby / GIB, standby_pct, true_free / GIB, free_pct,
                    snap.commit_total_bytes as f64 / GIB, snap.commit_limit_bytes as f64 / GIB, snap.commit_charge_pct());
                response.on_hover_text(tooltip);
            }
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                legend(ui, load_color(in_use_pct), &format!("In Use: {} ({:.0}%)", format_bytes(snap.ram_used_bytes), in_use_pct));
                legend(ui, egui::Color32::from_rgb(0x00, 0x88, 0xAA), &format!("Standby: {} ({:.0}%)", format_bytes(snap.ram_cached_bytes()), standby_pct));
                legend(ui, egui::Color32::from_rgb(0x1E, 0x22, 0x30), &format!("Free: {} ({:.0}%)", format_bytes(true_free as u64), free_pct));
            });
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_segment_widths_sum_to_bar_width() {
        for (in_use, standby) in [(50.0, 30.0), (100.0, 0.0), (0.0, 0.0), (33.0, 33.0)] {
            let (a, b, c) = segment_widths(100.0, in_use, standby, 100.0);
            assert!(a >= 0.0 && b >= 0.0 && c >= 0.0);
            assert!((a + b + c - 100.0).abs() < 1e-3);
        }
    }
}
