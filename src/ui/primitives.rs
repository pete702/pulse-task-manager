//! Cockpit graphic primitives for PULSE telemetry HUD.
use crate::theme::colors;
use eframe::egui;

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
        let cw = 2.0_f32.min(w);
        let cr = egui::Rect::from_min_max(egui::pos2(fr.max.x - cw, fr.min.y), fr.max);
        ui.painter().rect_filled(cr, egui::Rounding::ZERO, egui::Color32::from_rgb(240, 253, 250));
    }
    draw_tactical_ticks(ui, rect);
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
    if !rect.width().is_finite() || !rect.height().is_finite() {
        return;
    }
    ui.painter().rect_stroke(
        rect,
        egui::Rounding::same(colors::RADIUS),
        egui::Stroke::new(1.0_f32, color.gamma_multiply(0.30)),
    );
}

/// Small soft tinted chip with centered Inter text and no harsh border.
pub fn pill_badge(ui: &mut egui::Ui, text: &str, fg: egui::Color32) -> egui::Response {
    egui::Frame::none()
        .fill(fg.gamma_multiply(0.13))
        .stroke(egui::Stroke::NONE)
        .rounding(egui::Rounding::same(5.0))
        .inner_margin(egui::Margin::symmetric(8.0, 3.0))
        .show(ui, |ui| {
            let rich = egui::RichText::new(text).font(egui::FontId::proportional(11.5)).color(fg);
            ui.add(egui::Label::new(rich).wrap(false));
        })
        .response
}

/// Tighter chip used in the 28 px status bar.
pub fn pill_badge_tight(ui: &mut egui::Ui, text: &str, fg: egui::Color32) -> egui::Response {
    egui::Frame::none()
        .fill(fg.gamma_multiply(0.13))
        .stroke(egui::Stroke::NONE)
        .rounding(egui::Rounding::same(5.0))
        .inner_margin(egui::Margin::symmetric(5.0, 1.0))
        .show(ui, |ui| {
            let rich = egui::RichText::new(text).font(egui::FontId::proportional(10.5)).color(fg);
            ui.add(egui::Label::new(rich).wrap(false));
        })
        .response
}

/// Rounded clickable pill used for every header and footer action.
pub fn pill_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    pill_button_colored(ui, label, colors::TEXT_PRIMARY)
}

