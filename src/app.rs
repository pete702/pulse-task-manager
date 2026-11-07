//! PULSE main application state: UI, telemetry handling, tabs.
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
    pub(crate) snapshot_rx: mpsc::Receiver<Arc<SystemSnapshot>>,
    pub(crate) latest_slot: LatestSlot,
    pub(crate) latest_snapshot: Option<Arc<SystemSnapshot>>,
    pub(crate) active_tab: crate::app_tabs::ActiveTab,
    pub(crate) prev_active_tab: crate::app_tabs::ActiveTab,
    pub(crate) memory_sub_view: MemorySubView,
    pub(crate) prev_memory_sub_view: MemorySubView,
    pub(crate) cpu_sub_view: CpuSubView,
    pub(crate) prev_cpu_sub_view: CpuSubView,
    pub(crate) table_state: MemoryTableState,
    pub(crate) cpu_table_state: CpuTableState,
    pub(crate) action_state: MemoryActionBarState,
    pub(crate) frame_count: u64,
    pub(crate) last_ui_ms: f32,
    pub(crate) memory_history: MemoryHistory,
    pub(crate) cpu_history: CpuHistory,
    pub(crate) was_minimized: bool,
    pub(crate) startup_frames: u8,
    pub(crate) telemetry_paused: bool,
    pub(crate) worker: crate::core::TelemetryWorker,
    pub(crate) last_viewport_size: Option<egui::Vec2>,
    pub(crate) last_saved: persist::PersistedState,
    pub(crate) dirty_since: Option<Instant>,
    pub(crate) dirty_state_snapshot: Option<persist::PersistedState>,
    pub(crate) pointer_down: bool,
    pub(crate) last_trim: Instant,
}

impl PulseApp {
    fn with_worker(
        worker: crate::core::TelemetryWorker,
        snapshot_rx: mpsc::Receiver<Arc<SystemSnapshot>>,
    ) -> Self {
        Self::with_worker_and_state(worker, snapshot_rx, persist::load_state())
    }

    pub fn with_worker_and_state(
        worker: crate::core::TelemetryWorker,
        snapshot_rx: mpsc::Receiver<Arc<SystemSnapshot>>,
        saved: persist::PersistedState,
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
            dirty_since: None,
            dirty_state_snapshot: None,
            pointer_down: false,
            last_trim: Instant::now(),
        };
        app.apply_persisted(&saved);
        app
    }
}

impl Default for PulseApp {
    fn default() -> Self {
        let (worker, snapshot_rx) = spawn_telemetry_worker();
        Self::with_worker(worker, snapshot_rx)
    }
}

impl PulseApp {
    #[allow(dead_code)]
    pub fn new(ctx: &egui::Context) -> Self {
        Self::new_with_saved(ctx, persist::load_state())
    }

    pub fn new_with_saved(ctx: &egui::Context, saved: persist::PersistedState) -> Self {
        let ui_ctx = ctx.clone();
        let wakeup: Wakeup = Arc::new(move || ui_ctx.request_repaint());
        let (worker, snapshot_rx) = spawn_telemetry_worker_with_wakeup(wakeup);
        let app = Self::with_worker_and_state(worker, snapshot_rx, saved);
        crate::modules::memory::chart_view::set_window(ctx, app.memory_history.chart_window);
        crate::modules::cpu::chart_axis::set_window(ctx, app.cpu_history.chart_window);
        app
    }
}

impl eframe::App for PulseApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let frame_start = Instant::now();
        self.startup_frames = self.startup_frames.saturating_add(1);
        if self.startup_frames <= 3 {
            ctx.request_repaint_after(std::time::Duration::from_millis(16));
        }
        // Minimized: honest 0 FPS. No hotkeys or repaints while minimized.
        let is_minimized = ctx.input(|i| i.viewport().minimized.unwrap_or(false));
        self.worker.set_minimized(is_minimized);
        let minimize_edge = is_minimized && !self.was_minimized;
        if minimize_edge || crate::app_trim::periodic_due(&self.last_trim) {
            crate::app_trim::run_trim(&mut self.last_trim);
        }
        if self.startup_frames == 60 && self.latest_snapshot.is_some() {
            crate::app_trim::run_trim(&mut self.last_trim);
        }
        self.was_minimized = is_minimized;
        self.poll_snapshots();
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
            ctx.request_repaint();
        }
        let processes_active = self.active_tab == crate::app_tabs::ActiveTab::Processes
            || self.active_tab == crate::app_tabs::ActiveTab::Storage
            || self.active_tab == crate::app_tabs::ActiveTab::Memory
            || (self.active_tab == crate::app_tabs::ActiveTab::Cpu
                && self.cpu_sub_view == CpuSubView::Processes);
        self.worker.set_fast_process_cadence(processes_active);
        crate::core::worker_loop::set_worker_paused(crate::core::worker_loop::WorkerConfig {
            paused: self.telemetry_paused,
        });
        crate::modules::network::latency_probe::set_fast_probe_mode(
            self.active_tab == crate::app_tabs::ActiveTab::Network && !self.telemetry_paused,
        );
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
        if let Some(rect) = vp {
            self.last_viewport_size = Some(rect.size());
        }
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
        self.pointer_down = ctx.input(|i| i.pointer.any_down());
        self.save_state_if_needed(false);
        let focused = ctx.input(|i| i.viewport().focused.unwrap_or(true));
        ctx.request_repaint_after(std::time::Duration::from_millis(if focused {
            2000
        } else {
            5000
        }));
    }
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.save_state_if_needed(true);
    }
}

#[cfg(test)]
#[path = "app_tests.rs"]
mod app_tests;
