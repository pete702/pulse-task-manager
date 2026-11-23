//! Compact 330x124 Mini-HUD strip and viewport transition handling.

use eframe::egui;
use std::sync::{Arc, Mutex, OnceLock};

use crate::app_cmd::AppCommand;
use crate::core::SystemSnapshot;

pub const MINI_HUD_WIDTH: f32 = 330.0;
pub const MINI_HUD_HEIGHT: f32 = 124.0;
const SIZE_TOLERANCE: f32 = 4.0;
const LEGACY_HUD_HEIGHT: f32 = 104.0;
const DEFAULT_WIDTH: f32 = 1180.0;
const DEFAULT_HEIGHT: f32 = 760.0;

#[derive(Default)]
struct Geometry {
    normal_size: Option<egui::Vec2>,
    normal_pos: Option<egui::Pos2>,
    was_active: bool,
    was_maximized: bool,
    followup: u8,
}

static GEOMETRY: OnceLock<Mutex<Geometry>> = OnceLock::new();

fn geometry() -> &'static Mutex<Geometry> {
    GEOMETRY.get_or_init(|| Mutex::new(Geometry::default()))
}

pub fn should_persist_window_size(mini_hud_active: bool, width: f32, height: f32) -> bool {
    if mini_hud_active || !width.is_finite() || !height.is_finite() {
        return false;
    }
    let strip_height = (height - MINI_HUD_HEIGHT).abs() <= SIZE_TOLERANCE
        || (height - LEGACY_HUD_HEIGHT).abs() <= SIZE_TOLERANCE;
    !((width - MINI_HUD_WIDTH).abs() <= SIZE_TOLERANCE && strip_height)
}

pub fn restored_normal_size(saved: Option<egui::Vec2>) -> egui::Vec2 {
    match saved {
        Some(v) if v.is_finite() && v.x > 0.0 && v.y > 0.0 => v,
        _ => egui::vec2(DEFAULT_WIDTH, DEFAULT_HEIGHT),
    }
}

/// Maximized full-screen bounds are never the window's normal size.
fn kept_size(max: bool, new: Option<egui::Vec2>, old: Option<egui::Vec2>) -> Option<egui::Vec2> {
    if max {
        old
    } else {
        new.or(old)
    }
}

fn restore_commands(ctx: &egui::Context, g: &Geometry) {
    ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(egui::WindowLevel::Normal));
    ctx.send_viewport_cmd(egui::ViewportCommand::Decorations(true));
    ctx.send_viewport_cmd(egui::ViewportCommand::MinInnerSize(egui::vec2(860.0, 560.0)));
    ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(restored_normal_size(g.normal_size)));
    if g.was_maximized {
        ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(true));
    } else if let Some(pos) = g.normal_pos {
        ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(pos));
    }
}

pub fn sync_viewport(ctx: &egui::Context, active: bool) {
    let mut g = geometry().lock().unwrap_or_else(|e| e.into_inner());
    let maximized = ctx.input(|i| i.viewport().maximized.unwrap_or(false));
    if active != g.was_active {
        if active {
            g.was_maximized = maximized;
            let inner = ctx.input(|i| i.viewport().inner_rect.map(|r| r.size()));
            g.normal_size = kept_size(maximized, inner, g.normal_size);
            if !maximized {
                g.normal_pos = ctx.input(|i| i.viewport().outer_rect.map(|r| r.left_top()));
            }
            // Windows ignores InnerSize while the HWND is maximized.
            ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(false));
            ctx.send_viewport_cmd(egui::ViewportCommand::MinInnerSize(egui::vec2(
                MINI_HUD_WIDTH,
                MINI_HUD_HEIGHT,
            )));
            ctx.send_viewport_cmd(egui::ViewportCommand::Decorations(false));
            ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(
                egui::WindowLevel::AlwaysOnTop,
            ));
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(
                MINI_HUD_WIDTH,
                MINI_HUD_HEIGHT,
            )));
        } else {
            restore_commands(ctx, &g);
        }
        g.followup = 2;
    } else if g.followup > 0 {
        if active {
            ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(false));
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(
                MINI_HUD_WIDTH,
                MINI_HUD_HEIGHT,
            )));
        } else {
            restore_commands(ctx, &g);
        }
        g.followup -= 1;
        if g.followup == 0 {
            g.was_maximized = false;
        }
    }
    g.was_active = active;
}

fn pct(v: f32) -> f32 {
    if v.is_finite() {
        v.clamp(0.0, 100.0)
    } else {
        0.0
    }
}

pub fn format_metrics(
    cpu_pct: f32,
    ram_pct: f32,
    peak_core_pct: f32,
    uptime_secs: u64,
    self_ram_mb: f32,
) -> [String; 5] {
    [
        format!("CPU: {:.1}%", pct(cpu_pct)),
        format!("RAM: {:.1}%", pct(ram_pct)),
        format!("Peak Core: {:.1}%", pct(peak_core_pct)),
        format!("Uptime: {}", crate::util::format_uptime(uptime_secs)),
        format!("Self-RAM: {:.1} MB", if self_ram_mb.is_finite() { self_ram_mb } else { 0.0 }),
    ]
}

/// Two crisp lines that fit the 330px strip without right-edge clipping.
pub fn format_hud_lines(
    cpu_pct: f32,
    ram_pct: f32,
    peak_core_pct: f32,
    uptime_secs: u64,
    self_ram_mb: f32,
) -> [String; 2] {
    let m = format_metrics(cpu_pct, ram_pct, peak_core_pct, uptime_secs, self_ram_mb);
    [
        format!("{} | {} | {}", m[0], m[1], m[2].replace("Peak Core", "Peak")),
        format!(
            "{} | PULSE: {:.1} MB",
            m[3],
            if self_ram_mb.is_finite() { self_ram_mb } else { 0.0 }
        ),
    ]
}

