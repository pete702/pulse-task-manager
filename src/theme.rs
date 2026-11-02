//! "Obsidian HUD" theme for PULSE.
//! Contains design tokens, theme application function, traffic-light load color, and card Frame.

use eframe::egui;

/// Obsidian HUD theme color tokens (per ARCHITECTURE.md specification).
pub mod colors {
    use eframe::egui::Color32;

    /// Main window background: #0B0C10
    pub const BG_WINDOW: Color32 = Color32::from_rgb(0x0B, 0x0C, 0x10);
    /// Card/panel background: #12141C
    pub const BG_CARD: Color32 = Color32::from_rgb(0x12, 0x14, 0x1C);
    /// Borders/dividers: #1F2330
    pub const BORDER: Color32 = Color32::from_rgb(0x1F, 0x23, 0x30);
    /// Widget hover state: #1A1E2E
    pub const HOVER: Color32 = Color32::from_rgb(0x1A, 0x1E, 0x2E);
    /// Widget active state: #1F2330
    pub const ACTIVE: Color32 = Color32::from_rgb(0x1F, 0x23, 0x30);
    /// Corner radius: 6.0
    pub const RADIUS: f32 = 6.0;
    /// Border width: 1.0
    pub const BORDER_WIDTH: f32 = 1.0;
}

/// Traffic-light load color — returns color based on load percentage.
/// <60% — cyan (#00F0FF), 60-85% — amber (#FFB800), ≥85% — neon red (#FF2A54).
#[allow(dead_code)]
#[inline]
pub fn load_color(percent: f32) -> egui::Color32 {
    if percent < 60.0 {
        egui::Color32::from_rgb(0x00, 0xF0, 0xFF) // Cyan
    } else if percent < 85.0 {
        egui::Color32::from_rgb(0xFF, 0xB8, 0x00) // Amber
    } else {
        egui::Color32::from_rgb(0xFF, 0x2A, 0x54) // Neon Red
    }
}

/// Applies the "Obsidian HUD" theme to the egui context.
/// Sets colors, rounding, fonts (monospace for numbers).
pub fn apply_obsidian_theme(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();

    // Color tokens
    style.visuals.window_fill = colors::BG_WINDOW;
    style.visuals.panel_fill = colors::BG_CARD;
    style.visuals.window_stroke = egui::Stroke::new(colors::BORDER_WIDTH, colors::BORDER);
    style.visuals.window_rounding = egui::Rounding::same(colors::RADIUS);

    // Widget colors — noninteractive
    style.visuals.widgets.noninteractive.bg_fill = colors::BG_CARD;
    style.visuals.widgets.noninteractive.bg_stroke =
        egui::Stroke::new(colors::BORDER_WIDTH, colors::BORDER);
    style.visuals.widgets.noninteractive.rounding = egui::Rounding::same(colors::RADIUS);

    // Widget colors — inactive
    style.visuals.widgets.inactive.bg_fill = colors::BG_CARD;
    style.visuals.widgets.inactive.bg_stroke =
        egui::Stroke::new(colors::BORDER_WIDTH, colors::BORDER);
    style.visuals.widgets.inactive.rounding = egui::Rounding::same(colors::RADIUS);

    // Widget colors — hovered
    style.visuals.widgets.hovered.bg_fill = colors::HOVER;
    style.visuals.widgets.hovered.bg_stroke =
        egui::Stroke::new(colors::BORDER_WIDTH, colors::BORDER);
    style.visuals.widgets.hovered.rounding = egui::Rounding::same(colors::RADIUS);

    // Widget colors — active
    style.visuals.widgets.active.bg_fill = colors::ACTIVE;
    style.visuals.widgets.active.bg_stroke =
        egui::Stroke::new(colors::BORDER_WIDTH, colors::BORDER);
    style.visuals.widgets.active.rounding = egui::Rounding::same(colors::RADIUS);

    // Text style: monospace for numbers
    style.text_styles.insert(
        egui::TextStyle::Monospace,
        egui::FontId::new(13.0, egui::FontFamily::Monospace),
    );

    ctx.set_style(style);
}

/// Returns an `egui::Frame` for cards in Obsidian HUD style.
/// Background: #12141C, Border: 1px #1F2330, Radius: 6.0, Inner margin: 12.0.
pub fn card_frame() -> egui::Frame {
    egui::Frame::none()
        .fill(colors::BG_CARD)
        .stroke(egui::Stroke::new(colors::BORDER_WIDTH, colors::BORDER))
        .rounding(egui::Rounding::same(colors::RADIUS))
        .inner_margin(egui::Margin::same(12.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_colors_constants() {
        assert_eq!(colors::BG_WINDOW, egui::Color32::from_rgb(11, 12, 16));
        assert_eq!(colors::BG_CARD, egui::Color32::from_rgb(18, 20, 28));
        assert_eq!(colors::BORDER, egui::Color32::from_rgb(31, 35, 48));
        assert_eq!(colors::HOVER, egui::Color32::from_rgb(26, 30, 46));
        assert_eq!(colors::ACTIVE, egui::Color32::from_rgb(31, 35, 48));
        assert_eq!(colors::RADIUS, 6.0);
        assert_eq!(colors::BORDER_WIDTH, 1.0);
    }

    #[test]
    fn test_load_color_cyan_under_60() {
        let color = load_color(59.9);
        assert_eq!(color, egui::Color32::from_rgb(0x00, 0xF0, 0xFF));
        let color = load_color(0.0);
        assert_eq!(color, egui::Color32::from_rgb(0x00, 0xF0, 0xFF));
        let color = load_color(50.0);
        assert_eq!(color, egui::Color32::from_rgb(0x00, 0xF0, 0xFF));
    }

    #[test]
    fn test_load_color_amber_60_to_85() {
        let color = load_color(60.0);
        assert_eq!(color, egui::Color32::from_rgb(0xFF, 0xB8, 0x00));
        let color = load_color(75.0);
        assert_eq!(color, egui::Color32::from_rgb(0xFF, 0xB8, 0x00));
        let color = load_color(84.9);
        assert_eq!(color, egui::Color32::from_rgb(0xFF, 0xB8, 0x00));
    }

    #[test]
    fn test_load_color_red_85_and_above() {
        let color = load_color(85.0);
        assert_eq!(color, egui::Color32::from_rgb(0xFF, 0x2A, 0x54));
        let color = load_color(95.0);
        assert_eq!(color, egui::Color32::from_rgb(0xFF, 0x2A, 0x54));
        let color = load_color(100.0);
        assert_eq!(color, egui::Color32::from_rgb(0xFF, 0x2A, 0x54));
    }

    #[test]
    fn test_card_frame_properties() {
        let frame = card_frame();
        assert_eq!(frame.fill, colors::BG_CARD);
        assert_eq!(frame.stroke.width, colors::BORDER_WIDTH);
        assert_eq!(frame.stroke.color, colors::BORDER);
        assert_eq!(frame.rounding, egui::Rounding::same(colors::RADIUS));
        assert_eq!(frame.inner_margin.left, 12.0);
        assert_eq!(frame.inner_margin.right, 12.0);
        assert_eq!(frame.inner_margin.top, 12.0);
        assert_eq!(frame.inner_margin.bottom, 12.0);
    }
}
