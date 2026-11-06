//! CPU chart HUD drawing: axis labels and interactive legend.

use crate::modules::cpu::history::CpuChartWindow;
use crate::modules::memory::chart_axis::{format_tick_label, x_axis_ticks};
use crate::theme::colors;
use eframe::egui;

pub fn draw_y_axis_hud_labels(ui: &egui::Ui, plot_rect: egui::Rect) {
    let painter = ui.painter();
    let hud_label = egui::Color32::from_rgba_unmultiplied(148, 163, 184, 135);
    painter.text(
        egui::pos2(plot_rect.right() - 6.0, plot_rect.top() + 3.0),
        egui::Align2::RIGHT_TOP,
        "100%",
        egui::FontId::monospace(10.0),
        hud_label,
    );
    for (pct, label) in [(75.0, "75%"), (50.0, "50%"), (25.0, "25%")] {
        let y = plot_rect.bottom() - (pct / 100.0) * plot_rect.height();
        painter.text(
            egui::pos2(plot_rect.right() - 6.0, y - 2.0),
            egui::Align2::RIGHT_CENTER,
            label,
            egui::FontId::monospace(10.0),
            hud_label,
        );
    }
}

pub fn draw_x_axis_hud_labels(ui: &egui::Ui, plot_rect: egui::Rect, window: CpuChartWindow) {
    let painter = ui.painter();
    let hud_label = egui::Color32::from_rgba_unmultiplied(148, 163, 184, 165);
    let ticks = x_axis_ticks(window);
    // Draw labels in dedicated strip below plot so cyan line never collides.
    let y = plot_rect.bottom() + 8.0;
    let count = ticks.len();
    for (idx, tick) in ticks.iter().enumerate() {
        let x = if count > 1 {
            let fraction = idx as f32 / (count - 1) as f32;
            plot_rect.left() + fraction * plot_rect.width()
        } else {
            plot_rect.left()
        };
        let label = format_tick_label(window, *tick);
        let align = if idx == 0 {
            egui::Align2::LEFT_TOP
        } else if idx + 1 == count {
            egui::Align2::RIGHT_TOP
        } else {
            egui::Align2::CENTER_TOP
        };
        let x_adjusted = if idx == 0 {
            plot_rect.left() + 8.0
        } else if idx + 1 == count {
            plot_rect.right() - 8.0
        } else {
            x
        };
        painter.text(
            egui::pos2(x_adjusted, y),
            align,
            label,
            egui::FontId::monospace(11.0),
            hud_label,
        );
    }
}

pub fn draw_hud_legend(ui: &mut egui::Ui, plot_rect: egui::Rect) -> (bool, bool) {
    let painter = ui.painter();
    let radius = 3.5;
    let y_leg = plot_rect.top() + 3.0;

    let total_id = egui::Id::new("cpu_chart_show_total");
    let peak_id = egui::Id::new("cpu_chart_show_peak");

    let mut show_total: bool =
        ui.ctx().data_mut(|d| *d.get_temp_mut_or_insert_with(total_id, || true));
    let mut show_peak: bool =
        ui.ctx().data_mut(|d| *d.get_temp_mut_or_insert_with(peak_id, || true));

    // Item 1: Total CPU %
    let total_rect = egui::Rect::from_min_size(
        egui::pos2(plot_rect.left() + 8.0, y_leg),
        egui::vec2(72.0, 15.0),
    );
    let total_resp = ui.interact(total_rect, total_id, egui::Sense::click());
    if total_resp.clicked() {
        show_total = !show_total;
        ui.ctx().data_mut(|d| d.insert_temp(total_id, show_total));
    }
    if total_resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }

    let total_active_alpha = if show_total { 195 } else { 60 };
    let total_text_alpha = if show_total { 175 } else { 60 };
    let total_circle_color = egui::Color32::from_rgba_unmultiplied(
        colors::ACCENT_CYAN.r(),
        colors::ACCENT_CYAN.g(),
        colors::ACCENT_CYAN.b(),
        total_active_alpha,
    );
    let total_text_color = egui::Color32::from_rgba_unmultiplied(148, 163, 184, total_text_alpha);

    let circle_x = total_rect.left() + 6.0;
    painter.circle_filled(egui::pos2(circle_x, y_leg + 7.0), radius, total_circle_color);
    painter.text(
        egui::pos2(circle_x + 10.0, y_leg + 7.0),
        egui::Align2::LEFT_CENTER,
        "Total CPU %",
        egui::FontId::monospace(10.5),
        total_text_color,
    );

    // Item 2: Peak Thread % (+30.0 px breathing room from "Total CPU %")
    let peak_rect = egui::Rect::from_min_size(
        egui::pos2(plot_rect.left() + 116.0, y_leg),
        egui::vec2(82.0, 15.0),
    );
    let peak_resp = ui.interact(peak_rect, peak_id, egui::Sense::click());
    if peak_resp.clicked() {
        show_peak = !show_peak;
        ui.ctx().data_mut(|d| d.insert_temp(peak_id, show_peak));
    }
    if peak_resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }

    let peak_active_alpha = if show_peak { 195 } else { 60 };
    let peak_text_alpha = if show_peak { 175 } else { 60 };
    let peak_circle_color = egui::Color32::from_rgba_unmultiplied(
        colors::ACCENT_AMBER.r(),
        colors::ACCENT_AMBER.g(),
        colors::ACCENT_AMBER.b(),
        peak_active_alpha,
    );
    let peak_text_color = egui::Color32::from_rgba_unmultiplied(148, 163, 184, peak_text_alpha);

    let circle_x2 = peak_rect.left() + 6.0;
    painter.circle_filled(egui::pos2(circle_x2, y_leg + 7.0), radius, peak_circle_color);
    painter.text(
        egui::pos2(circle_x2 + 10.0, y_leg + 7.0),
        egui::Align2::LEFT_CENTER,
        "Peak Thread %",
        egui::FontId::monospace(10.5),
        peak_text_color,
    );

    (show_total, show_peak)
}
