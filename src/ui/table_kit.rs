//! Shared table helpers for PULSE process tables.

use eframe::egui;

/// Thin bottom-edge usage bar; digits above stay clean/unsliced.
pub fn draw_cell_usage_bar(ui: &egui::Ui, rect: egui::Rect, ratio: f32, alert: bool) {
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
    let fc = usage_bar_fill_color(alert);
    let bar_h = 3.0_f32;
    let fr = egui::Rect::from_min_max(
        egui::pos2(inner.min.x, inner.max.y - bar_h),
        egui::pos2(inner.min.x + w, inner.max.y),
    );
    ui.painter().rect_filled(fr, egui::Rounding::same(1.5), fc);
}

/// Fill color for the cell usage bar, derived from theme accent constants.
pub(crate) fn usage_bar_fill_color(is_alert: bool) -> egui::Color32 {
    if is_alert {
        let c = crate::theme::colors::ACCENT_RED;
        egui::Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), 72)
    } else {
        let c = crate::theme::colors::ACCENT_CYAN;
        egui::Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), 62)
    }
}

/// Sort-header indicator formatting.
pub fn sort_header_text(label: &str, active: bool, asc: bool) -> String {
    if active {
        let arrow = if asc { "\u{25B2}" } else { "\u{25BC}" };
        format!("{label} {arrow}")
    } else {
        label.to_owned()
    }
}

/// Single-line Ctrl+C process row text formatter.
pub fn format_copy_row(name: &str, pid: u32, cpu_pct: f32, mem_mb: f64) -> String {
    format!("{name}\t{pid}\t{cpu_pct:.1}\t{mem_mb:.1}")
}

/// Row selection/highlight background helper.
pub fn paint_row_highlight(
    ui: &egui::Ui,
    rect: egui::Rect,
    selected: bool,
    hovered: bool,
    striped: bool,
) {
    if selected {
        ui.painter().rect_filled(
            rect,
            egui::Rounding::same(2.0),
            egui::Color32::from_rgba_unmultiplied(24, 92, 118, 120),
        );
        ui.painter().rect_stroke(
            rect,
            egui::Rounding::same(2.0),
            egui::Stroke::new(1.0_f32, crate::theme::colors::ACCENT_CYAN),
        );
    } else if hovered {
        ui.painter().rect_filled(
            rect,
            egui::Rounding::same(2.0),
            egui::Color32::from_rgba_unmultiplied(38, 52, 68, 95),
        );
    } else if striped {
        ui.painter().rect_filled(
            rect,
            egui::Rounding::same(2.0),
            egui::Color32::from_rgba_unmultiplied(18, 24, 32, 70),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sort_header_inactive() {
        assert_eq!(sort_header_text("PID", false, true), "PID");
    }
    #[test]
    fn sort_header_active_asc() {
        assert_eq!(sort_header_text("PID", true, true), "PID \u{25B2}");
    }
    #[test]
    fn sort_header_active_desc() {
        assert_eq!(sort_header_text("PID", true, false), "PID \u{25BC}");
    }
    #[test]
    fn format_copy_row_basic() {
        assert_eq!(format_copy_row("foo.exe", 123, 4.5, 10.0), "foo.exe\t123\t4.5\t10.0");
    }

    #[test]
    fn usage_bar_fill_color_derives_from_theme() {
        let alert = usage_bar_fill_color(true).to_srgba_unmultiplied();
        let normal = usage_bar_fill_color(false).to_srgba_unmultiplied();
        let red = crate::theme::colors::ACCENT_RED.to_srgba_unmultiplied();
        let cyan = crate::theme::colors::ACCENT_CYAN.to_srgba_unmultiplied();
        assert_eq!(alert[0], red[0]);
        assert_eq!(alert[1], red[1]);
        assert_eq!(alert[2], red[2]);
        assert_eq!(normal[0], cyan[0]);
        assert_eq!(normal[1], cyan[1]);
        assert_eq!(normal[2], cyan[2]);
    }
}
