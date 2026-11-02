//! PULSE main application state: UI, telemetry handling, tabs, self-test.

use std::sync::mpsc;
use std::time::Instant;

use eframe::egui;

use crate::core::{spawn_telemetry_worker, SystemSnapshot};
use crate::i18n::Key;
use crate::modules::memory::model::MemoryHistory;
use crate::modules::memory::{chart_view, view};
use crate::theme::{apply_obsidian_theme, card_frame};
use crate::ui::chrome;

/// Active tab in the left sidebar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveTab {
    /// Memory tab.
    Memory,
    /// CPU tab.
    Cpu,
    /// Storage tab.
    Storage,
    /// Network tab.
    Network,
    /// System tab.
    System,
}

impl ActiveTab {
    /// Returns all available tabs.
    pub const fn all() -> &'static [ActiveTab] {
        &[
            ActiveTab::Memory,
            ActiveTab::Cpu,
            ActiveTab::Storage,
            ActiveTab::Network,
            ActiveTab::System,
        ]
    }

    /// Button label for the tab (localized).
    pub fn label(self) -> &'static str {
        match self {
            ActiveTab::Memory => Key::TabMemory.t(),
            ActiveTab::Cpu => Key::TabCpu.t(),
            ActiveTab::Storage => Key::TabStorage.t(),
            ActiveTab::Network => Key::TabNetwork.t(),
            ActiveTab::System => Key::TabSystem.t(),
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
    last_ui_ms: f32,
    /// Memory usage history (ring buffer).
    memory_history: MemoryHistory,
}

