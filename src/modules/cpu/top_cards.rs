//! CPU telemetry cards: Total Processor Load + Peak Thread Bottleneck.
use crate::app_persist::MIN_WINDOW_WIDTH;
use crate::modules::cpu::model::CpuSnapshot;
use crate::ui::primitives::draw_tactical_ticks;
use eframe::egui;

/// Renders the two CPU telemetry cards side by side.
pub fn render_cpu_cards_row(ui: &mut egui::Ui, cpu: &CpuSnapshot) {
    let avail = ui.available_width();
    if avail < MIN_WINDOW_WIDTH - 40.0 {
        ui.vertical(|ui| {
            crate::modules::cpu::total_load_card::render_total_load_card(ui, cpu);
            ui.add_space(10.0);
            crate::modules::cpu::bottleneck_card::render_bottleneck_card(ui, cpu);
        });
    } else {
        ui.columns(2, |cols| {
            crate::modules::cpu::total_load_card::render_total_load_card(&mut cols[0], cpu);
            crate::modules::cpu::bottleneck_card::render_bottleneck_card(&mut cols[1], cpu);
        });
    }
}

/// Smooth continuous precision bar with threshold marker (85%).
pub fn draw_precision_bar_with_threshold(
    ui: &egui::Ui,
    rect: egui::Rect,
    ratio: f32,
    active_color: egui::Color32,
    threshold: f32,
) {
    let r = ratio.clamp(0.0, 1.0);

    // Determine threshold color (same logic as draw_segmented_led_bar)
    let threshold_color = if r >= 0.85 {
        egui::Color32::from_rgb(239, 68, 68) // red alert
    } else if r >= 0.70 {
        egui::Color32::from_rgb(245, 158, 11) // amber warning
    } else {
        active_color
    };

    // 1. Draw dark recessed track background
    let track_color = egui::Color32::from_rgba_unmultiplied(15, 23, 42, 220);
    let border_color = egui::Color32::from_rgba_unmultiplied(51, 65, 85, 140);
    ui.painter().rect_filled(rect, egui::Rounding::same(3.0), track_color);
    ui.painter().rect_stroke(
        rect,
        egui::Rounding::same(3.0),
        egui::Stroke::new(1.0_f32, border_color),
    );

    // 2. Draw 85% threshold marker line
    if threshold > 0.0 && threshold < 1.0 {
        let thresh_x = rect.min.x + rect.width() * threshold;
        let thresh_rect = egui::Rect::from_min_max(
            egui::pos2(thresh_x, rect.min.y),
            egui::pos2(thresh_x + 1.0, rect.max.y),
        );
        let thresh_color = egui::Color32::from_rgba_unmultiplied(239, 68, 68, 160);
        ui.painter().rect_filled(thresh_rect, egui::Rounding::ZERO, thresh_color);
    }

    // 3. Draw smooth continuous fill with leading-edge cursor cap
    if r > 0.002 {
        let fill_w = (rect.width() * r).max(2.0).min(rect.width());
        let fill_rect =
            egui::Rect::from_min_max(rect.min, egui::pos2(rect.min.x + fill_w, rect.max.y));
        let fill_color = egui::Color32::from_rgba_unmultiplied(
            threshold_color.r(),
            threshold_color.g(),
            threshold_color.b(),
            215,
        );
        ui.painter().rect_filled(fill_rect, egui::Rounding::same(3.0), fill_color);
        // 4. Crisp leading-edge cursor cap (2px bright white at right tip of fill)
        let cap_w = 2.0_f32.min(fill_w);
        let cap_rect = egui::Rect::from_min_max(
            egui::pos2(fill_rect.max.x - cap_w, fill_rect.min.y),
            fill_rect.max,
        );
        let cap_color = egui::Color32::from_rgb(240, 253, 250);
        ui.painter().rect_filled(cap_rect, egui::Rounding::ZERO, cap_color);
    }
    draw_tactical_ticks(ui, rect);
}
