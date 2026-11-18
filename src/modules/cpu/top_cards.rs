//! CPU telemetry cards: Total Processor Load + Peak Thread Bottleneck.
use crate::app_persist::MIN_WINDOW_WIDTH;
use crate::modules::cpu::model::CpuSnapshot;
use crate::ui::primitives::draw_tactical_ticks;
use eframe::egui;

const TOP_CARDS_SIDEBAR_OFFSET: f32 = 40.0;

/// Renders the two CPU telemetry cards side by side.
pub fn render_cpu_cards_row(ui: &mut egui::Ui, cpu: &CpuSnapshot) {
    let avail = ui.available_width();
    if avail < MIN_WINDOW_WIDTH - TOP_CARDS_SIDEBAR_OFFSET {
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

/// Pure helper for precision bar threshold color.
pub(crate) fn precision_bar_color(pct: f32) -> egui::Color32 {
    if pct >= 85.0 {
        crate::theme::colors::ACCENT_RED
    } else if pct >= 70.0 {
        crate::theme::colors::ACCENT_AMBER
    } else {
        crate::theme::colors::ACCENT_CYAN
    }
}

/// Smooth continuous precision bar with threshold marker (85%).
pub fn draw_precision_bar_with_threshold(
    ui: &egui::Ui,
    rect: egui::Rect,
    ratio: f32,
    _active_color: egui::Color32,
    threshold: f32,
) {
    let r = ratio.clamp(0.0, 1.0);

    // Determine threshold color via pure helper.
    let threshold_color = precision_bar_color(r * 100.0);

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

#[cfg(test)]
mod tests {
    use super::precision_bar_color;
    use crate::theme::colors;

    #[test]
    fn precision_bar_color_thresholds() {
        assert_eq!(precision_bar_color(50.0), colors::ACCENT_CYAN);
        assert_eq!(precision_bar_color(69.9), colors::ACCENT_CYAN);
        assert_eq!(precision_bar_color(70.0), colors::ACCENT_AMBER);
        assert_eq!(precision_bar_color(80.0), colors::ACCENT_AMBER);
        assert_eq!(precision_bar_color(84.9), colors::ACCENT_AMBER);
        assert_eq!(precision_bar_color(85.0), colors::ACCENT_RED);
        assert_eq!(precision_bar_color(100.0), colors::ACCENT_RED);
    }
}