impl Default for PulseApp {
    fn default() -> Self {
        Self {
            snapshot_rx: spawn_telemetry_worker(),
            latest_snapshot: None,
            active_tab: ActiveTab::Memory,
            last_ui_ms: 0.0,
            memory_history: MemoryHistory::new(),
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
            // Push memory snapshot to history
            if let Some(ref mem) = snapshot.memory {
                self.memory_history.push(mem);
            }
            self.latest_snapshot = Some(snapshot);
        }

        // Clone snapshot for use in closures to avoid borrow issues
        let latest_snapshot = self.latest_snapshot.clone();

        // Extract active process count for status bar
        let active_processes = latest_snapshot
            .as_ref()
            .and_then(|s| s.memory.as_ref())
            .map(|m| m.system_processes)
            .unwrap_or(0);

        // Top panel
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            chrome::render_top_panel(ui, &latest_snapshot, self.last_ui_ms);
        });

        // Status bar
        egui::TopBottomPanel::bottom("status_bar")
            .frame(crate::theme::status_bar_frame())
            .show(ctx, |ui| {
                chrome::render_status_bar(ui, active_processes);
            });

        // Left sidebar
        egui::SidePanel::left("sidebar")
            .resizable(false)
            .exact_width(210.0)
            .show(ctx, |ui| {
                chrome::render_sidebar(ui, &mut self.active_tab, &latest_snapshot);
            });

        // Central panel: content based on active tab
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(16.0);

            match self.active_tab {
                ActiveTab::Memory => {
                    if let Some(ref snapshot) = latest_snapshot {
                        if let Some(ref mem) = snapshot.memory {
                            view::render_memory_cards(ui, mem, &self.memory_history);
                            ui.add_space(12.0);
                            // Pass mutable reference for chart window switcher
                            chart_view::render_memory_chart(ui, &mut self.memory_history);
                        } else {
                            ui.label(
                                egui::RichText::new(Key::AwaitingMemoryModel.t())
                                    .color(crate::theme::colors::TEXT_SECONDARY)
                                    .size(14.0),
                            );
                        }
                    } else {
                        ui.label(
                            egui::RichText::new(Key::AwaitingMemoryModel.t())
                                .color(crate::theme::colors::TEXT_SECONDARY)
                                .size(14.0),
                        );
                    }
                }
                ActiveTab::Cpu => {
                    card_frame().show(ui, |ui| {
                        ui.label(
                            egui::RichText::new(Key::TabCpu.t().to_owned() + " — Coming in v0.2.0")
                                .color(crate::theme::colors::TEXT_SECONDARY)
                                .size(14.0),
                        );
                    });
                }
                ActiveTab::Storage => {
                    card_frame().show(ui, |ui| {
                        ui.label(
                            egui::RichText::new(
                                Key::TabStorage.t().to_owned() + " — Coming in v0.3.0",
                            )
                            .color(crate::theme::colors::TEXT_SECONDARY)
                            .size(14.0),
                        );
                    });
                }
                ActiveTab::Network => {
                    card_frame().show(ui, |ui| {
                        ui.label(
                            egui::RichText::new(
                                Key::TabNetwork.t().to_owned() + " — Coming in v0.4.0",
                            )
                            .color(crate::theme::colors::TEXT_SECONDARY)
                            .size(14.0),
                        );
                    });
                }
                ActiveTab::System => {
                    card_frame().show(ui, |ui| {
                        ui.label(
                            egui::RichText::new(
                                Key::TabSystem.t().to_owned() + " — Coming in v0.5.0",
                            )
                            .color(crate::theme::colors::TEXT_SECONDARY)
                            .size(14.0),
                        );
                    });
                }
            }
        });

        // Store the measured UI cost of this frame (shown on the next frame)
        self.last_ui_ms = frame_start.elapsed().as_secs_f32() * 1000.0;

        // Request repaint after 1000ms (1 second interval for 1s chart updates)
        ctx.request_repaint_after(std::time::Duration::from_millis(1000));
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
            if let Some(ref snapshot) = self.latest_snapshot {
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
                    if let Some(ref snapshot) = self.latest_snapshot {
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
pub fn format_uptime(secs: u64) -> String {
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
        assert_eq!(tabs.len(), 5);
        assert_eq!(tabs[0], ActiveTab::Memory);
        assert_eq!(tabs[1], ActiveTab::Cpu);
        assert_eq!(tabs[2], ActiveTab::Storage);
        assert_eq!(tabs[3], ActiveTab::Network);
        assert_eq!(tabs[4], ActiveTab::System);
    }

    #[test]
    fn test_active_tab_label_uk() {
        set_lang(Lang::Uk);
        assert_eq!(ActiveTab::Memory.label(), "◈ Пам'ять (RAM)");
        assert_eq!(ActiveTab::Cpu.label(), "▣ Процесор (CPU)");
        assert_eq!(ActiveTab::Storage.label(), "◧ Диски (Storage)");
        assert_eq!(ActiveTab::Network.label(), "⇅ Мережа (Network)");
        assert_eq!(ActiveTab::System.label(), "⚙ Система та GPU");
    }

    #[test]
    fn test_active_tab_label_en() {
        set_lang(Lang::En);
        assert_eq!(ActiveTab::Memory.label(), "◈ Memory (RAM)");
        assert_eq!(ActiveTab::Cpu.label(), "▣ Processor (CPU)");
        assert_eq!(ActiveTab::Storage.label(), "◧ Storage (Disks)");
        assert_eq!(ActiveTab::Network.label(), "⇅ Network & Ping");
        assert_eq!(ActiveTab::System.label(), "⚙ System & GPU");
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
            "Telemetry core active. Awaiting memory model..."
        );
        assert_eq!(
            Key::SelfTestPassedHeadless.t(),
            "SELF-TEST PASSED (HEADLESS CI)"
        );
        assert_eq!(Key::SelfTestPassedLive.t(), "SELF-TEST PASSED (LIVE GUI)");
        assert_eq!(Key::SelfTestFailed.t(), "SELF-TEST FAILED");
    }

    #[test]
    fn test_memory_history_integration() {
        use crate::modules::memory::model::{MemoryHistory, MemorySnapshot};

        let mut history = MemoryHistory::new();
        let snap = MemorySnapshot {
            ram_total_bytes: 16_000_000_000,
            ram_used_bytes: 8_000_000_000,
            ram_available_bytes: 8_000_000_000,
            ram_free_bytes: 2_000_000_000,
            swap_total_bytes: 0,
            swap_used_bytes: 0,
            swap_free_bytes: 0,
            commit_total_bytes: 0,
            commit_limit_bytes: 0,
            kernel_paged_bytes: 0,
            kernel_nonpaged_bytes: 0,
            system_handles: 0,
            system_threads: 0,
            system_processes: 0,
            ram_cached_bytes_override: 0,
        };

        history.push(&snap);
        assert_eq!(history.len(), 1);
        assert!((history.peak_ram_percent - 50.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_all_tabs_present_in_sidebar() {
        let tabs = ActiveTab::all();
        assert_eq!(
            tabs.len(),
            5,
            "Should have 5 tabs: Memory, CPU, Storage, Network, System"
        );
    }
}
