//! Status Bar for PULSE.

use crate::core::persist;
use crate::core::SystemSnapshot;
use crate::i18n::{os_locale_short, Key};
use crate::theme::colors;
use eframe::egui;
use std::sync::{Arc, Mutex, OnceLock};

/// Medium CPU alert tier threshold: total CPU >= 85% or single-thread bottleneck.
const CPU_ALERT_TIER_PCT: f32 = 85.0;

fn warn(text: impl Into<String>, color: egui::Color32) -> egui::RichText {
    egui::RichText::new(text).monospace().size(11.0).color(color)
}

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
            "PULSE v{}  |  User-Mode Telemetry  |  Active Processes: {}",
            env!("CARGO_PKG_VERSION"),
            active_processes
        ));
    }
    Arc::clone(&cache.1)
}

/// Compact status for narrow width: drops "User-Mode Telemetry".
fn cached_status_compact(active_processes: u32) -> Arc<str> {
    static CACHE: OnceLock<Mutex<(u32, Arc<str>)>> = OnceLock::new();
    let mut cache = CACHE
        .get_or_init(|| Mutex::new((u32::MAX, Arc::from(""))))
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    if cache.0 != active_processes {
        cache.0 = active_processes;
        cache.1 = Arc::from(format!(
            "PULSE v{} | Active Processes: {}",
            env!("CARGO_PKG_VERSION"),
            active_processes
        ));
    }
    Arc::clone(&cache.1)
}

