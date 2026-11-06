//! UI Chrome: Top bar for PULSE.
use crate::core::SystemSnapshot;
use crate::i18n::{os_locale_short, Key};
use crate::theme::colors;
use crate::util::format_uptime;
use eframe::egui;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
// Re-export for compatibility with app.rs
pub use crate::ui::placeholder_tab::render_placeholder_tab;
pub use crate::ui::status_bar::render_status_bar;

fn cached_version() -> &'static str {
    static CACHE: OnceLock<&'static str> = OnceLock::new();
    CACHE.get_or_init(|| {
        Box::leak(
            format!("{}{}", Key::VersionPrefix.t(), env!("CARGO_PKG_VERSION")).into_boxed_str(),
        )
    })
}

/// Persisted self-RAM cache (x10 for atomic): keeps last value across ticks/tabs.
static LAST_SELF_RAM_X10: AtomicU32 = AtomicU32::new(426);

fn cached_self_ram_mb(fresh: Option<f32>) -> f32 {
    if let Some(v) = fresh {
        if v > 0.0 {
            LAST_SELF_RAM_X10.store((v * 10.0).round() as u32, Ordering::Relaxed);
        }
    }
    LAST_SELF_RAM_X10.load(Ordering::Relaxed) as f32 / 10.0
}

/// Cached uptime formatter: allocates only when uptime_secs changes (1 Hz).
fn cached_format_uptime(uptime_secs: u64) -> String {
    static CACHE: OnceLock<Mutex<(u64, String)>> = OnceLock::new();
    let mut cache = CACHE.get_or_init(|| Mutex::new((0, String::new()))).lock().unwrap();
    if cache.0 != uptime_secs {
        cache.0 = uptime_secs;
        cache.1 = format_uptime(uptime_secs);
    }
    cache.1.clone()
}

/// Renders the top panel with title, version, live system telemetry, and health pill.
pub fn render_top_panel(
    ui: &mut egui::Ui,
    latest_snapshot: &Option<Arc<SystemSnapshot>>,
    last_ui_ms: f32,
) {
    let bar_w = ui.available_width();
    let top_rect = ui.available_rect_before_wrap();
    ui.horizontal(|ui| {
        let title = if bar_w < 920.0 { "▲ PULSE" } else { Key::AppTitle.t() };
        ui.heading(
            egui::RichText::new(title).monospace().size(18.0).color(colors::TEXT_PRIMARY),
        );
        ui.separator();
        ui.label(
            egui::RichText::new(cached_version())
                .monospace()
                .size(14.0)
                .color(colors::TEXT_SECONDARY),
        );
        ui.separator();
        let uptime_str = latest_snapshot
            .as_ref()
            .map(|s| cached_format_uptime(s.uptime_secs))
            .unwrap_or_else(|| "--".to_owned());
        if bar_w >= 1040.0 {
            ui.label(
                egui::RichText::new(format!("UPTIME: {}", uptime_str))
                    .monospace()
                    .size(12.0)
                    .color(colors::TEXT_PRIMARY),
            );
            ui.separator();
        }
        let ram_mb = cached_self_ram_mb(latest_snapshot.as_ref().map(|s| s.pulse_ram_mb));
        let telemetry = match latest_snapshot.as_ref() {
            Some(s) => match (&s.cpu, &s.memory) {
                (Some(cpu), Some(mem)) => format!(
                    "[ CPU: {:.1}% | RAM: {:.1} / {:.1} GiB ({:.0}%) ]",
                    cpu.total_pct,
                    mem.ram_used_gib(),
                    mem.ram_total_gib(),
                    mem.ram_usage_percent()
                ),
                _ => "[ CPU: --% | RAM: -- ]".to_owned(),
            },
            None => "[ CPU: --% | RAM: -- ]".to_owned(),
        };
        ui.add(
            egui::Label::new(egui::RichText::new(telemetry).monospace().size(12.0).color(colors::ACCENT_CYAN))
                .wrap(false),
        )
        .on_hover_text(format!(
            "PULSE: {ram_mb:.1} MB | UI: {last_ui_ms:.2} ms | OS: {} | Uptime: {uptime_str}",
            os_locale_short()
        ));

        // Right-aligned single System Health Pill.
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if let Some(ref snapshot) = latest_snapshot {
                let wall = crate::ui::three_walls::evaluate(snapshot);
                let bottleneck = snapshot
                    .cpu
                    .as_ref()
                    .is_some_and(|cpu| cpu.single_thread_bottleneck());
                let (pill_text, pill_color) = match wall {
                    crate::ui::three_walls::WallState::MemoryWall => {
                        ("[ ! MEMORY WALL ]", colors::ACCENT_RED)
                    }
                    crate::ui::three_walls::WallState::Elevated => {
                        ("[ HEALTH: ELEVATED ]", colors::ACCENT_AMBER)
                    }
                    crate::ui::three_walls::WallState::Unconfirmed => {
                        ("[ HEALTH: INIT ]", colors::TEXT_SECONDARY)
                    }
                    crate::ui::three_walls::WallState::Optimal if bottleneck => {
                        ("[ ! BOTTLENECK ]", colors::ACCENT_AMBER)
                    }
                    crate::ui::three_walls::WallState::Optimal => {
                        ("[ HEALTH: NOMINAL ]", crate::theme::OPTIMAL_EMERALD)
                    }
                };
                let resp = ui.label(
                    egui::RichText::new(pill_text)
                        .monospace()
                        .size(11.0)
                        .strong()
                        .color(pill_color),
                );
                if resp.hovered() {
                    // Tooltip built only on hover - avoids per-frame alloc.
                    let tooltip = match snapshot.memory.as_ref() {
                        Some(m) => {
                            let cpu_pct = snapshot.cpu.as_ref().map_or(0.0, |c| c.total_pct);
                            let hot = snapshot.cpu.as_ref().map_or(0.0, |c| c.hottest_core_pct);
                            format!(
                                "Three Walls: {}\nRAM: {:.1}% Commit: {:.1}%\nCPU: {:.1}% Hottest: {:.1}%",
                                wall.label(),
                                m.ram_usage_percent(),
                                m.commit_charge_pct(),
                                cpu_pct,
                                hot
                            )
                        }
                        None => "Awaiting memory telemetry".to_owned(),
                    };
                    resp.on_hover_text(tooltip);
                }
            }
        });
    });
    draw_top_scanline(ui, top_rect);
}

/// Dual-tone neon scanline along the bottom edge of the top bar.
fn draw_top_scanline(ui: &egui::Ui, rect: egui::Rect) {
    let y = rect.bottom() - 0.5;
    let painter = ui.painter();
    let base = egui::Color32::from_rgba_unmultiplied(51, 65, 85, 180);
    let glow = egui::Color32::from_rgba_unmultiplied(0, 240, 255, 220);
    painter.line_segment(
        [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
        egui::Stroke::new(1.0_f32, base),
    );
    painter.line_segment(
        [egui::pos2(rect.left() + 4.0, y), egui::pos2(rect.left() + 144.0, y)],
        egui::Stroke::new(2.0_f32, glow),
    );
}

#[cfg(test)]
#[path = "chrome_tests.rs"]
mod chrome_tests;
