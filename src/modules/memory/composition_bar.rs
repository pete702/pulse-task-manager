//! RAMMap-style 3-segment Memory Composition Bar for PULSE.
//! Shows In Use (Active), Standby Cache (Reclaimable), and Free memory as proportional segments.

use crate::i18n::Key;
use crate::modules::memory::formatters::format_bytes;
use crate::modules::memory::model::MemorySnapshot;
use crate::theme::{card_frame, colors, load_color};
use eframe::egui;

/// Renders the RAMMap-style 3-segment memory composition bar.
/// Segments: In Use (Active), Standby Cache (Reclaimable), Free Memory.
pub fn render_composition_bar(ui: &mut egui::Ui, snap: &MemorySnapshot) {
    card_frame()
        .inner_margin(egui::Margin::symmetric(14.0, 10.0))
        .outer_margin(egui::Margin { left: 0.0, right: 4.0, top: 0.0, bottom: 0.0 })
        .show(ui, |ui| {
            let inner_width = (ui.available_width() - 16.0).max(80.0);
            ui.set_max_width(inner_width);

            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(Key::MemoryCompositionTitle.t()).monospace().size(13.0).color(colors::ACCENT_CYAN));
                });
                ui.add_space(6.0);

                let total = snap.ram_total_bytes.max(1) as f64;
                let in_use = (snap.ram_used_bytes as f64).min(total);
                let avail = (snap.ram_available_bytes as f64).min((total - in_use).max(0.0));
                let standby = (snap.ram_cached_bytes() as f64).min(avail);
                let true_free = (avail - standby).max(0.0);

                let in_use_pct = (in_use / total * 100.0) as f32;
                let standby_pct = (standby / total * 100.0) as f32;
                let free_pct = (true_free / total * 100.0) as f32;

                let (rect, response) = ui.allocate_exact_size(egui::vec2(inner_width, 12.0), egui::Sense::hover());

                ui.painter().rect_filled(rect, egui::Rounding::same(3.0), egui::Color32::from_rgb(0x1E, 0x22, 0x30));

                let rect_w = rect.width();
                let in_use_w = (rect_w * (in_use / total) as f32).clamp(0.0, rect_w);
                let remaining_after_in_use = (rect_w - in_use_w).max(0.0);
                let standby_w = (rect_w * (standby / total) as f32).clamp(0.0, remaining_after_in_use);

                if in_use_w > 0.0 {
                    let in_use_color = load_color(in_use_pct);
                    let in_use_rect = egui::Rect::from_min_max(egui::pos2(rect.left(), rect.top()), egui::pos2(rect.left() + in_use_w, rect.bottom()));
                    ui.painter().rect_filled(in_use_rect, egui::Rounding::same(3.0), in_use_color);
                }
                if standby_w > 0.0 {
                    let standby_rect = egui::Rect::from_min_max(egui::pos2(rect.left() + in_use_w, rect.top()), egui::pos2(rect.left() + in_use_w + standby_w, rect.bottom()));
                    ui.painter().rect_filled(standby_rect, egui::Rounding::same(3.0), egui::Color32::from_rgb(0x00, 0x88, 0xAA));
                }

                ui.painter().rect_stroke(rect, egui::Rounding::same(3.0), egui::Stroke::new(1.0_f32, colors::BORDER));

                if response.hovered() {
                    let tooltip_text = format!("In Use (Active): {:.2} GiB ({:.1}%)\nStandby Cache (Reclaimable): {:.2} GiB ({:.1}%)\nFree Memory: {:.2} GiB ({:.1}%)",
                        in_use / 1_073_741_824.0, in_use_pct, standby / 1_073_741_824.0, standby_pct, true_free / 1_073_741_824.0, free_pct);
                    response.on_hover_text(tooltip_text);
                }

                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("■").monospace().size(12.0).color(load_color(in_use_pct)));
                    ui.label(egui::RichText::new(format!("In Use: {} ({:.0}%)", format_bytes(snap.ram_used_bytes), in_use_pct)).monospace().size(11.0).color(colors::TEXT_PRIMARY));
                    ui.add_space(16.0);
                    ui.label(egui::RichText::new("■").monospace().size(12.0).color(egui::Color32::from_rgb(0x00, 0x88, 0xAA)));
                    ui.label(egui::RichText::new(format!("Standby: {} ({:.0}%)", format_bytes(snap.ram_cached_bytes()), standby_pct)).monospace().size(11.0).color(colors::TEXT_PRIMARY));
                    ui.add_space(16.0);
                    ui.label(egui::RichText::new("■").monospace().size(12.0).color(egui::Color32::from_rgb(0x1E, 0x22, 0x30)));
                    ui.label(egui::RichText::new(format!("Free: {} ({:.0}%)", format_bytes(true_free as u64), free_pct)).monospace().size(11.0).color(colors::TEXT_PRIMARY));
                });
            });
        });
}
