//! In-canvas HUD drawing for the memory chart: axis labels and interactive legend.

use crate::modules::memory::chart_axis::{format_tick_label, x_axis_ticks};
use crate::modules::memory::model::ChartWindow;
use crate::theme::colors;
use eframe::egui;

fn draw_label_bg(
    p: &egui::Painter,
    pos: egui::Pos2,
    align: egui::Align2,
    text: &str,
    font: egui::FontId,
    color: egui::Color32,
) {
    let galley = p.layout_no_wrap(text.to_owned(), font, color);
    let rect = align.anchor_rect(egui::Rect::from_min_size(pos, galley.size()));
    let bg = rect.expand2(egui::vec2(3.0, 1.0));
    p.rect_filled(
        bg,
        egui::Rounding::same(2.0),
        egui::Color32::from_rgba_unmultiplied(11, 15, 22, 220),
    );
    p.galley(rect.min, galley, color);
}

/// Draws Y-axis percentage labels (100%, 75%, 50%, 25%) inside the right edge of plot_rect.
pub fn draw_y_axis_hud_labels(ui: &egui::Ui, plot_rect: egui::Rect) {
    let p = ui.painter();
    let c = egui::Color32::from_rgba_unmultiplied(148, 163, 184, 135);
    let f = egui::FontId::monospace(10.0);
    draw_label_bg(
        p,
        egui::pos2(plot_rect.right() - 6.0, plot_rect.top() + 3.0),
        egui::Align2::RIGHT_TOP,
        "100%",
        f.clone(),
        c,
    );
    for (pct, label) in [(75.0, "75%"), (50.0, "50%"), (25.0, "25%")] {
        let y = plot_rect.bottom() - (pct / 100.0) * plot_rect.height();
        draw_label_bg(
            p,
            egui::pos2(plot_rect.right() - 6.0, y - 2.0),
            egui::Align2::RIGHT_CENTER,
            label,
            f.clone(),
            c,
        );
    }
}

/// Draws X-axis time labels inside the bottom edge of plot_rect.
pub fn draw_x_axis_hud_labels(ui: &egui::Ui, plot_rect: egui::Rect, window: ChartWindow) {
    let p = ui.painter();
    let c = egui::Color32::from_rgba_unmultiplied(148, 163, 184, 165);
    let f = egui::FontId::monospace(11.0);
    let ticks = x_axis_ticks(window);
    let y = plot_rect.bottom() - 5.0;
    let count = ticks.len();
    for (idx, tick) in ticks.iter().enumerate() {
        let x = if count > 1 {
            plot_rect.left() + (idx as f32 / (count - 1) as f32) * plot_rect.width()
        } else {
            plot_rect.left()
        };
        let label = format_tick_label(window, *tick);
        let (align, x_adj) = if idx == 0 {
            (egui::Align2::LEFT_BOTTOM, plot_rect.left() + 8.0)
        } else if idx + 1 == count {
            (egui::Align2::RIGHT_BOTTOM, plot_rect.right() - 8.0)
        } else {
            (egui::Align2::CENTER_BOTTOM, x)
        };
        draw_label_bg(p, egui::pos2(x_adj, y), align, &label, f.clone(), c);
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

fn fmt_rel_time(x_min: f64) -> String {
    let secs = (-x_min * 60.0).max(0.0);
    if secs < 0.5 {
        "now".to_owned()
    } else if secs < 60.0 {
        format!("{secs:.0}s ago")
    } else {
        format!("{:.1}m ago", secs / 60.0)
    }
}

fn nearest_sample(points: &[[f64; 2]], x: f64) -> Option<[f64; 2]> {
    points.iter().copied().min_by(|a, b| (a[0] - x).abs().total_cmp(&(b[0] - x).abs()))
}

#[allow(clippy::too_many_arguments)]
pub fn draw_hud_scanner(
    ui: &egui::Ui,
    plot_rect: egui::Rect,
    span_mins: f64,
    ram_pts: &[[f64; 2]],
    commit_pts: &[[f64; 2]],
    show_ram: bool,
    show_commit: bool,
    has_commit: bool,
) {
    let Some(pointer) = ui.ctx().pointer_hover_pos() else { return };
    if !plot_rect.contains(pointer) || span_mins <= 0.0 {
        return;
    }
    let y_leg = plot_rect.top() + 3.0;
    let ram_rect = egui::Rect::from_min_size(
        egui::pos2(plot_rect.left() + 8.0, y_leg),
        egui::vec2(62.0, 15.0),
    );
    let commit_rect = egui::Rect::from_min_size(
        egui::pos2(plot_rect.left() + 76.0, y_leg),
        egui::vec2(78.0, 15.0),
    );
    if ram_rect.contains(pointer) || commit_rect.contains(pointer) {
        return;
    }
    let frac = ((pointer.x - plot_rect.left()) / plot_rect.width().max(1.0)).clamp(0.0, 1.0) as f64;
    let x_min = -span_mins + frac * span_mins;
    let p = ui.painter();
    p.vline(
        pointer.x,
        plot_rect.y_range(),
        egui::Stroke::new(1.0_f32, egui::Color32::from_rgba_unmultiplied(0, 212, 255, 140)),
    );
    let mut tip: Vec<(String, egui::Color32)> = vec![(fmt_rel_time(x_min), colors::TEXT_PRIMARY)];
    let commit_label = if has_commit { "Commit" } else { "Swap" };
    for (enabled, pts, color, label) in [
        (show_ram, ram_pts, colors::ACCENT_CYAN, "RAM"),
        (show_commit, commit_pts, colors::ACCENT_PURPLE, commit_label),
    ] {
        if !enabled {
            continue;
        }
        if let Some(pt) = nearest_sample(pts, x_min) {
            let v = if pt[1].is_finite() { pt[1].clamp(0.0, 100.0) } else { 0.0 };
            let y = plot_rect.bottom() - (v as f32 / 100.0) * plot_rect.height();
            let pos = egui::pos2(pointer.x, y);
            p.circle_filled(
                pos,
                5.0,
                egui::Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 70),
            );
            p.circle_filled(pos, 3.5, color);
            p.circle_stroke(
                pos,
                3.5,
                egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(240, 253, 250)),
            );
            tip.push((format!("{label}: {v:.1}%"), color));
        }
    }
    if tip.len() <= 1 {
        return;
    }
    let tw = 138.0_f32;
    let th = tip.len() as f32 * 15.0 + 8.0;
    let x = (pointer.x + 14.0)
        .clamp(plot_rect.left() + 4.0, (plot_rect.right() - tw - 4.0).max(plot_rect.left() + 4.0));
    let y = (pointer.y + 14.0)
        .clamp(plot_rect.top() + 4.0, (plot_rect.bottom() - th - 4.0).max(plot_rect.top() + 4.0));
    let tr = egui::Rect::from_min_size(egui::pos2(x, y), egui::vec2(tw, th));
    p.rect_filled(
        tr,
        egui::Rounding::same(4.0),
        egui::Color32::from_rgba_unmultiplied(11, 12, 16, 235),
    );
    p.rect_stroke(tr, egui::Rounding::same(4.0), egui::Stroke::new(1.0_f32, colors::BORDER));
    for (i, (text, col)) in tip.iter().enumerate() {
        p.text(
            egui::pos2(x + 7.0, y + 4.0 + i as f32 * 15.0),
            egui::Align2::LEFT_TOP,
            text,
            egui::FontId::monospace(10.0),
            *col,
        );
    }
}
