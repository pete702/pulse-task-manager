//! "Obsidian Cyan Cockpit" theme for PULSE.
//! Design tokens, theme application, semantic load color and shared Frames.

use eframe::egui;

/// Obsidian Cyan Cockpit color tokens.
pub mod colors {
    use eframe::egui::Color32;

    /// Main window background: #090D12
    pub const BG_WINDOW: Color32 = Color32::from_rgb(0x09, 0x0D, 0x12);
    /// Card/panel background: #10161F
    pub const BG_CARD: Color32 = Color32::from_rgb(0x10, 0x16, 0x1F);
    /// Elevated surfaces and selected pills: #151D28
    pub const ELEVATED: Color32 = Color32::from_rgb(0x15, 0x1D, 0x28);
    /// Borders/dividers: #1A2433
    pub const BORDER: Color32 = Color32::from_rgb(0x1A, 0x24, 0x33);
    /// Subtle separators: #151D28
    pub const BORDER_SUBTLE: Color32 = Color32::from_rgb(0x15, 0x1D, 0x28);
    /// Widget hover state: #1F2B3D
    pub const HOVER: Color32 = Color32::from_rgb(0x1F, 0x2B, 0x3D);
    /// Widget pressed/selected state: #0E3A4A
    pub const ACTIVE: Color32 = Color32::from_rgb(0x0E, 0x3A, 0x4A);
    /// Card corner radius: 8.0
    pub const RADIUS: f32 = 8.0;
    /// Pill/button corner radius: 6.0
    pub const PILL_RADIUS: f32 = 6.0;
    /// Card corner radius (explicit alias): 8.0
    pub const CARD_RADIUS: f32 = 8.0;
    /// Badge corner radius: 4.0
    pub const BADGE_RADIUS: f32 = 4.0;
    /// Border width: 1.0
    pub const BORDER_WIDTH: f32 = 1.0;

    /// Primary text: #F1F5F9
    pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(0xF1, 0xF5, 0xF9);
    /// Secondary text: #94A3B8
    pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(0x94, 0xA3, 0xB8);
    /// Muted text: #64748B
    pub const TEXT_MUTED: Color32 = Color32::from_rgb(0x64, 0x74, 0x8B);

    /// Accent cyan: #22D3EE (green nudged by one for lossless alpha round-trips)
    pub const ACCENT_CYAN: Color32 = Color32::from_rgb(0x21, 0xD2, 0xEE);
    /// Accent amber: #F59E0B
    pub const ACCENT_AMBER: Color32 = Color32::from_rgb(0xF5, 0x9E, 0x0B);
    /// Accent red: #EF4444 (channels nudged for lossless alpha round-trips)
    pub const ACCENT_RED: Color32 = Color32::from_rgb(0xEE, 0x43, 0x43);
    /// Accent purple: #A855F7
    pub const ACCENT_PURPLE: Color32 = Color32::from_rgb(0xA8, 0x55, 0xF7);
    /// Nominal emerald: #10B981
    pub const EMERALD: Color32 = Color32::from_rgb(0x10, 0xB9, 0x81);

    /// Sidebar active background (elevated pill): #151D28
    pub const SIDEBAR_ACTIVE_BG: Color32 = Color32::from_rgb(0x15, 0x1D, 0x28);
    /// Status bar background: #090D12
    pub const STATUS_BAR_BG: Color32 = Color32::from_rgb(0x09, 0x0D, 0x12);
}

/// Nominal/"Three Walls" emerald token (kept for existing callers).
pub const OPTIMAL_EMERALD: egui::Color32 = colors::EMERALD;

/// Traffic-light load color: <60% cyan, 60-85% amber, >=85% red.
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

/// Applies the "Obsidian Cyan Cockpit" theme to the egui context.
pub fn apply_obsidian_theme(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();

    style.visuals.window_fill = colors::BG_WINDOW;
    style.visuals.panel_fill = colors::BG_CARD;
    style.visuals.window_stroke = egui::Stroke::new(colors::BORDER_WIDTH, colors::BORDER);
    style.visuals.window_rounding = egui::Rounding::same(colors::CARD_RADIUS);

    let noninteractive = &mut style.visuals.widgets.noninteractive;
    noninteractive.bg_fill = colors::BG_CARD;
    noninteractive.bg_stroke = egui::Stroke::new(colors::BORDER_WIDTH, colors::BORDER);
    noninteractive.rounding = egui::Rounding::same(colors::PILL_RADIUS);
    noninteractive.fg_stroke = egui::Stroke::new(1.0_f32, colors::TEXT_PRIMARY);

    let inactive = &mut style.visuals.widgets.inactive;
    inactive.bg_fill = colors::ELEVATED;
    inactive.bg_stroke = egui::Stroke::new(colors::BORDER_WIDTH, colors::BORDER);
    inactive.rounding = egui::Rounding::same(colors::PILL_RADIUS);
    inactive.fg_stroke = egui::Stroke::new(1.0_f32, colors::TEXT_PRIMARY);

    let hovered = &mut style.visuals.widgets.hovered;
    hovered.bg_fill = colors::HOVER;
    hovered.bg_stroke = egui::Stroke::new(1.0_f32, colors::ACCENT_CYAN);
    hovered.rounding = egui::Rounding::same(colors::PILL_RADIUS);
    hovered.fg_stroke = egui::Stroke::new(1.0_f32, egui::Color32::WHITE);

    let active = &mut style.visuals.widgets.active;
    active.bg_fill = colors::ACTIVE;
    active.bg_stroke = egui::Stroke::new(1.0_f32, colors::ACCENT_CYAN);
    active.rounding = egui::Rounding::same(colors::PILL_RADIUS);
    active.fg_stroke = egui::Stroke::new(1.0_f32, egui::Color32::WHITE);

    style.visuals.selection.bg_fill = colors::SIDEBAR_ACTIVE_BG;
    style.visuals.selection.stroke = egui::Stroke::new(1.0_f32, colors::ACCENT_CYAN);

    style
        .text_styles
        .insert(egui::TextStyle::Monospace, egui::FontId::new(12.5, egui::FontFamily::Monospace));
    style
        .text_styles
        .insert(egui::TextStyle::Body, egui::FontId::new(13.0, egui::FontFamily::Proportional));
    style
        .text_styles
        .insert(egui::TextStyle::Button, egui::FontId::new(12.0, egui::FontFamily::Proportional));
    style.interaction.selectable_labels = false;

    ctx.set_style(style);
}

/// Returns an `egui::Frame` for cards: #0F151D fill, 1px #1E293B, 8px radius.
pub fn card_frame() -> egui::Frame {
    egui::Frame::none()
        .fill(colors::BG_CARD)
        .stroke(egui::Stroke::new(colors::BORDER_WIDTH, colors::BORDER))
        .rounding(egui::Rounding::same(colors::RADIUS))
        .inner_margin(egui::Margin::same(12.0))
}

/// Returns an `egui::Frame` for the status bar.
pub fn status_bar_frame() -> egui::Frame {
    egui::Frame::none()
        .fill(colors::STATUS_BAR_BG)
        .stroke(egui::Stroke::new(1.0_f32, colors::BORDER))
        .rounding(egui::Rounding::ZERO)
        .inner_margin(egui::Margin::symmetric(12.0, 8.0))
}

pub use crate::theme_fonts::setup_fonts;

#[cfg(test)]
#[path = "theme_tests.rs"]
mod theme_tests;
