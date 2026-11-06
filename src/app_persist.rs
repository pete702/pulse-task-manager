//! Persisted state helpers for PulseApp, split from app.rs for the 280-line guard.
use std::time::{Duration, Instant};

use crate::app::PulseApp;
use crate::core::persist;

impl PulseApp {
    pub(crate) fn apply_persisted(&mut self, s: &persist::PersistedState) {
        self.active_tab = persist::active_tab_from_index(s.active_tab);
        self.prev_active_tab = self.active_tab;
        self.cpu_table_state.group_clones = s.group_clones;
        self.cpu_table_state.comfort_density = !s.compact_rows;
        self.cpu_table_state.sort_col = s.sort_col;
        self.cpu_table_state.sort_asc = s.sort_asc;
        self.cpu_table_state.quick_filter =
            crate::modules::cpu::table_state::ProcessQuickFilter::from_u8(s.process_quick_filter);
        self.memory_history.chart_window = persist::chart_window_from_secs(s.mem_window_secs);
        self.cpu_history.chart_window = persist::chart_window_from_secs(s.cpu_window_secs);
        self.cpu_history.show_core_grid = s.cpu_show_grid;
        self.last_viewport_size = Some(egui::Vec2::new(s.win_w, s.win_h));
        // Full-tab persistence: storage/network windows, chart mode, probe.
        crate::modules::storage::view::set_window_secs(s.storage_window_secs as usize);
        crate::modules::network::view::set_window_secs(s.network_window_secs as usize);
        crate::modules::network::view::set_chart_mode_value(s.network_chart_mode);
        crate::modules::network::latency_probe::set_probe_enabled(s.network_probe_enabled);
        self.last_saved = s.clone();
    }

    pub(crate) fn collect_persisted(&self) -> persist::PersistedState {
        let win = self.last_viewport_size.unwrap_or(egui::Vec2::new(1180.0, 760.0));
        persist::PersistedState {
            active_tab: persist::active_tab_index(self.active_tab),
            group_clones: self.cpu_table_state.group_clones,
            compact_rows: !self.cpu_table_state.comfort_density,
            sort_col: self.cpu_table_state.sort_col,
            sort_asc: self.cpu_table_state.sort_asc,
            cpu_show_grid: self.cpu_history.show_core_grid,
            mem_window_secs: persist::secs_from_chart_window(self.memory_history.chart_window),
            cpu_window_secs: persist::secs_from_chart_window(self.cpu_history.chart_window),
            storage_window_secs: crate::modules::storage::view::window_secs() as u64,
            network_window_secs: crate::modules::network::view::window_secs() as u64,
            network_chart_mode: crate::modules::network::view::chart_mode_value(),
            network_probe_enabled: crate::modules::network::latency_probe::probe_enabled(),
            process_quick_filter: self.cpu_table_state.quick_filter.to_u8(),
            win_w: (win.x.clamp(860.0, 3840.0) * 10.0).round() / 10.0,
            win_h: (win.y.clamp(560.0, 2160.0) * 10.0).round() / 10.0,
        }
    }

    pub(crate) fn save_state_if_needed(&mut self, force: bool) {
        if !force && self.pointer_down {
            return;
        }
        let cur = self.collect_persisted();
        if force {
            if cur != self.last_saved {
                let _ = persist::save_state(&cur);
                self.last_saved = cur;
            }
            return;
        }
        if cur == self.last_saved {
            return;
        }
        // 500ms debounce: only write after state is stable.
        let due = self.dirty_since.is_none_or(|t| t.elapsed() >= Duration::from_millis(500));
        if due {
            if self.dirty_since.is_some() {
                let _ = persist::save_state(&cur);
                self.last_saved = cur;
                self.dirty_since = None;
            } else {
                self.dirty_since = Some(Instant::now());
            }
        }
    }
}
