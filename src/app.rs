//! PULSE main application state: UI, telemetry handling, tabs.
use std::sync::atomic::Ordering;
use std::sync::{mpsc, Arc};
use std::time::Instant;

use eframe::egui;

use crate::app_render::render_chrome;
use crate::core::persist;
use crate::core::{
    spawn_telemetry_worker, spawn_telemetry_worker_with_wakeup, LatestSlot, SystemSnapshot, Wakeup,
};
use crate::modules::cpu::history::CpuHistory;
use crate::modules::cpu::page::CpuSubView;
use crate::modules::cpu::table_state::CpuTableState;
use crate::modules::memory::action_bar::MemoryActionBarState;
use crate::modules::memory::history::MemoryHistory;
use crate::modules::memory::page::MemorySubView;
use crate::modules::memory::table_state::MemoryTableState;

pub struct PulseApp {
    snapshot_rx: mpsc::Receiver<Arc<SystemSnapshot>>,
    latest_slot: LatestSlot,
    latest_snapshot: Option<Arc<SystemSnapshot>>,
    active_tab: crate::app_tabs::ActiveTab,
    prev_active_tab: crate::app_tabs::ActiveTab,
    memory_sub_view: MemorySubView,
    prev_memory_sub_view: MemorySubView,
    cpu_sub_view: CpuSubView,
    prev_cpu_sub_view: CpuSubView,
    table_state: MemoryTableState,
    cpu_table_state: CpuTableState,
    action_state: MemoryActionBarState,
    frame_count: u64,
    last_ui_ms: f32,
    memory_history: MemoryHistory,
    cpu_history: CpuHistory,
    was_minimized: bool,
    startup_frames: u8,
    telemetry_paused: bool,
    worker: crate::core::TelemetryWorker,
    last_viewport_size: Option<egui::Vec2>,
    last_saved: persist::PersistedState,
}

impl PulseApp {
    fn with_worker(
        worker: crate::core::TelemetryWorker,
        snapshot_rx: mpsc::Receiver<Arc<SystemSnapshot>>,
    ) -> Self {
        let latest_slot = worker.latest.clone();
        let mut app = Self {
            snapshot_rx,
            latest_slot,
            latest_snapshot: None,
            active_tab: crate::app_tabs::ActiveTab::Processes,
            prev_active_tab: crate::app_tabs::ActiveTab::Processes,
            memory_sub_view: MemorySubView::default(),
            prev_memory_sub_view: MemorySubView::default(),
            cpu_sub_view: CpuSubView::default(),
            prev_cpu_sub_view: CpuSubView::default(),
            table_state: MemoryTableState::default(),
            cpu_table_state: CpuTableState::default(),
            action_state: MemoryActionBarState::default(),
            frame_count: 0,
            last_ui_ms: 0.0,
            memory_history: MemoryHistory::new(),
            cpu_history: CpuHistory::new(),
            was_minimized: false,
            startup_frames: 0,
            telemetry_paused: false,
            worker,
            last_viewport_size: None,
            last_saved: persist::PersistedState::default(),
        };
        let saved = persist::load_state();
        app.apply_persisted(&saved);
        app
    }

    fn apply_persisted(&mut self, s: &persist::PersistedState) {
        self.active_tab = persist::active_tab_from_index(s.active_tab);
        self.prev_active_tab = self.active_tab;
        self.cpu_table_state.group_clones = s.group_clones;
        self.cpu_table_state.comfort_density = !s.compact_rows;
        self.cpu_table_state.sort_col = s.sort_col;
        self.cpu_table_state.sort_asc = s.sort_asc;
        self.memory_history.chart_window = persist::chart_window_from_secs(s.mem_window_secs);
        self.cpu_history.chart_window = persist::chart_window_from_secs(s.cpu_window_secs);
        self.cpu_history.show_core_grid = s.cpu_show_grid;
        self.last_viewport_size = Some(egui::Vec2::new(s.win_w, s.win_h));
        self.last_saved = s.clone();
    }

    fn collect_persisted(&self) -> persist::PersistedState {
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
            win_w: (win.x.clamp(860.0, 3840.0) * 10.0).round() / 10.0,
            win_h: (win.y.clamp(560.0, 2160.0) * 10.0).round() / 10.0,
        }
    }

    fn save_state_if_needed(&mut self, force: bool) {
        let cur = self.collect_persisted();
        if force || cur != self.last_saved {
            let _ = persist::save_state(&cur);
            self.last_saved = cur;
        }
    }
}

impl Default for PulseApp {
    fn default() -> Self {
        let (worker, snapshot_rx) = spawn_telemetry_worker();
        Self::with_worker(worker, snapshot_rx)
    }
}

impl PulseApp {
    pub fn new(ctx: &egui::Context) -> Self {
        let ui_ctx = ctx.clone();
        let wakeup: Wakeup = Arc::new(move || ui_ctx.request_repaint());
        let (worker, snapshot_rx) = spawn_telemetry_worker_with_wakeup(wakeup);
        let app = Self::with_worker(worker, snapshot_rx);
        crate::modules::memory::chart_view::set_window(ctx, app.memory_history.chart_window);
        crate::modules::cpu::chart_axis::set_window(ctx, app.cpu_history.chart_window);
        app
    }

