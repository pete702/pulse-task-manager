//! UI Chrome: Top bar for PULSE.
use crate::core::SystemSnapshot;
use crate::i18n::{os_locale_badge, Key};
use crate::theme::colors;
use crate::util::format_uptime;
use eframe::egui;
// Re-export for compatibility with app.rs
pub use crate::ui::placeholder_tab::render_placeholder_tab;
pub use crate::ui::status_bar::render_status_bar;

/// Renders the top panel with title, version, uptime, and locale indicator.
pub fn render_top_panel(
    ui: &mut egui::Ui,
    latest_snapshot: &Option<SystemSnapshot>,
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
            egui::RichText::new(format!(
                "{}{}",
                Key::VersionPrefix.t(),
                env!("CARGO_PKG_VERSION")
            ))
            .monospace()
            .size(14.0)
            .color(colors::TEXT_SECONDARY),
        );
        ui.separator();

        // Uptime
        if let Some(ref snapshot) = latest_snapshot {
            let uptime_str = format_uptime(snapshot.uptime_secs);
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
