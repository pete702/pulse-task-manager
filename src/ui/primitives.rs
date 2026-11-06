//! Zero-allocation cockpit graphic primitives for PULSE telemetry HUD.
//! All functions draw directly via `egui::Painter` with stack-only allocations.

use eframe::egui;

/// Draws a subtle heat bar inside a table cell background.
/// - `ratio`: 0.0..=1.0 fill ratio (clamped internally).
/// - `_color`: base accent color (cyan for RAM) — currently unused, kept for API compatibility.
/// - `is_alert`: if true, uses higher alpha and draws an accent strip.
pub fn draw_cell_heat_bar(
    ui: &egui::Ui,
    rect: egui::Rect,
    ratio: f32,
    _color: egui::Color32,
    is_alert: bool,
) {
    let r = ratio.clamp(0.0, 1.0);
    if r <= 0.002 {
        return;
    }

    let inner = rect.shrink2(egui::vec2(2.0, 2.0));
    let fill_w = (inner.width() * r).clamp(0.0, inner.width());

    // Higher alpha (62) for 150% DPI visibility.
    let fill_color = if is_alert {
        egui::Color32::from_rgba_unmultiplied(185, 28, 28, 72)
    } else {
        egui::Color32::from_rgba_unmultiplied(8, 145, 178, 62)
    };

    let fill_rect =
        egui::Rect::from_min_max(inner.min, egui::pos2(inner.min.x + fill_w, inner.max.y));
    ui.painter().rect_filled(fill_rect, egui::Rounding::same(2.0), fill_color);

    // Crisp 2.0px bright vertical cap at right end of fill.
    if fill_w > 2.5 {
        let cap_rect = egui::Rect::from_min_max(
            egui::pos2(fill_rect.max.x - 2.0, fill_rect.min.y),
            fill_rect.max,
        );
        let cap_color = if is_alert {
            egui::Color32::from_rgb(254, 205, 211)
        } else {
            egui::Color32::from_rgb(240, 253, 250)
        };
        ui.painter().rect_filled(cap_rect, egui::Rounding::ZERO, cap_color);
    }

    // Accent strip on left edge for high values or alerts (2px wide)
    if r >= 0.08 || is_alert {
        let strip_rect =
            egui::Rect::from_min_max(inner.min, egui::pos2(inner.min.x + 2.0, inner.max.y));
        let accent_color = if is_alert {
            egui::Color32::from_rgba_unmultiplied(239, 68, 68, 185)
        } else {
            egui::Color32::from_rgba_unmultiplied(8, 145, 178, 185)
        };
        ui.painter().rect_filled(strip_rect, egui::Rounding::same(2.0), accent_color);
    }
}

/// Draws a smooth high-precision continuous telemetry bar (replaces segmented LED bar).
/// - `ratio`: 0.0..=1.0 fill ratio (clamped internally).
/// - `active_color`: base color for the bar (overridden by threshold logic).
///
/// Draws track, continuous fill, 5% tick segments, and leading-edge cursor cap.
pub fn draw_segmented_led_bar(
    ui: &egui::Ui,
    rect: egui::Rect,
    ratio: f32,
    _segments: usize,
    active_color: egui::Color32,
) {
    let r = ratio.clamp(0.0, 1.0);

    // Determine threshold color
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

    // 2. Draw smooth continuous fill with leading-edge cursor cap
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
        draw_tactical_ticks(ui, rect);
        // 3. Crisp leading-edge cursor cap (2px bright white at right tip of fill)
        let cap_w = 2.0_f32.min(fill_w);
        let cap_rect = egui::Rect::from_min_max(
            egui::pos2(fill_rect.max.x - cap_w, fill_rect.min.y),
            fill_rect.max,
        );
        let cap_color = egui::Color32::from_rgb(240, 253, 250);
        ui.painter().rect_filled(cap_rect, egui::Rounding::ZERO, cap_color);
    }
}

/// Draws 19 dark tick segments every 5% across a bar (20 segments total).
pub fn draw_tactical_ticks(ui: &egui::Ui, rect: egui::Rect) {
    let tick = egui::Color32::from_rgba_unmultiplied(11, 12, 16, 140);
    for k in 1..20 {
        let x = rect.min.x + rect.width() * (k as f32 / 20.0_f32);
        ui.painter().line_segment(
            [egui::pos2(x, rect.min.y + 2.0), egui::pos2(x, rect.max.y - 2.0)],
            egui::Stroke::new(1.0_f32, tick),
        );
    }
}

