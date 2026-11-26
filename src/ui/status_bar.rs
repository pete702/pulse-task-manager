//! Bottom status bar for PULSE, built from Obsidian Cyan rounded pills.

use crate::core::persist;
use crate::core::SystemSnapshot;
use crate::i18n::{os_locale_short, Key};
use crate::theme::colors;
use crate::ui::primitives::{pill_badge_tight, pill_button};
use eframe::egui;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex, OnceLock};

const CPU_ALERT_TIER_PCT: f32 = 85.0;

type TextCache = (HashMap<(u8, u64), Arc<str>>, VecDeque<(u8, u64)>);
static TEXT_CACHE: OnceLock<Mutex<TextCache>> = OnceLock::new();

/// Bounded string cache: idle frames reuse the previous formatted line.
fn cached_text(kind: u8, key: u64, make: impl FnOnce() -> String) -> Arc<str> {
    let k = (kind, key);
    {
        let cache = TEXT_CACHE.get_or_init(|| Mutex::new((HashMap::new(), VecDeque::new())));
        let guard = cache.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(value) = guard.0.get(&k) {
            return Arc::clone(value);
        }
    }
    let value: Arc<str> = Arc::from(make());
    let cache = TEXT_CACHE.get_or_init(|| Mutex::new((HashMap::new(), VecDeque::new())));
    let mut guard = cache.lock().unwrap_or_else(|e| e.into_inner());
    if guard.0.len() >= 128 {
        if let Some(old) = guard.1.pop_front() {
            guard.0.remove(&old);
        }
    }
    guard.0.insert(k, Arc::clone(&value));
    guard.1.push_back(k);
    value
}

fn cached_status(active_processes: u32) -> Arc<str> {
    cached_text(1, active_processes as u64, || {
        format!(
            "PULSE v{}  |  User-Mode Telemetry  |  Active Processes: {}",
            env!("CARGO_PKG_VERSION"),
            active_processes
        )
    })
}

fn cached_diag(pulse_ram_mb: f32, last_ui_ms: f32) -> Arc<str> {
    let ram = if pulse_ram_mb.is_finite() { pulse_ram_mb.max(0.0) } else { 0.0 };
    let ui_ms = if last_ui_ms.is_finite() { last_ui_ms.max(0.0) } else { 0.0 };
    let key =
        (ram * 10.0).round().max(0.0) as u64 * 100_000 + (ui_ms / 10.0).round().max(0.0) as u64;
    cached_text(3, key, || {
        format!("PULSE: {ram:.1} MB | UI: {ui_ms:.2} ms | OS: {}", os_locale_short())
    })
}

fn cached_jitter(jitter_ms: u32) -> Arc<str> {
    cached_text(4, jitter_ms as u64, || format!("OS Jitter: {jitter_ms} ms"))
}

fn cached_drops(dropped: u64) -> Arc<str> {
    cached_text(
        5,
        dropped,
        || {
            if dropped > 0 {
                format!("Drops: {dropped}")
            } else {
                String::new()
            }
        },
    )
}

/// Numeric pill uses JetBrains Mono; text pills stay in Inter.
fn pill_number(ui: &mut egui::Ui, text: &str, fg: egui::Color32) -> egui::Response {
    egui::Frame::none()
        .fill(fg.gamma_multiply(0.13))
        .stroke(egui::Stroke::NONE)
        .rounding(egui::Rounding::same(5.0))
        .inner_margin(egui::Margin::symmetric(5.0, 1.0))
        .show(ui, |ui| {
            ui.add(
                egui::Label::new(
                    egui::RichText::new(text).font(egui::FontId::monospace(10.5)).color(fg),
                )
                .wrap(false),
            );
        })
        .response
}

fn receipt_tip() -> &'static Mutex<Option<String>> {
    static TIP: OnceLock<Mutex<Option<String>>> = OnceLock::new();
    TIP.get_or_init(|| Mutex::new(None))
}

