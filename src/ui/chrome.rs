//! UI Chrome: Top bar for PULSE.
use crate::core::SystemSnapshot;
use crate::i18n::{os_locale_badge, Key};
use crate::theme::colors;
use crate::util::format_uptime;
use eframe::egui;
use std::sync::{Arc, Mutex, OnceLock};
// Re-export for compatibility with app.rs
pub use crate::ui::placeholder_tab::render_placeholder_tab;
pub use crate::ui::status_bar::render_status_bar;

/// Cached uptime formatter: allocates only when uptime_secs changes (1 Hz).
fn cached_format_uptime(uptime_secs: u64) -> String {
    static CACHE: OnceLock<Mutex<(u64, String)>> = OnceLock::new();
    let mut cache = CACHE.get_or_init(|| Mutex::new((0, String::new()))).lock().unwrap();
    if cache.0 != uptime_secs {
        cache.0 = uptime_secs;
        cache.1 = format_uptime(uptime_secs);
    }
    cache.1.clone()
}

/// Renders the top panel with title, version, uptime, and locale indicator.
pub fn render_top_panel(
    ui: &mut egui::Ui,
    latest_snapshot: &Option<Arc<SystemSnapshot>>,
    last_ui_ms: f32,
) {
    ui.horizontal(|ui| {
        // App title
        ui.heading(
            egui::RichText::new(Key::AppTitle.t())
                .monospace()
                .size(18.0)
                .color(colors::TEXT_PRIMARY),
        );
        ui.separator();

        // Version
        ui.label(
            egui::RichText::new(format!("{}{}", Key::VersionPrefix.t(), env!("CARGO_PKG_VERSION")))
                .monospace()
                .size(14.0)
                .color(colors::TEXT_SECONDARY),
        );
        ui.separator();

        // Uptime (cached at 1 Hz)
        if let Some(ref snapshot) = latest_snapshot {
            let uptime_str = cached_format_uptime(snapshot.uptime_secs);
            ui.label(
                egui::RichText::new(format!("{}: {}", Key::UptimeLabel.t(), uptime_str))
                    .monospace()
                    .size(14.0)
                    .color(colors::TEXT_PRIMARY),
            );
        } else {
            ui.label(
                egui::RichText::new(format!("{}: --", Key::UptimeLabel.t()))
                    .monospace()
                    .size(14.0)
                    .color(colors::TEXT_SECONDARY),
            );
        }
        ui.separator();

        // Self-diagnosis badge: [ PULSE: XX.X MB | UI: X.XX ms ]
        if let Some(ref snapshot) = latest_snapshot {
            ui.label(
                egui::RichText::new(format!(
                    "[ PULSE: {:.1} MB | UI: {:.2} ms ]",
                    snapshot.pulse_ram_mb, last_ui_ms
                ))
                .monospace()
                .size(12.0)
                .color(colors::ACCENT_CYAN),
            );
        }

        // Three Walls pressure badge (RAM / commit charge)
        if let Some(ref snapshot) = latest_snapshot {
            ui.separator();
            crate::ui::three_walls::render_badge(ui, snapshot);
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // Locale indicator (auto-detected from Windows)
            let locale_str = os_locale_badge();
            ui.label(
                egui::RichText::new(format!("[ {} ]", locale_str))
                    .monospace()
                    .size(12.0)
                    .color(colors::ACCENT_CYAN),
            );
        });
    });
}

#[cfg(test)]
#[path = "chrome_tests.rs"]
mod chrome_tests;
