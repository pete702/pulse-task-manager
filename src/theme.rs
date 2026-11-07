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

    // ===== v0.0.6 New Colors =====
    /// Primary text (bright white): #F0F6FC
    pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(0xF0, 0xF6, 0xFC);
    /// Secondary text (silver-blue): #94A3B8
    pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(0x94, 0xA3, 0xB8);
    /// Accent cyan (neon): #00F0FF
    pub const ACCENT_CYAN: Color32 = Color32::from_rgb(0x00, 0xF0, 0xFF);
    /// Accent amber (warning): #FFB800
    pub const ACCENT_AMBER: Color32 = Color32::from_rgb(0xFF, 0xB8, 0x00);
    /// Accent red (critical): #FF2A54
    pub const ACCENT_RED: Color32 = Color32::from_rgb(0xFF, 0x2A, 0x54);
    /// Accent purple (swap): #BD00FF
    pub const ACCENT_PURPLE: Color32 = Color32::from_rgb(0xBD, 0x00, 0xFF);
    /// Sidebar active background: #1A2234
    pub const SIDEBAR_ACTIVE_BG: Color32 = Color32::from_rgb(0x1A, 0x22, 0x34);
    /// Sidebar active border: #00F0FF
    #[allow(dead_code)]
    pub const SIDEBAR_ACTIVE_BORDER: Color32 = Color32::from_rgb(0x00, 0xF0, 0xFF);
    /// Status bar background: #0A0B10
    pub const STATUS_BAR_BG: Color32 = Color32::from_rgb(0x0A, 0x0B, 0x10);
}

/// Three Walls badge and muted-text tokens (v0.1.0 aliases).
#[allow(dead_code)]
pub const CRITICAL_CORAL: egui::Color32 = colors::ACCENT_RED;
#[allow(dead_code)]
pub const WARNING_AMBER: egui::Color32 = colors::ACCENT_AMBER;
pub const OPTIMAL_EMERALD: egui::Color32 = egui::Color32::from_rgb(0x00, 0xE0, 0x80);
pub const TEXT_MUTED: egui::Color32 = colors::TEXT_SECONDARY;

/// Traffic-light load color — returns color based on load percentage.
/// <60% — cyan (#00F0FF), 60-85% — amber (#FFB800), ≥85% — neon red (#FF2A54).
#[allow(dead_code)]
#[inline]
pub fn load_color(percent: f32) -> egui::Color32 {
    if !percent.is_finite() {
        return colors::TEXT_SECONDARY;
    }
    if percent < 60.0 {
        colors::ACCENT_CYAN
    } else if percent < 85.0 {
        colors::ACCENT_AMBER
    } else {
        colors::ACCENT_RED
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
    style.visuals.widgets.noninteractive.fg_stroke =
        egui::Stroke::new(1.0_f32, colors::TEXT_PRIMARY);

    // Widget colors — inactive
    style.visuals.widgets.inactive.bg_fill = colors::BG_CARD;
    style.visuals.widgets.inactive.bg_stroke =
        egui::Stroke::new(colors::BORDER_WIDTH, colors::BORDER);
    style.visuals.widgets.inactive.rounding = egui::Rounding::same(colors::RADIUS);
    style.visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0_f32, colors::TEXT_PRIMARY);

    // Widget colors — hovered
    style.visuals.widgets.hovered.bg_fill = colors::HOVER;
    style.visuals.widgets.hovered.bg_stroke =
        egui::Stroke::new(colors::BORDER_WIDTH, colors::ACCENT_CYAN);
    style.visuals.widgets.hovered.rounding = egui::Rounding::same(colors::RADIUS);
    style.visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0_f32, colors::TEXT_PRIMARY);

    // Widget colors — active
    style.visuals.widgets.active.bg_fill = colors::ACTIVE;
    style.visuals.widgets.active.bg_stroke =
        egui::Stroke::new(colors::BORDER_WIDTH, colors::ACCENT_CYAN);
    style.visuals.widgets.active.rounding = egui::Rounding::same(colors::RADIUS);
    style.visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0_f32, colors::TEXT_PRIMARY);

    // Selection (for tables, etc.)
    style.visuals.selection.bg_fill = colors::SIDEBAR_ACTIVE_BG;
    style.visuals.selection.stroke = egui::Stroke::new(1.0_f32, colors::ACCENT_CYAN);

    // Text style: monospace for numbers
    style
        .text_styles
        .insert(egui::TextStyle::Monospace, egui::FontId::new(13.0, egui::FontFamily::Monospace));
    // Default text color
    style
        .text_styles
        .insert(egui::TextStyle::Body, egui::FontId::new(14.0, egui::FontFamily::Proportional));

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

/// Returns an `egui::Frame` for sidebar active tab.
/// Background: #1A2234, Border: 2px #00F0FF, Radius: 6.0, Inner margin: 8.0.
#[allow(dead_code)]
pub fn sidebar_active_frame() -> egui::Frame {
    egui::Frame::none()
        .fill(colors::SIDEBAR_ACTIVE_BG)
        .stroke(egui::Stroke::new(2.0_f32, colors::SIDEBAR_ACTIVE_BORDER))
        .rounding(egui::Rounding::same(colors::RADIUS))
        .inner_margin(egui::Margin::same(8.0))
}

/// Returns an `egui::Frame` for status bar.
/// Background: #0A0B10, Border top: 1px #1F2330, Inner margin: 8.0.
pub fn status_bar_frame() -> egui::Frame {
    egui::Frame::none()
        .fill(colors::STATUS_BAR_BG)
        .stroke(egui::Stroke::new(1.0_f32, colors::BORDER))
        .rounding(egui::Rounding::same(0.0))
        .inner_margin(egui::Margin::symmetric(12.0, 8.0))
}

// Re-export font setup from separate module
pub use crate::theme_fonts::setup_fonts;

#[cfg(test)]
#[path = "theme_tests.rs"]
mod theme_tests;