/// Renders the bottom status bar as a single row of rounded information pills.
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
    ui.spacing_mut().item_spacing = egui::vec2(4.0, 0.0);
    ui.set_height(28.0);
    ui.horizontal(|ui| {
        pill_badge_tight(ui, Key::StatusReady.t(), colors::ACCENT_CYAN);
        let (badge, badge_color) =
            if elevated { ("ADMIN", colors::ACCENT_CYAN) } else { ("USER", colors::ACCENT_AMBER) };
        pill_badge_tight(ui, badge, badge_color);
        if portable {
            pill_badge_tight(ui, "PORTABLE", colors::ACCENT_CYAN);
        }
        if !elevated
            && pill_button(ui, "Restart as Admin").clicked()
            && crate::platform::instance::restart_as_admin(portable)
        {
            crate::app_tray::set_allow_real_close(true);
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
        }
        if crate::app_tray::admin_restart_failed() {
            pill_badge_tight(ui, "ADMIN RESTART FAILED", colors::ACCENT_RED);
        }
        if crate::app_tray::tray_icon_failed() {
            pill_badge_tight(ui, "TRAY ICON FAILED", colors::ACCENT_AMBER)
                .on_hover_text("Tray icon could not be added after one retry");
        }
        let latest_verdict = crate::core::blackbox::latest_verdict();
        if let Some(ref verdict) = latest_verdict {
            pill_badge_tight(ui, verdict.as_str(), colors::ACCENT_AMBER)
                .on_hover_text("Latest stutter verdict (last 60 s)");
        }
        let holder = receipt_tip();
        let hint = holder
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
            .or_else(|| latest_verdict.clone())
            .unwrap_or_else(|| "Export last 60 seconds of blackbox samples".to_string());
        if pill_button(ui, "Lag-Receipt").on_hover_text(hint).clicked() {
            let msg = match crate::core::blackbox::export_lag_receipt() {
                Ok(path) => format!("Exported: {}", path.display()),
                Err(err) => format!("Export failed: {err}"),
            };
            *holder.lock().unwrap_or_else(|e| e.into_inner()) = Some(msg);
        }
        let wide = bar_w >= 1480.0;
        if wide {
            pill_badge_tight(ui, cached_status(active_processes).as_ref(), colors::TEXT_SECONDARY);
        }
        if persist::config_write_failed() {
            pill_badge_tight(ui, "CONFIG WRITE FAILED", colors::ACCENT_RED);
        }
        if dropped_snapshots > 0 {
            pill_number(ui, cached_drops(dropped_snapshots).as_ref(), colors::ACCENT_AMBER)
                .on_hover_text("Snapshots dropped because the channel was full");
        }
        if let Some(ref snap) = snapshot {
            let diag = cached_diag(snap.pulse_ram_mb, last_ui_ms);
            pill_number(ui, diag.as_ref(), colors::ACCENT_CYAN)
                .on_hover_text("PULSE working set, UI frame time, and real OS locale");
            let (mode_badge, mode_color) = if let Some(ref mem) = snap.memory {
                if mem.has_heavy_process {
                    ("MODE: GAMING / HEAVY APP", colors::ACCENT_RED)
                } else if mem.ram_usage_percent() >= 75.0 {
                    ("MODE: WORKSTATION LOAD", colors::ACCENT_AMBER)
                } else {
                    ("MODE: DESKTOP IDLE", colors::ACCENT_CYAN)
                }
            } else {
                ("MODE: DESKTOP IDLE", colors::ACCENT_CYAN)
            };
            pill_badge_tight(ui, mode_badge, mode_color);
            if let Some(ref cpu) = snap.cpu {
                if cpu.total_pct >= CPU_ALERT_TIER_PCT || cpu.single_thread_bottleneck() {
                    pill_badge_tight(ui, "CPU ALERT", colors::ACCENT_RED);
                }
            }
            if snap.os_scheduler_jitter_ms > 0 {
                pill_number(
                    ui,
                    cached_jitter(snap.os_scheduler_jitter_ms).as_ref(),
                    colors::TEXT_SECONDARY,
                );
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cached_drops_hides_when_zero() {
        assert_eq!(cached_drops(0).as_ref(), "");
        assert_eq!(cached_drops(5).as_ref(), "Drops: 5");
    }

    #[test]
    fn cached_text_reuses_identical_entries() {
        let a = cached_text(9, 42, || "value".to_owned());
        let b = cached_text(9, 42, || "other".to_owned());
        assert_eq!(&*a, "value");
        assert_eq!(&*b, "value");
    }
}
