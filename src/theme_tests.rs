//! Unit tests for the Obsidian HUD theme.

use super::{self as theme, colors};
use egui::Color32;

#[test]
fn test_colors_constants() {
    assert_eq!(colors::BG_WINDOW, Color32::from_rgb(11, 12, 16));
    assert_eq!(colors::BG_CARD, Color32::from_rgb(18, 20, 28));
    assert_eq!(colors::BORDER, Color32::from_rgb(31, 35, 48));
    assert_eq!(colors::HOVER, Color32::from_rgb(26, 30, 46));
    assert_eq!(colors::ACTIVE, Color32::from_rgb(31, 35, 48));
    assert_eq!(colors::RADIUS, 6.0);
    assert_eq!(colors::BORDER_WIDTH, 1.0);
    // v0.0.6 new colors
    assert_eq!(colors::TEXT_PRIMARY, Color32::from_rgb(0xF0, 0xF6, 0xFC));
    assert_eq!(colors::TEXT_SECONDARY, Color32::from_rgb(0x94, 0xA3, 0xB8));
    assert_eq!(colors::ACCENT_CYAN, Color32::from_rgb(0x00, 0xF0, 0xFF));
    assert_eq!(colors::ACCENT_AMBER, Color32::from_rgb(0xFF, 0xB8, 0x00));
    assert_eq!(colors::ACCENT_RED, Color32::from_rgb(0xFF, 0x2A, 0x54));
    assert_eq!(colors::ACCENT_PURPLE, Color32::from_rgb(0xBD, 0x00, 0xFF));
    assert_eq!(colors::SIDEBAR_ACTIVE_BG, Color32::from_rgb(0x1A, 0x22, 0x34));
    assert_eq!(colors::SIDEBAR_ACTIVE_BORDER, Color32::from_rgb(0x00, 0xF0, 0xFF));
    assert_eq!(colors::STATUS_BAR_BG, Color32::from_rgb(0x0A, 0x0B, 0x10));
}

#[test]
fn test_load_color_cyan_under_60() {
    let color = theme::load_color(59.9);
    assert_eq!(color, colors::ACCENT_CYAN);
    let color = theme::load_color(0.0);
    assert_eq!(color, colors::ACCENT_CYAN);
    let color = theme::load_color(50.0);
    assert_eq!(color, colors::ACCENT_CYAN);
}

#[test]
fn test_load_color_amber_60_to_85() {
    let color = theme::load_color(60.0);
    assert_eq!(color, colors::ACCENT_AMBER);
    let color = theme::load_color(75.0);
    assert_eq!(color, colors::ACCENT_AMBER);
    let color = theme::load_color(84.9);
    assert_eq!(color, colors::ACCENT_AMBER);
}

#[test]
fn test_load_color_red_85_and_above() {
    let color = theme::load_color(85.0);
    assert_eq!(color, colors::ACCENT_RED);
    let color = theme::load_color(95.0);
    assert_eq!(color, colors::ACCENT_RED);
    let color = theme::load_color(100.0);
    assert_eq!(color, colors::ACCENT_RED);
}

#[test]
fn test_card_frame_properties() {
    let frame = theme::card_frame();
    assert_eq!(frame.fill, colors::BG_CARD);
    assert_eq!(frame.stroke.width, colors::BORDER_WIDTH);
    assert_eq!(frame.stroke.color, colors::BORDER);
    assert_eq!(frame.rounding, egui::Rounding::same(colors::RADIUS));
    assert_eq!(frame.inner_margin.left, 12.0);
    assert_eq!(frame.inner_margin.right, 12.0);
    assert_eq!(frame.inner_margin.top, 12.0);
    assert_eq!(frame.inner_margin.bottom, 12.0);
}

#[test]
fn test_sidebar_active_frame() {
    let frame = theme::sidebar_active_frame();
    assert_eq!(frame.fill, colors::SIDEBAR_ACTIVE_BG);
    assert_eq!(frame.stroke.width, 2.0);
    assert_eq!(frame.stroke.color, colors::SIDEBAR_ACTIVE_BORDER);
    assert_eq!(frame.rounding, egui::Rounding::same(colors::RADIUS));
    assert_eq!(frame.inner_margin.left, 8.0);
    assert_eq!(frame.inner_margin.right, 8.0);
    assert_eq!(frame.inner_margin.top, 8.0);
    assert_eq!(frame.inner_margin.bottom, 8.0);
}

#[test]
fn test_status_bar_frame() {
    let frame = theme::status_bar_frame();
    assert_eq!(frame.fill, colors::STATUS_BAR_BG);
    assert_eq!(frame.stroke.width, 1.0);
    assert_eq!(frame.stroke.color, colors::BORDER);
    assert_eq!(frame.rounding, egui::Rounding::same(0.0));
    assert_eq!(frame.inner_margin.left, 12.0);
    assert_eq!(frame.inner_margin.right, 12.0);
    assert_eq!(frame.inner_margin.top, 8.0);
    assert_eq!(frame.inner_margin.bottom, 8.0);
}
