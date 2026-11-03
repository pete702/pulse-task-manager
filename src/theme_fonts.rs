//! Font setup for PULSE Obsidian HUD theme.

use eframe::egui;

/// Loads system fonts: Segoe UI (proportional) and Consolas (monospace) from Windows Fonts.
/// Falls back to egui default fonts if files are not found.
/// Uses only geometric Unicode symbols (◈, ▣, ⚡, ▲, ▼, ●) — no emoji fonts needed.
pub fn setup_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    // Segoe UI for proportional text
    let segoe_path = r"C:\Windows\Fonts\segoeui.ttf";
    // Consolas for monospace (numbers, PID, etc.)
    let consola_path = r"C:\Windows\Fonts\consola.ttf";

    let mut proportional_loaded = false;
    let mut monospace_loaded = false;

    if let Ok(data) = std::fs::read(segoe_path) {
        fonts.font_data.insert("segoe_ui".to_owned(), egui::FontData::from_owned(data));
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "segoe_ui".to_owned());
        proportional_loaded = true;
    }

    if let Ok(data) = std::fs::read(consola_path) {
        fonts.font_data.insert("consolas".to_owned(), egui::FontData::from_owned(data));
        fonts
            .families
            .entry(egui::FontFamily::Monospace)
            .or_default()
            .insert(0, "consolas".to_owned());
        monospace_loaded = true;
    }

    // If either failed to load, ensure defaults are present
    if !proportional_loaded {
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .push("default".to_owned());
    }
    if !monospace_loaded {
        fonts.families.entry(egui::FontFamily::Monospace).or_default().push("default".to_owned());
    }

    ctx.set_fonts(fonts);
}