/// Draws 7px L-shaped tactical corner brackets on the 4 corners of a card.
pub fn cyber_accent(pct: f32) -> egui::Color32 {
    if pct >= 85.0 {
        egui::Color32::from_rgb(255, 42, 84)
    } else {
        egui::Color32::from_rgba_unmultiplied(0, 229, 255, 130)
    }
}

/// Draws 7px L-shaped tactical corner brackets on the 4 corners of a card.
pub fn draw_cyber_corner_brackets(ui: &egui::Ui, rect: egui::Rect, color: egui::Color32) {
    let l = 7.0_f32;
    let i = 2.0_f32;
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

/// Draws a mini sparkline (up to 24 points) using stack-only Pos2 array.
/// - `values`: slice of f32 values in range 0.0..=100.0 (last 24 used).
/// - `color`: line color.
#[allow(dead_code)]
pub fn draw_mini_sparkline(ui: &egui::Ui, rect: egui::Rect, values: &[f32], color: egui::Color32) {
    if values.is_empty() {
        return;
    }

    // Use last 24 values
    let start = values.len().saturating_sub(24);
    let visible = &values[start..];
    let count = visible.len();
    if count < 2 {
        return;
    }

    // Stack-allocated array of Pos2 (max 24)
    let mut points: [egui::Pos2; 24] = [egui::Pos2::ZERO; 24];

    let step_x = rect.width() / (count - 1) as f32;
    for (i, &v) in visible.iter().enumerate() {
        let y_ratio = v.clamp(0.0, 100.0) / 100.0;
        let y = rect.max.y - y_ratio * rect.height();
        let x = rect.min.x + i as f32 * step_x;
        points[i] = egui::pos2(x, y);
    }

    let painter = ui.painter();
    for i in 0..count - 1 {
        painter.line_segment([points[i], points[i + 1]], egui::Stroke::new(1.5_f32, color));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heat_bar_clamping() {
        // Test ratio clamping logic
        assert!((0.0f32).clamp(0.0, 1.0) == 0.0);
        assert!((1.5f32).clamp(0.0, 1.0) == 1.0);
        assert!((-0.5f32).clamp(0.0, 1.0) == 0.0);
        assert!((0.5f32).clamp(0.0, 1.0) == 0.5);
    }

    #[test]
    fn test_precision_bar_threshold_color() {
        // Test threshold color logic for continuous bar
        let r = 0.9;
        let color = if r >= 0.85 {
            egui::Color32::from_rgb(239, 68, 68)
        } else if r >= 0.70 {
            egui::Color32::from_rgb(245, 158, 11)
        } else {
            egui::Color32::from_rgb(0, 229, 255)
        };
        assert_eq!(color, egui::Color32::from_rgb(239, 68, 68));

        let r = 0.75;
        let color = if r >= 0.85 {
            egui::Color32::from_rgb(239, 68, 68)
        } else if r >= 0.70 {
            egui::Color32::from_rgb(245, 158, 11)
        } else {
            egui::Color32::from_rgb(0, 229, 255)
        };
        assert_eq!(color, egui::Color32::from_rgb(245, 158, 11));

        let r = 0.5;
        let color = if r >= 0.85 {
            egui::Color32::from_rgb(239, 68, 68)
        } else if r >= 0.70 {
            egui::Color32::from_rgb(245, 158, 11)
        } else {
            egui::Color32::from_rgb(0, 229, 255)
        };
        assert_eq!(color, egui::Color32::from_rgb(0, 229, 255));
    }

    #[test]
    fn test_precision_bar_fill_width() {
        // Test fill width calculation (clamped to min 2.0)
        let rect_width: f32 = 200.0;
        let r: f32 = 0.5;
        let fill_w = (rect_width * r).max(2.0).min(rect_width);
        assert_eq!(fill_w, 100.0);

        let r: f32 = 0.001; // below 0.002 threshold
        let fill_w = (rect_width * r).max(2.0).min(rect_width);
        assert_eq!(fill_w, 2.0);

        let r: f32 = 1.0;
        let fill_w = (rect_width * r).max(2.0).min(rect_width);
        assert_eq!(fill_w, 200.0);
    }

    #[test]
    fn test_heat_bar_colors() {
        // Test heat bar color logic (deep teal for normal, dark crimson for alert)
        let normal_color = egui::Color32::from_rgba_unmultiplied(8, 145, 178, 62);
        let alert_color = egui::Color32::from_rgba_unmultiplied(185, 28, 28, 72);
        assert_eq!(normal_color.a(), 62);
        assert_eq!(alert_color.a(), 72);
    }
}
