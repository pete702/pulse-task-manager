//! PULSE main application state: UI, telemetry handling, tabs, self-test.

use std::sync::mpsc;
use std::time::Instant;

use eframe::egui;

use crate::core::{spawn_telemetry_worker, SystemSnapshot};
use crate::i18n::{current_lang, toggle_lang, Key};
use crate::modules::memory::model::MemoryHistory;
use crate::modules::memory::{chart_view, view};
use crate::theme::{apply_obsidian_theme, card_frame};

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
    last_frame_time: Instant,
    /// Memory usage history (ring buffer).
    memory_history: MemoryHistory,
}

impl Default for PulseApp {
    fn default() -> Self {
        Self {
            snapshot_rx: spawn_telemetry_worker(),
            latest_snapshot: None,
            active_tab: ActiveTab::Memory,
            last_frame_time: Instant::now(),
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
        let frame_start_for_ui = frame_start;

        // Top panel: title, version, uptime, language switch, self-diagnosis
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                // App title
                ui.heading(
                    egui::RichText::new(Key::AppTitle.t())
                        .monospace()
                        .size(18.0)
                        .color(crate::theme::colors::TEXT_PRIMARY),
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
                    .color(crate::theme::colors::TEXT_SECONDARY),
                );
                ui.separator();

                // Uptime
                if let Some(ref snapshot) = latest_snapshot {
                    let uptime_str = format_uptime(snapshot.uptime_secs);
                    ui.label(
                        egui::RichText::new(format!("{}: {}", Key::UptimeLabel.t(), uptime_str))
                            .monospace()
                            .size(14.0)
                            .color(crate::theme::colors::TEXT_PRIMARY),
                    );
                } else {
                    ui.label(
                        egui::RichText::new(format!("{}: --", Key::UptimeLabel.t()))
                            .monospace()
                            .size(14.0)
                            .color(crate::theme::colors::TEXT_SECONDARY),
                    );
                }
                ui.separator();

                // Self-diagnosis badge: [ PULSE: XX.X MB | UI: X.XX ms ]
                if let Some(ref snapshot) = latest_snapshot {
                    let ui_ms = frame_start_for_ui
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
                        .color(crate::theme::colors::ACCENT_CYAN),
                    );
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Language switcher button [ UA | EN ]
                    let lang = current_lang();
                    let btn_text = lang.button_label();
                    let btn = ui.button(
                        egui::RichText::new(btn_text)
                            .monospace()
                            .size(12.0)
                            .color(crate::theme::colors::TEXT_PRIMARY),
                    );
                    if btn.clicked() {
                        toggle_lang();
                    }
                    btn.on_hover_text(Key::LangSwitcherTooltip.t());
                });
            });
        });

        // Status bar (bottom)
        egui::TopBottomPanel::bottom("status_bar")
            .frame(crate::theme::status_bar_frame())
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("● READY")
                            .monospace()
                            .size(11.0)
                            .color(crate::theme::colors::ACCENT_CYAN),
                    );
                    ui.separator();
                    ui.label(
                        egui::RichText::new("PULSE v0.0.6  |  Obsidian HUD  |  60s History  |  Windows Kernel Metrics")
                            .monospace()
                            .size(11.0)
                            .color(crate::theme::colors::TEXT_SECONDARY),
                    );
                });
            });

        // Left sidebar: tab navigation (exact width 210.0)
        egui::SidePanel::left("sidebar")
            .resizable(false)
            .exact_width(210.0)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.add_space(8.0);
                    for tab in ActiveTab::all() {
                        let is_active = self.active_tab == *tab;
                        let btn_text = tab.label();

                        // Add live RAM percentage to Memory tab button
                        let display_text = if *tab == ActiveTab::Memory {
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
                                        .color(crate::theme::colors::TEXT_PRIMARY),
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
                                        .color(crate::theme::colors::TEXT_SECONDARY),
                                ),
                            )
                        };

                        if button.clicked() {
                            self.active_tab = *tab;
                        }

                        // Apply custom frame for active tab
                        if is_active {
                            button.widget_info(|| {
                                egui::WidgetInfo::labeled(egui::WidgetType::Button, &display_text)
                            });
                        }
                    }
                });
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
                            chart_view::render_memory_chart(ui, &self.memory_history);
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
                            egui::RichText::new("▣ Процесор (CPU) — Coming in v0.2.0")
                                .color(crate::theme::colors::TEXT_SECONDARY)
                                .size(14.0),
                        );
                    });
                }
                ActiveTab::Storage => {
                    card_frame().show(ui, |ui| {
                        ui.label(
                            egui::RichText::new("◧ Диски (Storage) — Coming in v0.3.0")
                                .color(crate::theme::colors::TEXT_SECONDARY)
                                .size(14.0),
                        );
                    });
                }
                ActiveTab::Network => {
                    card_frame().show(ui, |ui| {
                        ui.label(
                            egui::RichText::new("⇅ Мережа (Network) — Coming in v0.4.0")
                                .color(crate::theme::colors::TEXT_SECONDARY)
                                .size(14.0),
                        );
                    });
                }
                ActiveTab::System => {
                    card_frame().show(ui, |ui| {
                        ui.label(
                            egui::RichText::new("⚙ Система та GPU — Coming in v0.5.0")
                                .color(crate::theme::colors::TEXT_SECONDARY)
                                .size(14.0),
                        );
                    });
                }
            }
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
            "Telemetry core active. Awaiting memory model (v0.0.4)..."
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
