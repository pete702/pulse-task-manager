//! Clamped hover tooltip and legend toggles for storage chart.
use crate::modules::storage::chart_view::next_visibility;
use crate::theme::colors;
use eframe::egui;
use std::sync::Arc;
fn san(v: f64) -> f64 {
    if v.is_finite() {
        v.max(0.0)
    } else {
        0.0
    }
}
fn nearest_y(s: &Arc<Vec<[f64; 2]>>, target: f64) -> Option<f64> {
    s.iter().min_by(|a, b| (a[0] - target).abs().total_cmp(&(b[0] - target).abs())).map(|p| p[1])
}
fn when_str(ago: f64) -> String {
    if ago.abs() < 0.05 {
        "now".to_owned()
    } else {
        format!("{ago:.1}m ago")
    }
}
fn push_tip(
    lines: &mut Vec<(String, egui::Color32)>,
    name: &str,
    s: &Arc<Vec<[f64; 2]>>,
    show: bool,
    target: f64,
    ago: f64,
    col: egui::Color32,
) {
    if !show {
        return;
    }
    if let Some(y) = nearest_y(s, target) {
        lines.push((format!("{name}: {:.1} MB/s ({})", san(y), when_str(ago)), col));
    }
}
/// Draws a wide tooltip box clamped inside the plot rect.
#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_hover_tip(
    ui: &egui::Ui,
    rect: egui::Rect,
    span: f64,
    read: &Arc<Vec<[f64; 2]>>,
    write: &Arc<Vec<[f64; 2]>>,
    show_read: bool,
    show_write: bool,
    hover: Option<egui::Pos2>,
) {
    let Some(pos) = hover else { return };
    if !rect.contains(pos) {
        return;
    }
    let frac = ((pos.x - rect.left()) / rect.width().max(1.0)).clamp(0.0, 1.0) as f64;
    let ago = span * (1.0 - frac);
    let target = -ago;
    let mut lines: Vec<(String, egui::Color32)> = Vec::new();
    push_tip(&mut lines, "Read", read, show_read, target, ago, colors::ACCENT_CYAN);
    push_tip(&mut lines, "Write", write, show_write, target, ago, colors::ACCENT_AMBER);
    if lines.is_empty() {
        return;
    }
    let tw = 240.0_f32;
    let th = lines.len() as f32 * 16.0 + 10.0;
    let x =
        (pos.x + 14.0).clamp(rect.left() + 4.0, (rect.right() - tw - 4.0).max(rect.left() + 4.0));
    let y =
        (pos.y + 14.0).clamp(rect.top() + 4.0, (rect.bottom() - th - 4.0).max(rect.top() + 4.0));
    let tip_rect = egui::Rect::from_min_size(egui::pos2(x, y), egui::vec2(tw, th));
    let p = ui.painter();
    p.rect_filled(
        tip_rect,
        egui::Rounding::same(4.0),
        egui::Color32::from_rgba_unmultiplied(11, 12, 16, 235),
    );
    p.rect_stroke(tip_rect, egui::Rounding::same(4.0), egui::Stroke::new(1.0_f32, colors::BORDER));
    for (i, (text, col)) in lines.iter().enumerate() {
        p.text(
            egui::pos2(x + 8.0, y + 5.0 + i as f32 * 16.0),
            egui::Align2::LEFT_TOP,
            text,
            egui::FontId::monospace(10.5),
            *col,
        );
    }
}
/// Interactive Read/Write legend toggles on the plot top edge.
pub(crate) fn draw_toggles(ui: &mut egui::Ui, rect: egui::Rect, sr: bool, sw: bool) {
    let y = rect.top() + 3.0;
    let items = [
        ("Read MB/s", colors::ACCENT_CYAN, sr, true, 64.0, "storage_chart_show_read"),
        ("Write MB/s", colors::ACCENT_AMBER, sw, false, 162.0, "storage_chart_show_write"),
    ];
    let p = ui.painter();
    for (label, base, active, is_read, dx, id) in items {
        let r = egui::Rect::from_min_size(egui::pos2(rect.left() + dx, y), egui::vec2(92.0, 15.0));
        let resp = ui.interact(r, egui::Id::new(id), egui::Sense::click());
        if resp.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
        if resp.clicked() {
            let (nr, nw) = next_visibility(sr, sw, is_read);
            if is_read {
                crate::modules::storage::view::set_show_read(nr);
            } else {
                crate::modules::storage::view::set_show_write(nw);
            }
        }
        let ca = if active { 195 } else { 60 };
        let ta = if active { 175 } else { 60 };
        let cc = egui::Color32::from_rgba_unmultiplied(base.r(), base.g(), base.b(), ca);
        let tc = egui::Color32::from_rgba_unmultiplied(148, 163, 184, ta);
        p.circle_filled(egui::pos2(r.min.x + 6.0, r.min.y + 7.0), 3.5, cc);
        p.text(
            egui::pos2(r.min.x + 16.0, r.min.y + 7.0),
            egui::Align2::LEFT_CENTER,
            label,
            egui::FontId::monospace(10.5),
            tc,
        );
    }
}
