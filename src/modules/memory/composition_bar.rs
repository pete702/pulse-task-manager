//! RAMMap-style 3-segment Memory Composition Bar for PULSE.
use crate::i18n::Key;
use crate::modules::memory::formatters::format_bytes;
use crate::modules::memory::model::MemorySnapshot;
use crate::theme::{card_frame, colors, load_color};
use eframe::egui;

const GIB: f64 = 1_073_741_824.0;
const BORDER_COLOR: egui::Color32 = egui::Color32::from_rgb(35, 48, 68);

fn segment_widths(
    total_px: f32,
    in_use: f64,
    cache: f64,
    free: f64,
    total: f64,
) -> (f32, f32, f32) {
    let px = total_px.max(0.0);
    let total = total.max(1.0);
    let in_use_frac = (in_use.max(0.0) / total).clamp(0.0, 1.0);
    let cache_frac = (cache.max(0.0) / total).clamp(0.0, 1.0 - in_use_frac);
    let free_frac = (free.max(0.0) / total).clamp(0.0, 1.0 - in_use_frac - cache_frac);
    let in_use_w = (px as f64 * in_use_frac).clamp(0.0, px as f64) as f32;
    let cache_w = (px as f64 * cache_frac).clamp(0.0, (px - in_use_w).max(0.0) as f64) as f32;
    let free_w =
        (px as f64 * free_frac).clamp(0.0, (px - in_use_w - cache_w).max(0.0) as f64) as f32;
    (in_use_w, cache_w, free_w)
}

fn legend(ui: &mut egui::Ui, color: egui::Color32, text: &str) {
    ui.label(egui::RichText::new("■").monospace().size(12.0).color(color));
    ui.label(egui::RichText::new(text).monospace().size(11.0).color(colors::TEXT_PRIMARY));
    ui.add_space(16.0);
}

pub fn render_composition_bar(ui: &mut egui::Ui, snap: &MemorySnapshot) {
    card_frame().show(ui, |ui| {
        ui.vertical(|ui| {
            ui.label(egui::RichText::new(Key::MemoryCompositionTitle.t()).monospace().size(13.0).color(colors::ACCENT_CYAN));
            ui.add_space(6.0);
            let total = snap.ram_total_bytes.max(1) as f64;
            let in_use = (snap.ram_used_bytes as f64).clamp(0.0, total);
            let avail = (snap.ram_available_bytes as f64).clamp(0.0, (total - in_use).max(0.0));
            let cache = (snap.ram_cached_bytes() as f64).clamp(0.0, avail);
            let free = (avail - cache).max(0.0);
            let pct = |v: f64| (v / total * 100.0) as f32;
            let (in_use_pct, cache_pct, free_pct) = (pct(in_use), pct(cache), pct(free));
            let bar_w = ui.available_width().max(80.0);
            let (rect, response) = ui.allocate_exact_size(egui::vec2(bar_w, 12.0), egui::Sense::hover());
            ui.painter().rect_filled(rect, egui::Rounding::same(3.0), egui::Color32::from_rgb(0x1E, 0x22, 0x30));
            let (in_use_w, cache_w, free_w) = segment_widths(rect.width(), in_use, cache, free, total);
            let (x0, x1) = (rect.left(), rect.left() + in_use_w);
            let x2 = (x1 + cache_w).min(rect.right());
            let x3 = (x2 + free_w).min(rect.right());
            if in_use_w > 0.0 { ui.painter().rect_filled(egui::Rect::from_min_max(egui::pos2(x0, rect.top()), egui::pos2(x1, rect.bottom())), egui::Rounding::same(3.0), load_color(in_use_pct)); }
            if cache_w > 0.0 { ui.painter().rect_filled(egui::Rect::from_min_max(egui::pos2(x1, rect.top()), egui::pos2(x2, rect.bottom())), egui::Rounding::same(3.0), egui::Color32::from_rgb(0x00, 0x88, 0xAA)); }
            if free_w > 0.0 { ui.painter().rect_filled(egui::Rect::from_min_max(egui::pos2(x2, rect.top()), egui::pos2(x3, rect.bottom())), egui::Rounding::same(3.0), egui::Color32::from_rgb(0x1E, 0x22, 0x30)); }
            ui.painter().rect_stroke(rect, egui::Rounding::same(3.0), egui::Stroke::new(1.0_f32, BORDER_COLOR));
            if response.hovered() {
                let tooltip = format!("In Use (Active): {:.2} GB ({:.1}%)\nAvailable: {:.2} GB ({:.1}%)\n  Windows File & System Cache (K32GetPerformanceInfo.SystemCache): {:.2} GB ({:.1}%)\n  Free: {:.2} GB ({:.1}%)\nCommitted / Limit: {:.2} / {:.2} GB ({:.1}%)", in_use / GIB, in_use_pct, avail / GIB, cache_pct + free_pct, cache / GIB, cache_pct, free / GIB, free_pct, snap.commit_total_bytes as f64 / GIB, snap.commit_limit_bytes as f64 / GIB, snap.commit_charge_pct());
                response.on_hover_text(tooltip);
            }
            ui.add_space(6.0);
            ui.horizontal_wrapped(|ui| {
                legend(ui, load_color(in_use_pct), &format!("In Use: {} ({:.0}%)", format_bytes(snap.ram_used_bytes), in_use_pct));
                legend(ui, egui::Color32::from_rgb(0x00, 0x88, 0xAA), &format!("Cached (OS): {} ({:.0}%)", format_bytes(snap.ram_cached_bytes()), cache_pct));
                legend(ui, colors::TEXT_SECONDARY, &format!("Free: {} ({:.0}%)", format_bytes(free as u64), free_pct));
            });
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_segment_widths_sum_to_bar_width() {
        for (in_use, cache, free) in
            [(50.0, 30.0, 20.0), (100.0, 0.0, 0.0), (0.0, 0.0, 100.0), (33.0, 33.0, 34.0)]
        {
            let (a, b, c) = segment_widths(100.0, in_use, cache, free, 100.0);
            assert!(a >= 0.0 && b >= 0.0 && c >= 0.0);
            assert!((a + b + c - 100.0).abs() < 1e-3);
        }
    }
}