/// Colored variant used for active navigation pills.
pub fn pill_button_colored(ui: &mut egui::Ui, label: &str, color: egui::Color32) -> egui::Response {
    let button = egui::Button::new(
        egui::RichText::new(label).font(egui::FontId::proportional(11.5)).color(color),
    )
    .rounding(egui::Rounding::same(colors::PILL_RADIUS));
    ui.add(button).on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// Exact-sized clickable pill: emerald dot when on, muted dot when off.
pub fn pill_toggle(ui: &mut egui::Ui, label: &str, value: &mut bool) -> egui::Response {
    let text_col = if *value { colors::TEXT_PRIMARY } else { colors::TEXT_SECONDARY };
    let font = egui::FontId::proportional(11.5);
    let galley = ui.painter().layout_no_wrap(label.to_owned(), font, text_col);
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(28.0 + galley.size().x, 22.0), egui::Sense::click());
    let (fill, border) = if response.is_pointer_button_down_on() {
        (colors::ACTIVE, colors::ACCENT_CYAN)
    } else if response.hovered() {
        (colors::HOVER, colors::ACCENT_CYAN)
    } else if *value {
        (colors::ELEVATED, colors::BORDER)
    } else {
        (colors::BG_CARD, colors::BORDER)
    };
    let p = ui.painter();
    let r = egui::Rounding::same(colors::PILL_RADIUS);
    p.rect_filled(rect, r, fill);
    p.rect_stroke(rect, r, egui::Stroke::new(1.0_f32, border));
    let dot = if *value { colors::EMERALD } else { colors::TEXT_MUTED };
    p.circle_filled(egui::pos2(rect.left() + 11.0, rect.center().y), 3.0_f32, dot);
    p.galley(
        egui::pos2(rect.left() + 19.0, rect.center().y - galley.size().y * 0.5),
        galley,
        text_col,
    );
    if response.clicked() {
        *value = !*value;
    }
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// Draws a semantic status dot without any literal Unicode glyph.
pub fn draw_status_dot(ui: &egui::Ui, center: egui::Pos2, color: egui::Color32) {
    ui.painter().circle_filled(center, 3.0_f32, color);
}

/// Maps a value into the sparkline band using explicit fixed bounds.
pub fn spark_y(value: f32, min: f32, max: f32, top: f32, bottom: f32) -> f32 {
    let v = if value.is_finite() { value.clamp(min, max) } else { min };
    bottom - ((v - min) / (max - min).max(1e-6)) * (bottom - top)
}

/// Clipped gap-aware sparkline: `None` breaks the line instead of compressing time.
/// Percentage rows pass `0.0..100.0`; rate rows pass `0.0..hi.max(1.0)`.
pub fn draw_sparkline(
    ui: &egui::Ui,
    rect: egui::Rect,
    samples: &[Option<f32>],
    color: egui::Color32,
    min: f32,
    max: f32,
) {
    if rect.width() <= 8.0 || rect.height() <= 4.0 || max <= min || samples.len() < 2 {
        return;
    }
    let painter = ui.painter().with_clip_rect(rect);
    let (top, bottom) = (rect.top() + 2.0, rect.bottom() - 2.0);
    let step = rect.width() / (samples.len() - 1) as f32;
    let mut seg: Vec<egui::Pos2> = Vec::new();
    let flush = |seg: &mut Vec<egui::Pos2>| {
        if seg.len() == 1 {
            let p = seg[0];
            seg.insert(0, egui::pos2(p.x - 1.5, p.y));
            seg.push(egui::pos2(p.x + 1.5, p.y));
        }
        if seg.len() >= 2 {
            let fill = color.gamma_multiply(0.16);
            for pair in seg.windows(2) {
                let area = egui::Rect::from_min_max(
                    egui::pos2(pair[0].x, pair[0].y.min(pair[1].y)),
                    egui::pos2((pair[1].x + 0.8_f32).min(rect.right()), bottom),
                );
                painter.rect_filled(area, egui::Rounding::ZERO, fill);
            }
            painter.add(egui::Shape::line(std::mem::take(seg), egui::Stroke::new(1.5_f32, color)));
        }
        seg.clear();
    };
    for (i, sample) in samples.iter().enumerate() {
        match sample.filter(|v| v.is_finite()) {
            Some(v) => seg
                .push(egui::pos2(rect.left() + i as f32 * step, spark_y(v, min, max, top, bottom))),
            None => flush(&mut seg),
        }
    }
    flush(&mut seg);
}

/// Slim rounded progress bar for compact metric cards.
pub fn mini_progress(ui: &egui::Ui, rect: egui::Rect, ratio: f32, color: egui::Color32) {
    if rect.width() <= 2.0 || rect.height() <= 2.0 {
        return;
    }
    let r = if ratio.is_finite() { ratio.clamp(0.0, 1.0) } else { 0.0 };
    let painter = ui.painter();
    painter.rect_filled(rect, egui::Rounding::same(3.0_f32), colors::BORDER_SUBTLE);
    if r > 0.0 {
        let filled = egui::Rect::from_min_max(
            rect.min,
            egui::pos2(rect.left() + rect.width() * r, rect.bottom()),
        );
        painter.rect_filled(filled, egui::Rounding::same(3.0_f32), color);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pure_helpers() {
        assert_eq!((0.0f32).clamp(0.0, 1.0), 0.0);
        assert_eq!((1.5f32).clamp(0.0, 1.0), 1.0);
        assert_eq!((-0.5f32).clamp(0.0, 1.0), 0.0);
        assert_eq!((200.0f32 * 0.5).clamp(2.0, 200.0), 100.0);
        assert_eq!((200.0f32 * 1.0).clamp(2.0, 200.0), 200.0);
    }

    #[test]
    fn sparkline_uses_fixed_0_to_100_bounds() {
        assert_eq!(spark_y(150.0, 0.0, 100.0, 0.0, 100.0), 0.0);
        assert_eq!(spark_y(-5.0, 0.0, 100.0, 0.0, 100.0), 100.0);
        assert_eq!(spark_y(50.0, 0.0, 100.0, 0.0, 100.0), 50.0);
        assert_eq!(spark_y(f32::NAN, 0.0, 100.0, 0.0, 100.0), 100.0);
        assert_eq!(spark_y(1.0, 0.0, 4.0, 0.0, 100.0), 75.0);
    }

    #[test]
    fn test_cyber_accent_rules() {
        assert_eq!(cyber_accent(90.0), egui::Color32::from_rgb(255, 42, 84));
        let normal = cyber_accent(50.0);
        assert_eq!(normal, egui::Color32::from_rgba_unmultiplied(0, 229, 255, 130));
        assert_eq!(cyber_accent(f32::NAN), normal);
        assert_eq!(cyber_accent(f32::INFINITY), normal);
        assert_eq!(cyber_accent(f32::NEG_INFINITY), normal);
    }
}
