//! Unit tests for the Obsidian HUD theme.

use crate::theme::{self, colors};
use crate::theme_fonts::read_font_bounded;
use std::io::Write;

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
fn test_load_color_nan_returns_secondary() {
    assert_eq!(theme::load_color(f32::NAN), colors::TEXT_SECONDARY);
    assert_eq!(theme::load_color(f32::INFINITY), colors::TEXT_SECONDARY);
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

#[test]
fn test_configure_theme_disables_selectable_labels() {
    let ctx = egui::Context::default();
    theme::apply_obsidian_theme(&ctx);
    assert!(!ctx.style().interaction.selectable_labels);
}

#[test]
fn test_read_font_bounded_rejects_missing_empty_oversized() {
    let dir = std::env::temp_dir();
    let missing = dir.join("pulse_missing_font.ttf");
    let _ = std::fs::remove_file(&missing);
    assert!(read_font_bounded(&missing).is_none());

    let empty = dir.join("pulse_empty_font.ttf");
    let _ = std::fs::write(&empty, b"");
    assert!(read_font_bounded(&empty).is_none());
    let _ = std::fs::remove_file(&empty);

    // valid small file succeeds
    let valid = dir.join("pulse_valid_font.ttf");
    let mut f = std::fs::File::create(&valid).expect("create");
    f.write_all(b"fontdata").expect("write");
    assert!(read_font_bounded(&valid).is_some());
    let _ = std::fs::remove_file(&valid);

    // oversized rejected via metadata length
    let oversized = dir.join("pulse_oversized_font.ttf");
    let f = std::fs::File::create(&oversized).expect("create");
    f.set_len(crate::theme_fonts::MAX_FONT_BYTES + 1).expect("set_len");
    drop(f);
    assert!(read_font_bounded(&oversized).is_none());
    let _ = std::fs::remove_file(&oversized);
}