pub fn render_mini_hud(
    ui: &mut egui::Ui,
    snapshot: &Option<Arc<SystemSnapshot>>,
    cmds: &mut Vec<AppCommand>,
) {
    ui.horizontal(|ui| {
        let drag_width = (ui.available_width() - 150.0).max(40.0);
        let (drag_rect, drag) =
            ui.allocate_exact_size(egui::vec2(drag_width, 22.0), egui::Sense::click_and_drag());
        if drag.drag_started_by(egui::PointerButton::Primary) {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
        }
        ui.allocate_ui_at_rect(drag_rect, |ui| {
            ui.label(egui::RichText::new("PULSE // HUD").monospace().size(13.0).strong());
        });
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("[ EXPAND ]").clicked() {
                crate::platform::hotkey::set_mini_hud_active(false);
            }
            if ui.button("[ TRAY ]").clicked() {
                cmds.push(AppCommand::HideToTray);
            }
        });
    });
    ui.separator();
    let (cpu, ram, peak, uptime, self_ram) = match snapshot.as_ref() {
        Some(s) => {
            let cpu_pct = s.cpu.as_ref().map(|c| c.total_pct).unwrap_or(0.0);
            let peak = s.cpu.as_ref().map(|c| c.hottest_core_pct).unwrap_or(0.0);
            let ram_pct = s.memory.as_ref().map(|m| m.ram_usage_percent()).unwrap_or(0.0);
            (cpu_pct, ram_pct, peak, s.uptime_secs, s.pulse_ram_mb)
        }
        None => (0.0, 0.0, 0.0, 0, 0.0),
    };
    let lines = format_hud_lines(cpu, ram, peak, uptime, self_ram);
    let wall = snapshot.as_ref().map(|s| crate::ui::three_walls::evaluate(s));
    let status = match wall {
        Some(crate::ui::three_walls::WallState::MemoryWall) => "[ MEMORY WALL ]",
        Some(crate::ui::three_walls::WallState::Elevated) => "[ ELEVATED ]",
        Some(crate::ui::three_walls::WallState::Optimal) => "[ NOMINAL ]",
        Some(crate::ui::three_walls::WallState::Unconfirmed) | None => "[ INIT ]",
    };
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(status).monospace().size(11.0));
    });
    if let Some(verdict) = crate::core::blackbox::latest_verdict() {
        let short: String = verdict.chars().take(42).collect();
        ui.label(
            egui::RichText::new(short)
                .monospace()
                .size(10.0)
                .color(crate::theme::colors::ACCENT_AMBER),
        );
    }
    for line in &lines {
        ui.label(egui::RichText::new(line).monospace().size(12.0));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn persist_guard_rejects_hud_and_strip_dimensions() {
        assert_eq!(MINI_HUD_HEIGHT, 124.0);
        assert!(!should_persist_window_size(true, 1180.0, 760.0));
        assert!(!should_persist_window_size(false, MINI_HUD_WIDTH, 124.0));
        assert!(!should_persist_window_size(false, MINI_HUD_WIDTH, 104.0));
        assert!(!should_persist_window_size(false, 332.0, 121.0));
        assert!(should_persist_window_size(false, 1180.0, 760.0));
        assert!(!should_persist_window_size(false, f32::NAN, 760.0));
        assert!(!Geometry::default().was_maximized);
    }

    #[test]
    fn maximized_entry_keeps_saved_normal_size() {
        let saved = egui::vec2(1000.0, 640.0);
        assert_eq!(kept_size(true, Some(egui::vec2(1920.0, 1080.0)), Some(saved)), Some(saved));
        let new = egui::vec2(11.0, 7.0);
        assert_eq!(kept_size(false, Some(new), Some(saved)), Some(new));
        assert_eq!(kept_size(false, None, Some(saved)), Some(saved));
    }

    #[test]
    fn restored_normal_size_falls_back_when_missing_or_invalid() {
        assert_eq!(
            restored_normal_size(Some(egui::vec2(1000.0, 640.0))),
            egui::vec2(1000.0, 640.0)
        );
        assert_eq!(restored_normal_size(None), egui::vec2(DEFAULT_WIDTH, DEFAULT_HEIGHT));
        assert_eq!(
            restored_normal_size(Some(egui::vec2(f32::NAN, 10.0))),
            egui::vec2(DEFAULT_WIDTH, DEFAULT_HEIGHT)
        );
    }

    #[test]
    fn compact_metric_formatting_is_stable() {
        let m = format_metrics(42.4, 61.8, 97.2, 0, 123.4);
        assert_eq!(m[0], "CPU: 42.4%");
        assert_eq!(m[1], "RAM: 61.8%");
        assert_eq!(m[2], "Peak Core: 97.2%");
        assert!(m[3].starts_with("Uptime: "));
        assert_eq!(m[4], "Self-RAM: 123.4 MB");
    }

    #[test]
    fn hud_lines_fit_the_strip_without_clipping() {
        let lines = format_hud_lines(11.3, 48.1, 56.4, 123, 123.4);
        assert_eq!(lines[0], "CPU: 11.3% | RAM: 48.1% | Peak: 56.4%");
        assert!(lines[1].starts_with("Uptime: "));
        assert!(lines[1].contains("PULSE: 123.4 MB"));
        // Monospace 12px is about 7.2px per glyph; 330px fits <= 40 glyphs.
        assert!(lines[0].len() <= 40);
        assert!(lines[1].len() <= 40);
    }
}
