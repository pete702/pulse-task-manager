//! PULSE main application state: UI, telemetry handling, tabs, self-test.

use std::sync::mpsc;
use std::time::Instant;

use eframe::egui;

use crate::core::{spawn_telemetry_worker, SystemSnapshot};
use crate::i18n::{current_lang, toggle_lang, Key};
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

    /// Button label for the tab (localized).
    pub fn label(self) -> &'static str {
        match self {
            ActiveTab::Memory => Key::TabMemory.t(),
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
    /// Last frame render time for UI timing.
    last_frame_time: Instant,
}

impl Default for PulseApp {
    fn default() -> Self {
        Self {
            snapshot_rx: spawn_telemetry_worker(),
            latest_snapshot: None,
            active_tab: ActiveTab::Memory,
            last_frame_time: Instant::now(),
        }
    }
}

impl eframe::App for PulseApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let frame_start = Instant::now();

        // Apply theme
        apply_obsidian_theme(ctx);

        // Non-blocking snapshot reads from channel
        while let Ok(snapshot) = self.snapshot_rx.try_recv() {
            self.latest_snapshot = Some(snapshot);
        }

        // Top panel: title, version, uptime, language switch, self-diagnosis
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                // App title
                ui.heading(
                    egui::RichText::new(Key::AppTitle.t())
                        .monospace()
                        .size(18.0),
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
                    .size(14.0),
                );
                ui.separator();

                // Uptime
                if let Some(snapshot) = self.latest_snapshot {
                    let uptime_str = format_uptime(snapshot.uptime_secs);
                    ui.label(
                        egui::RichText::new(format!("{}: {}", Key::UptimeLabel.t(), uptime_str))
                            .monospace()
                            .size(14.0),
                    );
                } else {
                    ui.label(
                        egui::RichText::new(format!("{}: --", Key::UptimeLabel.t()))
                            .monospace()
                            .size(14.0),
                    );
                }
                ui.separator();

                // Self-diagnosis badge: [ PULSE: XX.X MB | UI: X.XX ms ]
                if let Some(snapshot) = self.latest_snapshot {
                    let ui_ms = frame_start
                        .duration_since(self.last_frame_time)
                        .as_secs_f32()
                        * 1000.0;
                    ui.label(
                        egui::RichText::new(format!(
                            "[ PULSE: {:.1} MB | UI: {:.2} ms ]",
                            snapshot.pulse_ram_mb, ui_ms
                        ))
                        .monospace()
                        .size(12.0)
                        .color(crate::theme::colors::BORDER),
                    );
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Language switcher button [ UA | EN ]
                    let lang = current_lang();
                    let btn_text = lang.button_label();
                    let btn = ui.button(egui::RichText::new(btn_text).monospace().size(12.0));
                    if btn.clicked() {
                        toggle_lang();
                    }
                    btn.on_hover_text(Key::LangSwitcherTooltip.t());
                });
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
                ui.label(Key::AwaitingMemoryModel.t());
            });
        });

        // Update last frame time
        self.last_frame_time = frame_start;

        // Request repaint after 500ms (economical)
        ctx.request_repaint_after(std::time::Duration::from_millis(500));
    }
}

/// Self-test application for `--self-test` mode.
pub struct SelfTestApp {
    headless: bool,
    frames_rendered: u32,
    snapshot_received: bool,
    snapshot_rx: mpsc::Receiver<SystemSnapshot>,
    latest_snapshot: Option<SystemSnapshot>,
    test_passed: bool,
}

impl SelfTestApp {
    pub fn new(headless: bool) -> Self {
        Self {
            headless,
            frames_rendered: 0,
            snapshot_received: false,
            snapshot_rx: spawn_telemetry_worker(),
            latest_snapshot: None,
            test_passed: false,
        }
    }
}

impl eframe::App for SelfTestApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        apply_obsidian_theme(ctx);

        // Read snapshots
        let mut got_snapshot = false;
        while let Ok(snapshot) = self.snapshot_rx.try_recv() {
            self.latest_snapshot = Some(snapshot);
            got_snapshot = true;
        }

        if got_snapshot {
            self.snapshot_received = true;
        }

        if self.headless {
            // Headless CI mode: just check pulse_ram_mb > 0 after first snapshot
            if let Some(snapshot) = self.latest_snapshot {
                if snapshot.pulse_ram_mb > 0.0 {
                    self.test_passed = true;
                    println!("{}", Key::SelfTestPassedHeadless.t());
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        } else {
            // Local GUI mode: wait for first snapshot, then render 5 frames, then check and close
            if self.snapshot_received {
                self.frames_rendered += 1;
                if self.frames_rendered >= 5 {
                    if let Some(snapshot) = self.latest_snapshot {
                        if snapshot.pulse_ram_mb > 0.0 {
                            self.test_passed = true;
                            println!("{}", Key::SelfTestPassedLive.t());
                        } else {
                            eprintln!("{}", Key::SelfTestFailed.t());
                        }
                    } else {
                        eprintln!("{}", Key::SelfTestFailed.t());
                    }
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            } else {
                // Still waiting for first snapshot, request repaint to keep checking
                ctx.request_repaint_after(std::time::Duration::from_millis(100));
            }
        }
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
    use crate::i18n::{set_lang, Lang};

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
    fn test_active_tab_label_uk() {
        set_lang(Lang::Uk);
        assert_eq!(ActiveTab::Memory.label(), "🧠 Пам'ять");
    }

    #[test]
    fn test_active_tab_label_en() {
        set_lang(Lang::En);
        assert_eq!(ActiveTab::Memory.label(), "🧠 Memory");
    }

    #[test]
    fn test_i18n_keys() {
        set_lang(Lang::En);
        assert_eq!(Key::AppTitle.t(), "⚡ PULSE // TELEMETRY");
        assert_eq!(Key::VersionPrefix.t(), "v");
        assert_eq!(Key::UptimeLabel.t(), "Uptime");
        assert_eq!(Key::SelfDiagLabel.t(), "Self-Diag");
        assert_eq!(
            Key::AwaitingMemoryModel.t(),
            "Telemetry core active. Awaiting memory model (v0.0.4)..."
        );
        assert_eq!(
            Key::SelfTestPassedHeadless.t(),
            "SELF-TEST PASSED (HEADLESS CI)"
        );
        assert_eq!(Key::SelfTestPassedLive.t(), "SELF-TEST PASSED (LIVE GUI)");
        assert_eq!(Key::SelfTestFailed.t(), "SELF-TEST FAILED");
    }
}
