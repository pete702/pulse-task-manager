//! Font setup for PULSE Obsidian HUD theme.

use eframe::egui;
use std::path::Path;

/// Loads system fonts: Segoe UI (proportional) and Consolas (monospace) from Windows Fonts.
/// Falls back to egui default fonts if files are not found.
/// Uses only geometric Unicode symbols -- no emoji fonts needed.
/// Maximum bytes allowed for a single font file (16 MB).
pub const MAX_FONT_BYTES: u64 = 16_000_000;

/// Reads a font file only when its metadata reports a non-empty regular file within
/// `MAX_FONT_BYTES`. Oversized/empty/missing files are rejected before reading.
pub(crate) fn read_font_bounded(path: &Path) -> Option<Vec<u8>> {
    let meta = std::fs::metadata(path).ok()?;
    if !meta.is_file() || meta.len() == 0 || meta.len() > MAX_FONT_BYTES {
        return None;
    }
    let bytes = std::fs::read(path).ok()?;
    if bytes.len() as u64 > MAX_FONT_BYTES {
        return None;
    }
    Some(bytes)
}

pub fn setup_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    // Segoe UI for proportional text
    let segoe_path = r"C:\Windows\Fonts\segoeui.ttf";
    // Consolas for monospace (numbers, PID, etc.)
    let consola_path = r"C:\Windows\Fonts\consola.ttf";

    if let Some(data) = read_font_bounded(std::path::Path::new(segoe_path)) {
        fonts.font_data.insert("segoe_ui".to_owned(), egui::FontData::from_owned(data));
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "segoe_ui".to_owned());
    }

    if let Some(data) = read_font_bounded(std::path::Path::new(consola_path)) {
        fonts.font_data.insert("consolas".to_owned(), egui::FontData::from_owned(data));
        fonts
            .families
            .entry(egui::FontFamily::Monospace)
            .or_default()
            .insert(0, "consolas".to_owned());
    }

    ctx.set_fonts(fonts);
}
