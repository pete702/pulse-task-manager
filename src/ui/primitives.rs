//! Cockpit graphic primitives for PULSE telemetry HUD.
use eframe::egui;
pub fn draw_cell_heat_bar(
    ui: &egui::Ui,
    rect: egui::Rect,
    ratio: f32,
    _c: egui::Color32,
    alert: bool,
) {
    if !rect.width().is_finite()
        || !rect.height().is_finite()
        || !(rect.width() > 4.0 && rect.height() > 4.0)
    {
        return;
    }
    let r = if ratio.is_finite() { ratio.clamp(0.0, 1.0) } else { 0.0 };
    if r <= 0.002 {
        return;
    }
    let inner = rect.shrink2(egui::vec2(2.0, 2.0));
    if !(inner.width() > 0.0 && inner.height() > 0.0) {
        return;
    }
    let w = (inner.width() * r).clamp(0.0, inner.width());
    let fc = if alert {
        egui::Color32::from_rgba_unmultiplied(185, 28, 28, 72)
    } else {
        egui::Color32::from_rgba_unmultiplied(8, 145, 178, 62)
    };
    let fr = egui::Rect::from_min_max(inner.min, egui::pos2(inner.min.x + w, inner.max.y));
    ui.painter().rect_filled(fr, egui::Rounding::same(2.0), fc);
    if w > 2.5 {
        let cr = egui::Rect::from_min_max(egui::pos2(fr.max.x - 2.0, fr.min.y), fr.max);
        let cc = if alert {
            egui::Color32::from_rgb(254, 205, 211)
        } else {
            egui::Color32::from_rgb(240, 253, 250)
        };
        ui.painter().rect_filled(cr, egui::Rounding::ZERO, cc);
    }
    if r >= 0.08 || alert {
        let sr = egui::Rect::from_min_max(inner.min, egui::pos2(inner.min.x + 2.0, inner.max.y));
        let sc = if alert {
            egui::Color32::from_rgba_unmultiplied(239, 68, 68, 185)
        } else {
            egui::Color32::from_rgba_unmultiplied(8, 145, 178, 185)
        };
        ui.painter().rect_filled(sr, egui::Rounding::same(2.0), sc);
    }
}
pub fn draw_segmented_led_bar(
    ui: &egui::Ui,
    rect: egui::Rect,
    ratio: f32,
    _s: usize,
    ac: egui::Color32,
) {
    if !rect.width().is_finite()
        || !rect.height().is_finite()
        || !(rect.width() > 4.0 && rect.height() > 4.0)
    {
        return;
    }
    let r = if ratio.is_finite() { ratio.clamp(0.0, 1.0) } else { 0.0 };
    {
        let inner = rect.shrink2(egui::vec2(0.0, 0.0));
        if !(inner.width() > 0.0 && inner.height() > 0.0) {
            return;
        }
    }
    let tc = if r >= 0.85 {
        egui::Color32::from_rgb(239, 68, 68)
    } else if r >= 0.70 {
        egui::Color32::from_rgb(245, 158, 11)
    } else {
        ac
    };
    ui.painter().rect_filled(
        rect,
        egui::Rounding::same(3.0),
        egui::Color32::from_rgba_unmultiplied(15, 23, 42, 220),
    );
    ui.painter().rect_stroke(
        rect,
        egui::Rounding::same(3.0),
        egui::Stroke::new(1.0_f32, egui::Color32::from_rgba_unmultiplied(51, 65, 85, 140)),
    );
    if r > 0.002 {
        let w = (rect.width() * r).max(2.0).min(rect.width());
        let fr = egui::Rect::from_min_max(rect.min, egui::pos2(rect.min.x + w, rect.max.y));
        let fc = egui::Color32::from_rgba_unmultiplied(tc.r(), tc.g(), tc.b(), 215);
        ui.painter().rect_filled(fr, egui::Rounding::same(3.0), fc);
        draw_tactical_ticks(ui, rect);
        let cw = 2.0_f32.min(w);
        let cr = egui::Rect::from_min_max(egui::pos2(fr.max.x - cw, fr.min.y), fr.max);
        ui.painter().rect_filled(cr, egui::Rounding::ZERO, egui::Color32::from_rgb(240, 253, 250));
    }
}
pub fn draw_tactical_ticks(ui: &egui::Ui, rect: egui::Rect) {
    if !rect.width().is_finite()
        || !rect.height().is_finite()
        || !(rect.width() > 6.0 && rect.height() > 6.0)
    {
        return;
    }
    let tick = egui::Color32::from_rgba_unmultiplied(11, 12, 16, 140);
    for k in 1..20 {
        let x = rect.min.x + rect.width() * (k as f32 / 20.0);
        ui.painter().line_segment(
            [egui::pos2(x, rect.min.y + 2.0), egui::pos2(x, rect.max.y - 2.0)],
            egui::Stroke::new(1.0_f32, tick),
        );
    }
}
pub fn cyber_accent(pct: f32) -> egui::Color32 {
    let p = if pct.is_finite() { pct.clamp(0.0, 100.0) } else { 0.0 };
    if p >= 85.0 {
        egui::Color32::from_rgb(255, 42, 84)
    } else {
        egui::Color32::from_rgba_unmultiplied(0, 229, 255, 130)
    }
}
pub fn draw_cyber_corner_brackets(ui: &egui::Ui, rect: egui::Rect, color: egui::Color32) {
    if !rect.width().is_finite()
        || !rect.height().is_finite()
        || !(rect.width() > 6.0 && rect.height() > 6.0)
    {
        return;
    }
    let (l, i) = (7.0_f32, 2.0_f32);
    let s = egui::Stroke::new(1.5_f32, color);
    let p = ui.painter();
    let (x0, x1, y0, y1) = (rect.left() + i, rect.right() - i, rect.top() + i, rect.bottom() - i);
    p.line_segment([egui::pos2(x0, y0), egui::pos2(x0 + l, y0)], s);
    p.line_segment([egui::pos2(x0, y0), egui::pos2(x0, y0 + l)], s);
    p.line_segment([egui::pos2(x1 - l, y0), egui::pos2(x1, y0)], s);
    p.line_segment([egui::pos2(x1, y0), egui::pos2(x1, y0 + l)], s);
    p.line_segment([egui::pos2(x0, y1 - l), egui::pos2(x0, y1)], s);
    p.line_segment([egui::pos2(x0, y1), egui::pos2(x0 + l, y1)], s);
    p.line_segment([egui::pos2(x1, y1 - l), egui::pos2(x1, y1)], s);
    p.line_segment([egui::pos2(x1 - l, y1), egui::pos2(x1, y1)], s);
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_clamp() {
        assert_eq!((0.0f32).clamp(0.0, 1.0), 0.0);
        assert_eq!((1.5f32).clamp(0.0, 1.0), 1.0);
        assert_eq!((-0.5f32).clamp(0.0, 1.0), 0.0);
    }
    #[test]
    fn test_threshold() {
        assert_eq!(cyber_accent(90.0), egui::Color32::from_rgb(255, 42, 84));
        assert_eq!(cyber_accent(50.0), egui::Color32::from_rgba_unmultiplied(0, 229, 255, 130));
    }
    #[test]
    fn test_fill_width() {
        assert_eq!((200.0f32 * 0.5).clamp(2.0, 200.0), 100.0);
        assert_eq!((200.0f32 * 1.0).clamp(2.0, 200.0), 200.0);
    }
    #[test]
    fn test_colors() {
        assert_eq!(egui::Color32::from_rgba_unmultiplied(8, 145, 178, 62).a(), 62);
        assert_eq!(egui::Color32::from_rgba_unmultiplied(185, 28, 28, 72).a(), 72);
    }
    #[test]
    fn test_cyber_accent_non_finite() {
        let ok = cyber_accent(50.0);
        assert_eq!(cyber_accent(f32::NAN), ok);
        assert_eq!(cyber_accent(f32::INFINITY), ok);
        assert_eq!(cyber_accent(f32::NEG_INFINITY), ok);
    }
}
