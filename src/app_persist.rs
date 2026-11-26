//! Persisted state and Mini-HUD window geometry helpers for PulseApp.
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use eframe::egui;

use crate::app::PulseApp;
use crate::core::persist;

pub use crate::core::persist::MIN_WINDOW_WIDTH;

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

/// HUD/strip dimensions are transient and must never overwrite the normal window size.
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

impl PulseApp {
    pub(crate) fn apply_persisted(&mut self, s: &persist::PersistedState) {
        self.active_tab = persist::active_tab_from_index(s.active_tab);
        self.prev_active_tab = self.active_tab;
        self.cpu_table_state.set_group_clones(s.group_clones);
        self.cpu_table_state.comfort_density = !s.compact_rows;
        self.cpu_table_state.sort_col = s.sort_col;
        self.cpu_table_state.sort_asc = s.sort_asc;
        self.cpu_table_state.quick_filter =
            crate::modules::cpu::table_state::ProcessQuickFilter::from_u8(s.process_quick_filter);
        crate::platform::hotkey::set_global_hotkey_enabled(s.global_hotkey_enabled);
        self.sidebar_hidden = s.sidebar_hidden;
        self.memory_history.chart_window = persist::chart_window_from_secs(s.mem_window_secs);
        self.cpu_history.chart_window = persist::chart_window_from_secs(s.cpu_window_secs);
        self.cpu_history.set_show_core_grid(s.cpu_show_grid);
        self.last_viewport_size = Some(egui::Vec2::new(s.win_w, s.win_h));
        self.storage_view_state.set_window_secs(s.storage_window_secs as usize);
        self.network_view_state.set_window_secs(s.network_window_secs as usize);
        self.network_view_state.set_chart_mode_value(s.network_chart_mode);
        crate::modules::network::latency_probe::set_probe_enabled(s.network_probe_enabled);
        crate::modules::cpu::table_cols::set_persisted_col_widths(s.cpu_col_widths);
        self.last_saved = s.clone();
    }

    pub(crate) fn collect_persisted(&self) -> persist::PersistedState {
        let win = self.last_viewport_size.unwrap_or(egui::Vec2::new(DEFAULT_WIDTH, DEFAULT_HEIGHT));
        let wx = if win.x.is_finite() { win.x } else { DEFAULT_WIDTH };
        let wy = if win.y.is_finite() { win.y } else { DEFAULT_HEIGHT };
        persist::PersistedState {
            active_tab: persist::active_tab_index(self.active_tab),
            group_clones: self.cpu_table_state.group_clones,
            compact_rows: !self.cpu_table_state.comfort_density,
            sort_col: self.cpu_table_state.sort_col,
            sort_asc: self.cpu_table_state.sort_asc,
            cpu_show_grid: self.cpu_history.show_core_grid(),
            mem_window_secs: persist::secs_from_chart_window(self.memory_history.chart_window),
            cpu_window_secs: persist::secs_from_chart_window(self.cpu_history.chart_window),
            storage_window_secs: self.storage_view_state.window_secs() as u64,
            network_window_secs: self.network_view_state.window_secs() as u64,
            network_chart_mode: self.network_view_state.chart_mode_value(),
            network_probe_enabled: crate::modules::network::latency_probe::probe_enabled(),
            process_quick_filter: self.cpu_table_state.quick_filter.to_u8(),
            global_hotkey_enabled: crate::platform::hotkey::is_global_hotkey_enabled(),
            sidebar_hidden: self.sidebar_hidden,
            cpu_col_widths: crate::modules::cpu::table_cols::get_persisted_col_widths(),
            win_w: (wx.clamp(MIN_WINDOW_WIDTH, 3840.0) * 10.0).round() / 10.0,
            win_h: (wy.clamp(560.0, 2160.0) * 10.0).round() / 10.0,
        }
    }

    /// Drains the async writer first so a queued old write cannot win on exit.
    pub(crate) fn save_state_on_exit(&mut self) {
        persist::join_save_worker();
        persist::flush_staged_state();
        self.save_state_if_needed(true);
    }

    pub(crate) fn save_state_if_needed(&mut self, force: bool) {
        if !force && self.pointer_down {
            return;
        }
        let cur = self.collect_persisted();
        if force {
            if cur != self.last_saved {
                persist::save_now(&cur);
                self.last_saved = cur;
                self.dirty_since = None;
                self.dirty_state_snapshot = None;
            }
            return;
        }
        if cur == self.last_saved {
            return;
        }
        if self.dirty_state_snapshot.as_ref() != Some(&cur) {
            persist::stage_state(&cur);
            self.dirty_state_snapshot = Some(cur.clone());
            self.dirty_since = Some(Instant::now());
            return;
        }
        let due = self.dirty_since.is_some_and(|t| t.elapsed() >= Duration::from_millis(500));
        if due && persist::save_if_due(&cur) {
            self.last_saved = cur;
            self.dirty_since = None;
            self.dirty_state_snapshot = None;
        }
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
        assert_eq!(
            kept_size(false, Some(egui::vec2(11.0, 7.0)), Some(saved)),
            Some(egui::vec2(11.0, 7.0))
        );
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
}