/// Cached diagnostics keyed by (RAM tenths, UI frame-time).
/// Quantization: RAM step 0.1 MB, frame-time step 10 ms (idle frames reuse the Arc).
fn cached_diag(pulse_ram_mb: f32, last_ui_ms: f32) -> Arc<str> {
    let ram = if pulse_ram_mb.is_finite() { pulse_ram_mb.max(0.0) } else { 0.0 };
    let ui_ms = if last_ui_ms.is_finite() { last_ui_ms.max(0.0) } else { 0.0 };
    type Key2 = (u32, u32);
    static CACHE: OnceLock<Mutex<(Key2, Arc<str>)>> = OnceLock::new();
    let key: Key2 = ((ram * 10.0).round().max(0.0) as u32, (ui_ms / 10.0).round().max(0.0) as u32);
    let mut cache = CACHE
        .get_or_init(|| Mutex::new(((u32::MAX, u32::MAX), Arc::from(""))))
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    if cache.0 != key {
        cache.0 = key;
        cache.1 = Arc::from(format!(
            "PULSE: {:.1} MB | UI: {:.2} ms | OS: {}",
            ram,
            ui_ms,
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusBarCacheKey {
    pub active_processes: u32,
    pub dropped_snapshots: u64,
    pub wide: bool,
}

fn receipt_tip() -> &'static Mutex<Option<String>> {
    static TIP: OnceLock<Mutex<Option<String>>> = OnceLock::new();
    TIP.get_or_init(|| Mutex::new(None))
}

/// Cached dropped-snapshot indicator text.
fn cached_drops(dropped: u64) -> Arc<str> {
    static CACHE: OnceLock<Mutex<(u64, Arc<str>)>> = OnceLock::new();
    let mut cache = CACHE
        .get_or_init(|| Mutex::new((u64::MAX, Arc::from(""))))
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    if cache.0 != dropped {
        cache.0 = dropped;
        cache.1 = if dropped > 0 { Arc::from(format!("Drops: {dropped}")) } else { Arc::from("") };
    }
    Arc::clone(&cache.1)
}

pub fn render_status_bar(
    ui: &mut egui::Ui,
    active_processes: u32,
    snapshot: &Option<Arc<SystemSnapshot>>,
    last_ui_ms: f32,
    dropped_snapshots: u64,
) {
    let bar_w = ui.available_width().max(0.0);
    let elevated = crate::platform::handle::is_app_elevated();
    let portable = persist::is_portable_mode();
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(Key::StatusReady.t())
                .monospace()
                .size(11.0)
                .color(colors::ACCENT_CYAN),
        );
        ui.separator();
        let (badge, badge_color) =
            if elevated { ("ADMIN", colors::ACCENT_CYAN) } else { ("USER", colors::ACCENT_AMBER) };
        ui.label(egui::RichText::new(badge).monospace().size(11.0).color(badge_color));
        if portable {
            ui.label(
                egui::RichText::new("[PORTABLE]").monospace().size(11.0).color(colors::ACCENT_CYAN),
            );
        }
        if !elevated
            && ui.button("[ Restart as Admin ]").clicked()
            && crate::platform::instance::restart_as_admin(portable)
        {
            crate::app_tray::set_allow_real_close(true);
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
        }
        if crate::app_tray::admin_restart_failed() {
            ui.label(warn("[ ADMIN RESTART FAILED ]", colors::ACCENT_RED));
        }
        if crate::app_tray::tray_icon_failed() {
            ui.label(warn("[ TRAY ICON FAILED ]", colors::ACCENT_AMBER))
                .on_hover_text("Tray icon could not be added after one retry");
        }
        let latest_verdict = crate::core::blackbox::latest_verdict();
        if let Some(ref verdict) = latest_verdict {
            ui.separator();
            ui.label(warn(verdict.as_str(), colors::ACCENT_AMBER))
                .on_hover_text("Latest stutter verdict (last 60 s)");
        }
        let holder = receipt_tip();
        let hint = holder
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
            .or_else(|| latest_verdict.clone())
            .unwrap_or_else(|| "Export last 60 seconds of blackbox samples".to_string());
        if ui.button("[ Lag-Receipt ]").on_hover_text(hint).clicked() {
            let msg = match crate::core::blackbox::export_lag_receipt() {
                Ok(path) => format!("Exported: {}", path.display()),
                Err(err) => format!("Export failed: {err}"),
            };
            *holder.lock().unwrap_or_else(|e| e.into_inner()) = Some(msg);
        }
        ui.separator();
        let wide = bar_w >= 960.0;
        let _cache_key = StatusBarCacheKey { active_processes, dropped_snapshots, wide };
        let status_text = if wide {
            cached_status(active_processes)
        } else {
            cached_status_compact(active_processes)
        };
        ui.label(
            egui::RichText::new(status_text.as_ref())
                .monospace()
                .size(11.0)
                .color(colors::TEXT_SECONDARY),
        );

        if persist::config_write_failed() {
            ui.separator();
            ui.label(warn("CONFIG WRITE FAILED", colors::ACCENT_RED));
        }
        if dropped_snapshots > 0 {
            ui.separator();
            ui.label(
                egui::RichText::new(cached_drops(dropped_snapshots).as_ref())
                    .monospace()
                    .size(11.0)
                    .color(colors::ACCENT_AMBER),
            )
            .on_hover_text("Snapshots dropped because the channel was full");
        }
        if let Some(ref snap) = snapshot {
            let diag = cached_diag(snap.pulse_ram_mb, last_ui_ms);
            ui.separator();
            ui.add(
                egui::Label::new(
                    egui::RichText::new(diag.as_ref())
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
                    ("[ ^ MODE: GAMING / HEAVY APP ]", colors::ACCENT_RED)
                } else if total_ram_pct >= 75.0 {
                    ("[ [o] MODE: WORKSTATION LOAD ]", colors::ACCENT_AMBER)
                } else {
                    ("[ * MODE: DESKTOP IDLE ]", colors::ACCENT_CYAN)
                }
            } else {
                ("[ * MODE: DESKTOP IDLE ]", colors::ACCENT_CYAN)
            };
            ui.label(egui::RichText::new(mode_badge).monospace().size(11.0).color(mode_color));

            if let Some(ref cpu) = snap.cpu {
                if cpu.total_pct >= CPU_ALERT_TIER_PCT || cpu.single_thread_bottleneck() {
                    ui.separator();
                    ui.label(
                        egui::RichText::new("[ CPU ALERT ]")
                            .monospace()
                            .size(11.0)
                            .color(colors::ACCENT_RED),
                    );
                }
            }

            if snap.os_scheduler_jitter_ms > 0 && wide {
                ui.separator();
                ui.label(
                    egui::RichText::new(cached_jitter(snap.os_scheduler_jitter_ms).as_ref())
                        .monospace()
                        .size(11.0)
                        .color(colors::TEXT_SECONDARY),
                );
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn status_bar_cache_key_differs_on_drops() {
        let a = StatusBarCacheKey { active_processes: 10, dropped_snapshots: 0, wide: true };
        let b = StatusBarCacheKey { active_processes: 10, dropped_snapshots: 3, wide: true };
        assert_ne!(a, b);
    }
    #[test]
    fn cached_drops_hides_when_zero() {
        assert_eq!(cached_drops(0).as_ref(), "");
        assert_eq!(cached_drops(5).as_ref(), "Drops: 5");
    }
}
