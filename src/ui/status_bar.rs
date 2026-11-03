//! Status Bar for PULSE.

use crate::core::SystemSnapshot;
use crate::i18n::Key;
use crate::theme::colors;
use eframe::egui;

/// Renders the bottom status bar with active process count, heuristic mode, and OS jitter.
pub fn render_status_bar(
    ui: &mut egui::Ui,
    active_processes: u32,
    snapshot: &Option<SystemSnapshot>,
) {
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(Key::StatusReady.t())
                .monospace()
                .size(11.0)
                .color(colors::ACCENT_CYAN),
        );
        ui.separator();

        let status_text = format!(
            "PULSE v{}  |  Windows Kernel Monitor  |  Active Processes: {}",
            env!("CARGO_PKG_VERSION"),
            active_processes
        );
        ui.label(
            egui::RichText::new(status_text)
                .monospace()
                .size(11.0)
                .color(colors::TEXT_SECONDARY),
        );

        // Heuristic mode badge & OS jitter (from latest snapshot)
        if let Some(ref snap) = snapshot {
            ui.separator();

            // Heuristic mode badge
            let (mode_badge, mode_color) = if let Some(ref mem) = snap.memory {
                let any_heavy_process = mem
                    .processes
                    .iter()
                    .any(|(_, _, ram, _)| *ram >= (2560_u64 << 20)); // 2.5 GiB
                let total_ram_pct = mem.ram_usage_percent();
                if any_heavy_process {
                    ("[ ◈ MODE: GAMING / HEAVY APP ]", colors::ACCENT_RED)
                } else if total_ram_pct >= 75.0 {
                    ("[ ▣ MODE: WORKSTATION LOAD ]", colors::ACCENT_AMBER)
                } else {
                    ("[ ● MODE: DESKTOP IDLE ]", colors::ACCENT_CYAN)
                }
            } else {
                ("[ ● MODE: DESKTOP IDLE ]", colors::ACCENT_CYAN)
            };
            ui.label(
                egui::RichText::new(mode_badge)
                    .monospace()
                    .size(11.0)
                    .color(mode_color),
            );

            // OS Scheduler Jitter
            if snap.os_scheduler_jitter_ms > 0 {
                ui.separator();
                ui.label(
                    egui::RichText::new(format!("OS Jitter: {} ms", snap.os_scheduler_jitter_ms))
                        .monospace()
                        .size(11.0)
                        .color(colors::TEXT_SECONDARY),
                );
            }
        }
    });
}
