//! PULSE main application state: UI, telemetry handling, tabs.

use std::sync::mpsc;

use eframe::egui;

use crate::core::{spawn_telemetry_worker, SystemSnapshot};
use crate::theme::{apply_obsidian_theme, card_frame};

/// Active tab in the left sidebar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveTab {
    /// Memory tab (only one for now).
    Memory,
}

impl ActiveTab {
    /// Returns all available tabs.
    pub const fn all() -> &'static [ActiveTab] {
        &[ActiveTab::Memory]
    }

    /// Button label for the tab.
    pub fn label(self) -> &'static str {
        match self {
            ActiveTab::Memory => "🧠 Memory",
        }
    }
}

/// Main application state.
pub struct PulseApp {
    /// Snapshot receiver from background thread.
    snapshot_rx: mpsc::Receiver<SystemSnapshot>,
    /// Latest received snapshot.
    latest_snapshot: Option<SystemSnapshot>,
    /// Currently active tab.
    active_tab: ActiveTab,
}

impl Default for PulseApp {
    fn default() -> Self {
        Self {
            snapshot_rx: spawn_telemetry_worker(),
            latest_snapshot: None,
            active_tab: ActiveTab::Memory,
        }
    }
}

impl eframe::App for PulseApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Apply theme
        apply_obsidian_theme(ctx);

        // Non-blocking snapshot reads from channel
        while let Ok(snapshot) = self.snapshot_rx.try_recv() {
            self.latest_snapshot = Some(snapshot);
        }

        // Top panel: title, version, uptime
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading(
                    egui::RichText::new("⚡ PULSE // TELEMETRY")
                        .monospace()
                        .size(18.0),
                );
                ui.separator();
                ui.label(
                    egui::RichText::new(format!("v{}", env!("CARGO_PKG_VERSION")))
                        .monospace()
                        .size(14.0),
                );
                ui.separator();

                if let Some(snapshot) = self.latest_snapshot {
                    let uptime_str = format_uptime(snapshot.uptime_secs);
                    ui.label(
                        egui::RichText::new(format!("Uptime: {}", uptime_str))
                            .monospace()
                            .size(14.0),
                    );
                } else {
                    ui.label(egui::RichText::new("Uptime: --").monospace().size(14.0));
                }
            });
        });

        // Left sidebar: tab navigation
        egui::SidePanel::left("sidebar")
            .resizable(false)
            .min_width(180.0)
            .max_width(180.0)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.add_space(8.0);
                    for tab in ActiveTab::all() {
                        let is_active = self.active_tab == *tab;
                        let btn = ui.selectable_label(is_active, tab.label());
                        if btn.clicked() {
                            self.active_tab = *tab;
                        }
                    }
                });
            });

        // Central panel: content based on active tab
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(16.0);
            card_frame().show(ui, |ui| {
                ui.label("Telemetry core active. Awaiting memory model (v0.0.3)...");
            });
        });

        // Request repaint for uptime updates
        ctx.request_repaint_after(std::time::Duration::from_millis(100));
    }
}

/// Formats uptime in seconds into a human-readable string (days, hours, minutes, seconds).
fn format_uptime(secs: u64) -> String {
    let days = secs / 86400;
    let hours = (secs % 86400) / 3600;
    let mins = (secs % 3600) / 60;
    let secs = secs % 60;

    let mut parts = Vec::new();
    if days > 0 {
        parts.push(format!("{}d", days));
    }
    if hours > 0 || days > 0 {
        parts.push(format!("{}h", hours));
    }
    if mins > 0 || hours > 0 || days > 0 {
        parts.push(format!("{}m", mins));
    }
    parts.push(format!("{}s", secs));

    parts.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_uptime_seconds() {
        assert_eq!(format_uptime(0), "0s");
        assert_eq!(format_uptime(30), "30s");
        assert_eq!(format_uptime(59), "59s");
    }

    #[test]
    fn test_format_uptime_minutes() {
        assert_eq!(format_uptime(60), "1m 0s");
        assert_eq!(format_uptime(90), "1m 30s");
        assert_eq!(format_uptime(3599), "59m 59s");
    }

    #[test]
    fn test_format_uptime_hours() {
        assert_eq!(format_uptime(3600), "1h 0m 0s");
        assert_eq!(format_uptime(7200), "2h 0m 0s");
        assert_eq!(format_uptime(86399), "23h 59m 59s");
    }

    #[test]
    fn test_format_uptime_days() {
        assert_eq!(format_uptime(86400), "1d 0h 0m 0s");
        assert_eq!(format_uptime(172800), "2d 0h 0m 0s");
        assert_eq!(format_uptime(90000), "1d 1h 0m 0s");
    }

    #[test]
    fn test_active_tab_all() {
        let tabs = ActiveTab::all();
        assert_eq!(tabs.len(), 1);
        assert_eq!(tabs[0], ActiveTab::Memory);
    }

    #[test]
    fn test_active_tab_label() {
        assert_eq!(ActiveTab::Memory.label(), "🧠 Memory");
    }
}
