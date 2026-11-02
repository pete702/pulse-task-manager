//! UI Chrome: Top bar, sidebar, status bar rendering for PULSE.

use eframe::egui;

use crate::app::format_uptime;
use crate::core::SystemSnapshot;
use crate::i18n::{current_lang, Key};
use crate::theme::colors;

/// Renders the top panel with title, version, uptime, and locale indicator.
pub fn render_top_panel(
    ui: &mut egui::Ui,
    latest_snapshot: &Option<SystemSnapshot>,
    frame_start: std::time::Instant,
    last_frame_time: std::time::Instant,
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
            let ui_ms = frame_start.duration_since(last_frame_time).as_secs_f32() * 1000.0;
            ui.label(
                egui::RichText::new(format!(
                    "[ PULSE: {:.1} MB | UI: {:.2} ms ]",
                    snapshot.pulse_ram_mb, ui_ms
                ))
                .monospace()
                .size(12.0)
                .color(colors::ACCENT_CYAN),
            );
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // Locale indicator (auto-detected from Windows)
            let lang = current_lang();
            let locale_str = lang.locale_string();
            ui.label(
                egui::RichText::new(format!("[ LOCALE: {} ]", locale_str))
                    .monospace()
                    .size(12.0)
                    .color(colors::ACCENT_CYAN),
            );
        });
    });
}

/// Renders the bottom status bar.
pub fn render_status_bar(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(Key::StatusReady.t())
                .monospace()
                .size(11.0)
                .color(colors::ACCENT_CYAN),
        );
        ui.separator();
        ui.label(
            egui::RichText::new(Key::StatusInfo.t())
                .monospace()
                .size(11.0)
                .color(colors::TEXT_SECONDARY),
        );
    });
}

/// Renders the left sidebar with tab navigation.
pub fn render_sidebar(
    ui: &mut egui::Ui,
    active_tab: &mut crate::app::ActiveTab,
    latest_snapshot: &Option<SystemSnapshot>,
) {
    ui.vertical(|ui| {
        ui.add_space(8.0);
        for tab in crate::app::ActiveTab::all() {
            let is_active = *active_tab == *tab;
            let btn_text = tab.label();

            // Add live RAM percentage to Memory tab button
            let display_text = if *tab == crate::app::ActiveTab::Memory {
                if let Some(ref snapshot) = latest_snapshot {
                    if let Some(ref mem) = snapshot.memory {
                        format!("{} [{:.0}%]", btn_text, mem.ram_usage_percent())
                    } else {
                        btn_text.to_string()
                    }
                } else {
                    btn_text.to_string()
                }
            } else {
                btn_text.to_string()
            };

            let button = if is_active {
                ui.add_sized(
                    [186.0, 38.0],
                    egui::Button::new(
                        egui::RichText::new(&display_text)
                            .monospace()
                            .size(13.0)
                            .color(colors::TEXT_PRIMARY),
                    )
                    .frame(true),
                )
            } else {
                ui.add_sized(
                    [186.0, 38.0],
                    egui::Button::new(
                        egui::RichText::new(&display_text)
                            .monospace()
                            .size(13.0)
                            .color(colors::TEXT_SECONDARY),
                    ),
                )
            };

            if button.clicked() {
                *active_tab = *tab;
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::{set_lang, Lang};

    #[test]
    fn test_locale_strings() {
        set_lang(Lang::Uk);
        assert_eq!(current_lang(), Lang::Uk);
        set_lang(Lang::En);
        assert_eq!(current_lang(), Lang::En);
    }
}
