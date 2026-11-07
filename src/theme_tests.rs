//! Unit tests for the Obsidian HUD theme.

use crate::theme::{self, colors};

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
