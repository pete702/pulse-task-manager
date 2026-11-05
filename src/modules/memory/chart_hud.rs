//! In-canvas HUD drawing for the memory chart: axis labels and legend.

use crate::modules::memory::chart_axis::{format_tick_label, x_axis_ticks};
use crate::modules::memory::model::ChartWindow;
use crate::theme::colors;
use eframe::egui;

/// Draws Y-axis percentage labels (100%, 75%, 50%, 25%) inside the right edge of plot_rect.
pub fn draw_y_axis_hud_labels(ui: &egui::Ui, plot_rect: egui::Rect) {
    let painter = ui.painter();
    let hud_label = egui::Color32::from_rgba_unmultiplied(148, 163, 184, 135);
    // 100% at top
    painter.text(
        egui::pos2(plot_rect.right() - 6.0, plot_rect.top() + 3.0),
        egui::Align2::RIGHT_TOP,
        "100%",
        egui::FontId::monospace(10.0),
        hud_label,
    );
    // 75%, 50%, 25% down the right edge
    for (pct, label) in [(75.0, "75%"), (50.0, "50%"), (25.0, "25%")] {
        let y = plot_rect.bottom() - (pct / 100.0) * plot_rect.height();
        painter.text(
            egui::pos2(plot_rect.right() - 6.0, y - 2.0),
            egui::Align2::RIGHT_BOTTOM,
            label,
            egui::FontId::monospace(10.0),
            hud_label,
        );
    }
}

/// Draws X-axis time labels inside the bottom edge of plot_rect.
/// Leftmost at plot_rect.left() + 8.0, rightmost at plot_rect.right() - 8.0.
pub fn draw_x_axis_hud_labels(ui: &egui::Ui, plot_rect: egui::Rect, window: ChartWindow) {
    let painter = ui.painter();
    let hud_label = egui::Color32::from_rgba_unmultiplied(148, 163, 184, 165);
    let ticks = x_axis_ticks(window);
    let y = plot_rect.bottom() - 5.0;
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
            egui::Align2::LEFT_BOTTOM
        } else if idx + 1 == count {
            egui::Align2::RIGHT_BOTTOM
        } else {
            egui::Align2::CENTER_BOTTOM
        };
        // Apply left/right padding for first/last labels
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

/// Draws translucent in-canvas HUD legend at top-left of plot_rect.
pub fn draw_hud_legend(ui: &egui::Ui, plot_rect: egui::Rect, has_commit: bool) {
    let painter = ui.painter();
    let hud_label = egui::Color32::from_rgba_unmultiplied(148, 163, 184, 165);
    let font = egui::FontId::monospace(10.5);
    let radius = 3.5;

    // Row 1: RAM % (cyan)
    let y1 = plot_rect.top() + 12.0;
    let x_circle = plot_rect.left() + 14.0;
    painter.circle_filled(
        egui::pos2(x_circle, y1),
        radius,
        egui::Color32::from_rgba_unmultiplied(
            colors::ACCENT_CYAN.r(),
            colors::ACCENT_CYAN.g(),
            colors::ACCENT_CYAN.b(),
            190,
        ),
    );
    painter.text(
        egui::pos2(x_circle + 10.0, y1),
        egui::Align2::LEFT_CENTER,
        "RAM %",
        font,
        hud_label,
    );

    // Row 2: Commit % or Swap % (purple)
    let y2 = plot_rect.top() + 26.0;
    let label2 = if has_commit { "Commit %" } else { "Swap %" };
    painter.circle_filled(
        egui::pos2(x_circle, y2),
        radius,
        egui::Color32::from_rgba_unmultiplied(
            colors::ACCENT_PURPLE.r(),
            colors::ACCENT_PURPLE.g(),
            colors::ACCENT_PURPLE.b(),
            190,
        ),
    );
    painter.text(
        egui::pos2(x_circle + 10.0, y2),
        egui::Align2::LEFT_CENTER,
        label2,
        egui::FontId::monospace(10.5),
        hud_label,
    );
}
