use crate::modules::memory::process_item::ProcessMemoryItem;
use crate::theme::colors;
use eframe::egui;

pub(super) fn render_name_cell(ui: &mut egui::Ui, item: &ProcessMemoryItem) {
    ui.label(
        egui::RichText::new(&*item.name)
            .monospace()
            .size(12.0)
            .strong()
            .color(colors::TEXT_PRIMARY),
    );
    if item.process_count > 1 {
        ui.label(
            egui::RichText::new(format!("[{}x]", item.process_count))
                .monospace()
                .size(10.0)
                .color(colors::ACCENT_CYAN)
                .background_color(colors::ACCENT_CYAN.gamma_multiply(0.12)),
        );
    } else if item.is_hog {
        egui::Frame::none()
            .fill(egui::Color32::from_rgb(0x3A, 0x10, 0x1D))
            .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(0xE0, 0x2A, 0x54)))
            .rounding(egui::Rounding::same(3.0))
            .inner_margin(egui::Margin::symmetric(5.0, 1.0))
            .show(ui, |ui| {
                ui.label(
                    egui::RichText::new("▲ HOG")
                        .monospace()
                        .size(9.0)
                        .color(egui::Color32::from_rgb(0xE0, 0x2A, 0x54)),
                );
            });
    }
}