    fn ingest_snapshot(&mut self, snap: Arc<SystemSnapshot>) {
        let is_new = self.latest_snapshot.as_ref().is_none_or(|prev| !Arc::ptr_eq(prev, &snap));
        if !is_new {
            return;
        }
        if let Some(ref mem) = snap.memory {
            self.memory_history.push(mem);
        }
        if let Some(ref cpu) = snap.cpu {
            let timestamp = snap
                .memory
                .as_ref()
                .map(|m| m.sample_elapsed_secs)
                .unwrap_or(snap.uptime_secs as f64);
            self.cpu_history.push(cpu.total_pct, cpu.hottest_core_pct, timestamp);
        }
        self.latest_snapshot = Some(snap);
    }

    fn collect_snapshots(&mut self) {
        if self.telemetry_paused {
            while self.snapshot_rx.try_recv().is_ok() {}
            let _ = self.latest_slot.take();
            return;
        }
        while let Ok(snap) = self.snapshot_rx.try_recv() {
            self.ingest_snapshot(snap);
        }
        if let Some(snap) = self.latest_slot.take() {
            self.ingest_snapshot(snap);
        }
    }
}

impl eframe::App for PulseApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let frame_start = Instant::now();
        self.startup_frames = self.startup_frames.saturating_add(1);
        if self.startup_frames <= 3 {
            ctx.request_repaint_after(std::time::Duration::from_millis(16));
        }
        if self.startup_frames == 3 {
            let _ = crate::core::win32::self_trim_working_set();
        }
        // Minimized: honest 0 FPS. No hotkeys or repaints while minimized.
        let is_minimized = ctx.input(|i| i.viewport().minimized.unwrap_or(false));
        self.worker.is_minimized.store(is_minimized, Ordering::Relaxed);
        if is_minimized && !self.was_minimized {
            let _ = crate::core::win32::self_trim_working_set();
        }
        self.was_minimized = is_minimized;
        if is_minimized {
            ctx.request_repaint_after(std::time::Duration::from_secs(30));
            return;
        }
        if crate::app_tabs::handle_global_hotkeys(
            ctx,
            &mut self.active_tab,
            &mut self.memory_sub_view,
            &mut self.table_state,
            &mut self.cpu_table_state,
        ) {
            ctx.request_repaint_after(std::time::Duration::from_millis(1000));
            return;
        }
        let processes_active = self.active_tab == crate::app_tabs::ActiveTab::Processes
            || (self.active_tab == crate::app_tabs::ActiveTab::Memory
                && self.memory_sub_view == MemorySubView::Processes)
            || (self.active_tab == crate::app_tabs::ActiveTab::Cpu
                && self.cpu_sub_view == CpuSubView::Processes);
        self.worker.set_fast_process_cadence(processes_active);
        if self.active_tab == crate::app_tabs::ActiveTab::Processes
            && self.prev_active_tab != crate::app_tabs::ActiveTab::Processes
        {
            self.worker.request_process_scan();
        }
        self.prev_active_tab = self.active_tab;
        if self.prev_memory_sub_view != self.memory_sub_view {
            if self.memory_sub_view == MemorySubView::Processes {
                self.worker.request_process_scan();
            }
            self.prev_memory_sub_view = self.memory_sub_view;
        }
        if self.prev_cpu_sub_view != self.cpu_sub_view {
            if self.cpu_sub_view == CpuSubView::Processes {
                self.worker.request_process_scan();
            }
            self.prev_cpu_sub_view = self.cpu_sub_view;
        }
        self.frame_count = self.frame_count.wrapping_add(1);
        self.collect_snapshots();
        let vp = ctx.input(|i| i.viewport().inner_rect.or(i.viewport().outer_rect));
        let current_size = vp.map(|r| r.size()).unwrap_or(egui::Vec2::ZERO);
        if self.last_viewport_size.is_some_and(|p| (p - current_size).length() > 100.0) {
            let _ = crate::core::win32::self_trim_working_set();
        }
        self.last_viewport_size = Some(current_size);
        let latest_snapshot = self.latest_snapshot.clone();
        let active_processes = latest_snapshot
            .as_ref()
            .and_then(|s| s.memory.as_ref())
            .map(|m| m.system_processes)
            .unwrap_or(0);
        let uptime = latest_snapshot.as_ref().map(|s| s.uptime_secs).unwrap_or(0);
        render_chrome(
            ctx,
            &mut self.active_tab,
            &mut self.memory_sub_view,
            &mut self.cpu_sub_view,
            &mut self.table_state,
            &mut self.cpu_table_state,
            &mut self.action_state,
            &mut self.memory_history,
            &mut self.cpu_history,
            &latest_snapshot,
            uptime,
            &mut self.telemetry_paused,
            self.last_ui_ms,
            active_processes,
        );
        self.last_ui_ms = frame_start.elapsed().as_secs_f32() * 1000.0;
        self.save_state_if_needed(false);
        ctx.request_repaint_after(std::time::Duration::from_millis(2000));
    }
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.save_state_if_needed(true);
    }
}

#[cfg(test)]
#[path = "app_tests.rs"]
mod app_tests;
