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
fn when_str(ago: f64) -> String {
    if ago.abs() < 0.05 {
        "now".to_owned()
    } else {
        format!("{ago:.1}m ago")
    }
}
/// Draws vertical scanner, curve dots, and tooltip clamped inside the plot rect.
#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_hover_tip(
    ui: &egui::Ui,
    rect: egui::Rect,
    span: f64,
    ceil: f64,
    read: &Arc<Vec<[f64; 2]>>,
    write: &Arc<Vec<[f64; 2]>>,
    show_read: bool,
    show_write: bool,
    hover: Option<egui::Pos2>,
) {
    let Some(pos) = hover else { return };
    if !rect.contains(pos) || span <= 0.0 {
        return;
    }
    let frac = ((pos.x - rect.left()) / rect.width().max(1.0)).clamp(0.0, 1.0) as f64;
    let ago = span * (1.0 - frac);
    let target = -ago;
    let p = ui.painter();
    p.vline(
        pos.x,
        rect.y_range(),
        egui::Stroke::new(1.0_f32, egui::Color32::from_rgba_unmultiplied(0, 212, 255, 140)),
    );
    let mut lines: Vec<(String, egui::Color32)> = Vec::new();
    for (show, s, name, col) in [
        (show_read, read, "Read", colors::ACCENT_CYAN),
        (show_write, write, "Write", colors::ACCENT_AMBER),
    ] {
        if !show {
            continue;
        }
        if let Some(pt) =
            s.iter().copied().min_by(|a, b| (a[0] - target).abs().total_cmp(&(b[0] - target).abs()))
        {
            let yv = san(pt[1]);
            let y = if ceil > 0.0 {
                rect.bottom() - ((yv / ceil).clamp(0.0, 1.0) as f32) * rect.height()
            } else {
                rect.bottom()
            };
            let dot = egui::pos2(pos.x, y);
            p.circle_filled(
                dot,
                5.0,
                egui::Color32::from_rgba_unmultiplied(col.r(), col.g(), col.b(), 70),
            );
            p.circle_filled(dot, 3.5, col);
            p.circle_stroke(
                dot,
                3.5,
                egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(240, 253, 250)),
            );
            lines.push((format!("{name}: {yv:.1} MB/s ({})", when_str(ago)), col));
        }
    }
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
