//! In-canvas HUD drawing for the memory chart: axis labels and interactive legend.

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

/// Draws horizontal interactive HUD legend at top of plot_rect.
/// Returns (show_ram, show_commit) toggle states.
pub fn draw_hud_legend(ui: &mut egui::Ui, plot_rect: egui::Rect, has_commit: bool) -> (bool, bool) {
    let painter = ui.painter();
    let radius = 3.5;
    let y_leg = plot_rect.top() + 3.0;

    // Persistent toggle states stored in egui context data
    let ram_id = egui::Id::new("mem_chart_show_ram");
    let commit_id = egui::Id::new("mem_chart_show_commit");

    let mut show_ram: bool = ui.ctx().data_mut(|d| *d.get_temp_mut_or_insert_with(ram_id, || true));
    let mut show_commit: bool =
        ui.ctx().data_mut(|d| *d.get_temp_mut_or_insert_with(commit_id, || true));

    // --- Item 1: RAM % ---
    let ram_rect = egui::Rect::from_min_size(
        egui::pos2(plot_rect.left() + 8.0, y_leg),
        egui::vec2(62.0, 15.0),
    );
    let ram_resp = ui.interact(ram_rect, ram_id, egui::Sense::click());
    if ram_resp.clicked() {
        show_ram = !show_ram;
        ui.ctx().data_mut(|d| d.insert_temp(ram_id, show_ram));
    }
    if ram_resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }

    let ram_active_alpha = if show_ram { 195 } else { 60 };
    let ram_text_alpha = if show_ram { 175 } else { 60 };
    let ram_circle_color = egui::Color32::from_rgba_unmultiplied(
        colors::ACCENT_CYAN.r(),
        colors::ACCENT_CYAN.g(),
        colors::ACCENT_CYAN.b(),
        ram_active_alpha,
    );
    let ram_text_color = egui::Color32::from_rgba_unmultiplied(148, 163, 184, ram_text_alpha);

    let circle_x = ram_rect.left() + 6.0;
    painter.circle_filled(egui::pos2(circle_x, y_leg + 7.0), radius, ram_circle_color);
    painter.text(
        egui::pos2(circle_x + 10.0, y_leg + 7.0),
        egui::Align2::LEFT_CENTER,
        "RAM %",
        egui::FontId::monospace(10.5),
        ram_text_color,
    );

    // --- Item 2: Commit % or Swap % ---
    let label2 = if has_commit { "Commit %" } else { "Swap %" };
    let commit_rect = egui::Rect::from_min_size(
        egui::pos2(plot_rect.left() + 76.0, y_leg),
        egui::vec2(78.0, 15.0),
    );
    let commit_resp = ui.interact(commit_rect, commit_id, egui::Sense::click());
    if commit_resp.clicked() {
        show_commit = !show_commit;
        ui.ctx().data_mut(|d| d.insert_temp(commit_id, show_commit));
    }
    if commit_resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }

    let commit_active_alpha = if show_commit { 195 } else { 60 };
    let commit_text_alpha = if show_commit { 175 } else { 60 };
    let commit_circle_color = egui::Color32::from_rgba_unmultiplied(
        colors::ACCENT_PURPLE.r(),
        colors::ACCENT_PURPLE.g(),
        colors::ACCENT_PURPLE.b(),
        commit_active_alpha,
    );
    let commit_text_color = egui::Color32::from_rgba_unmultiplied(148, 163, 184, commit_text_alpha);

    let circle_x2 = commit_rect.left() + 6.0;
    painter.circle_filled(egui::pos2(circle_x2, y_leg + 7.0), radius, commit_circle_color);
    painter.text(
        egui::pos2(circle_x2 + 10.0, y_leg + 7.0),
        egui::Align2::LEFT_CENTER,
        label2,
        egui::FontId::monospace(10.5),
        commit_text_color,
    );

    (show_ram, show_commit)
}
