//! Status Bar for PULSE.

use crate::core::SystemSnapshot;
use crate::i18n::{os_locale_short, Key};
use crate::theme::colors;
use eframe::egui;
use std::sync::{Arc, Mutex, OnceLock};

/// Cached status line keyed by process count: zero format! on idle frames.
fn cached_status(active_processes: u32) -> Arc<str> {
    static CACHE: OnceLock<Mutex<(u32, Arc<str>)>> = OnceLock::new();
    let mut cache = CACHE
        .get_or_init(|| Mutex::new((u32::MAX, Arc::from(""))))
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    if cache.0 != active_processes {
        cache.0 = active_processes;
        cache.1 = Arc::from(format!(
            "PULSE v{}  |  Windows Kernel Monitor  |  Active Processes: {}",
            env!("CARGO_PKG_VERSION"),
            active_processes
        ));
    }
    Arc::clone(&cache.1)
}

/// Cached diagnostics keyed by (RAM tenths, UI-hundredths): idle frames reuse the Arc.
fn cached_diag(pulse_ram_mb: f32, last_ui_ms: f32) -> Arc<str> {
    type Key2 = (u32, u32);
    static CACHE: OnceLock<Mutex<(Key2, Arc<str>)>> = OnceLock::new();
    let key: Key2 = (
        (pulse_ram_mb * 10.0).round().max(0.0) as u32,
        (last_ui_ms * 100.0).round().max(0.0) as u32,
    );
    let mut cache = CACHE
        .get_or_init(|| Mutex::new(((u32::MAX, u32::MAX), Arc::from(""))))
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    if cache.0 != key {
        cache.0 = key;
        cache.1 = Arc::from(format!(
            "PULSE: {:.1} MB | UI: {:.2} ms | OS: {}",
            pulse_ram_mb,
            last_ui_ms,
            os_locale_short()
        ));
    }
    Arc::clone(&cache.1)
}

/// Cached jitter line keyed by jitter ms.
fn cached_jitter(jitter_ms: u32) -> Arc<str> {
    static CACHE: OnceLock<Mutex<(u32, Arc<str>)>> = OnceLock::new();
    let mut cache = CACHE
        .get_or_init(|| Mutex::new((u32::MAX, Arc::from(""))))
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    if cache.0 != jitter_ms {
        cache.0 = jitter_ms;
        cache.1 = Arc::from(format!("OS Jitter: {jitter_ms} ms"));
    }
    Arc::clone(&cache.1)
}

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

        let status_text = cached_status(active_processes);
        ui.label(
            egui::RichText::new(status_text.as_ref().to_owned())
                .monospace()
                .size(11.0)
                .color(colors::TEXT_SECONDARY),
        );

        // PULSE self-diagnostics live here (moved from the old top-bar pill).
        if let Some(ref snap) = snapshot {
            let diag = cached_diag(snap.pulse_ram_mb, last_ui_ms);
            ui.separator();
            ui.add(
                egui::Label::new(
                    egui::RichText::new(diag.as_ref().to_owned())
                        .monospace()
                        .size(11.0)
                        .color(colors::ACCENT_CYAN),
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
                    egui::RichText::new(
                        cached_jitter(snap.os_scheduler_jitter_ms).as_ref().to_owned(),
                    )
                    .monospace()
                    .size(11.0)
                    .color(colors::TEXT_SECONDARY),
                );
            }
        }
    });
}
