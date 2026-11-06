//! Status Bar for PULSE.

use crate::core::SystemSnapshot;
use crate::i18n::{os_locale_short, Key};
use crate::theme::colors;
use eframe::egui;
use std::sync::Arc;

/// Renders the bottom status bar with active process count, self-diagnostics, and OS jitter.
pub fn render_status_bar(
    ui: &mut egui::Ui,
    active_processes: u32,
    snapshot: &Option<Arc<SystemSnapshot>>,
    last_ui_ms: f32,
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
            egui::RichText::new(status_text).monospace().size(11.0).color(colors::TEXT_SECONDARY),
        );

        // PULSE self-diagnostics live here (moved from the old top-bar pill).
        if let Some(ref snap) = snapshot {
            let diag = format!(
                "PULSE: {:.1} MB | UI: {:.2} ms | OS: {}",
                snap.pulse_ram_mb,
                last_ui_ms,
                os_locale_short()
            );
            ui.separator();
            ui.add(
                egui::Label::new(
                    egui::RichText::new(diag).monospace().size(11.0).color(colors::ACCENT_CYAN),
                )
                .wrap(false),
            )
            .on_hover_text("PULSE working set, UI frame time, and real OS locale");

            ui.separator();
            let (mode_badge, mode_color) = if let Some(ref mem) = snap.memory {
                let any_heavy_process = mem.has_heavy_process;
                let total_ram_pct = mem.ram_usage_percent();
                if any_heavy_process {
                    ("[ ▲ MODE: GAMING / HEAVY APP ]", colors::ACCENT_RED)
                } else if total_ram_pct >= 75.0 {
                    ("[ ▣ MODE: WORKSTATION LOAD ]", colors::ACCENT_AMBER)
                } else {
                    ("[ ● MODE: DESKTOP IDLE ]", colors::ACCENT_CYAN)
                }
            } else {
                ("[ ● MODE: DESKTOP IDLE ]", colors::ACCENT_CYAN)
            };
            ui.label(egui::RichText::new(mode_badge).monospace().size(11.0).color(mode_color));

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
