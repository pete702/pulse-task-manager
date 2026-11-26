//! PULSE main application state: UI, telemetry handling, tabs.
use std::sync::{mpsc, Arc};
use std::time::Instant;

use eframe::egui;

use crate::app_render::render_chrome;
pub(crate) use crate::app_tray::should_use_fast_process_cadence;
use crate::core::persist;
use crate::core::{spawn_telemetry_worker_with_wakeup, SystemSnapshot, Wakeup};
use crate::modules::cpu::history::CpuHistory;
use crate::modules::cpu::page::CpuSubView;
use crate::modules::cpu::table_state::CpuTableState;
use crate::modules::memory::history::MemoryHistory;
use crate::modules::network::latency_probe::LatencyProbe;
use crate::modules::network::view::NetworkViewState;
use crate::modules::storage::view::StorageViewState;

pub struct PulseApp {
    pub(crate) snapshot_rx: mpsc::Receiver<Arc<SystemSnapshot>>,
    pub(crate) latest_snapshot: Option<Arc<SystemSnapshot>>,
    pub(crate) active_tab: crate::app_tabs::ActiveTab,
    pub(crate) prev_active_tab: crate::app_tabs::ActiveTab,
    pub(crate) cpu_sub_view: CpuSubView,
    pub(crate) prev_cpu_sub_view: CpuSubView,
    pub(crate) cpu_table_state: CpuTableState,
    pub(crate) frame_count: u64,
    pub(crate) last_ui_ms: f32,
    pub(crate) memory_history: MemoryHistory,
    pub(crate) cpu_history: CpuHistory,
    pub(crate) latency_probe: LatencyProbe,
    pub(crate) storage_view_state: StorageViewState,
    pub(crate) network_view_state: NetworkViewState,
    pub(crate) was_minimized: bool,
    pub(crate) startup_frames: u8,
    pub(crate) telemetry_paused: bool,
    pub(crate) close_to_tray: bool,
    pub(crate) sidebar_hidden: bool,
    pub(crate) paused_pending_snapshot: Option<Arc<SystemSnapshot>>,
    pub(crate) worker: crate::core::TelemetryWorker,
    pub(crate) last_viewport_size: Option<egui::Vec2>,
    pub(crate) last_saved: persist::PersistedState,
    pub(crate) dirty_since: Option<Instant>,
    pub(crate) dirty_state_snapshot: Option<persist::PersistedState>,
    pub(crate) pointer_down: bool,
    pub(crate) last_self_trim: Instant,
    #[cfg(target_os = "windows")]
    pub(crate) main_hwnd: isize,
    pub(crate) tray: Option<crate::platform::tray::TrayGuard>,
    #[cfg(target_os = "windows")]
    pub(crate) tray_rx: Option<std::sync::mpsc::Receiver<crate::platform::tray::TrayAction>>,
    pub(crate) tray_tooltip_last: Instant,
}

impl PulseApp {
    pub fn with_worker_and_state(
        worker: crate::core::TelemetryWorker,
        snapshot_rx: mpsc::Receiver<Arc<SystemSnapshot>>,
        saved: persist::PersistedState,
    ) -> Self {
        let mut app = Self {
            snapshot_rx,
            latest_snapshot: None,
            active_tab: crate::app_tabs::ActiveTab::Processes,
            prev_active_tab: crate::app_tabs::ActiveTab::Processes,
            cpu_sub_view: CpuSubView::default(),
            prev_cpu_sub_view: CpuSubView::default(),
            cpu_table_state: CpuTableState::default(),
            frame_count: 0,
            last_ui_ms: 0.0,
            memory_history: MemoryHistory::new(),
            cpu_history: CpuHistory::new(),
            latency_probe: LatencyProbe::new(),
            storage_view_state: StorageViewState::new(),
            network_view_state: NetworkViewState::new(),
            was_minimized: false,
            startup_frames: 0,
            telemetry_paused: false,
            close_to_tray: false,
            sidebar_hidden: false,
            paused_pending_snapshot: None,
            worker,
            last_viewport_size: None,
            last_saved: persist::PersistedState::default(),
            dirty_since: None,
            dirty_state_snapshot: None,
            pointer_down: false,
            last_self_trim: Instant::now(),
            #[cfg(target_os = "windows")]
            main_hwnd: 0,
            tray: None,
            #[cfg(target_os = "windows")]
            tray_rx: None,
            tray_tooltip_last: Instant::now(),
        };
        app.apply_persisted(&saved);
        app
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
        crate::app_tray::update(self, ctx);
        let is_minimized = ctx.input(|i| i.viewport().minimized.unwrap_or(false));
        self.worker.set_minimized(is_minimized);
        let just_minimized = is_minimized && !self.was_minimized;
        self.was_minimized = is_minimized;
        crate::app_trim::maybe_self_trim(
            &mut self.last_self_trim,
            self.startup_frames as u32,
            just_minimized,
            is_minimized,
        );
        if is_minimized {
            let _ = self.poll_snapshots();
            ctx.request_repaint_after(std::time::Duration::from_secs(5));
            return;
        }
        let paused_before = self.telemetry_paused;
        let hotkey_tab = self.active_tab;
        let hotkey_paused = self.telemetry_paused;
        if crate::app_tabs::handle_global_hotkeys(
            ctx,
            &mut self.active_tab,
            &mut self.cpu_table_state,
            &mut self.telemetry_paused,
        ) || crate::app_tabs::handle_navigation_hotkeys(
            ctx,
            &mut self.active_tab,
            &mut self.sidebar_hidden,
        ) {
            ctx.request_repaint();
        }
        if self.active_tab != hotkey_tab {
            let tab = std::mem::replace(&mut self.active_tab, hotkey_tab);
            self.apply_command(crate::app_cmd::AppCommand::SetTab(tab));
        }
        if self.telemetry_paused != hotkey_paused {
            self.telemetry_paused = hotkey_paused;
            self.apply_command(crate::app_cmd::AppCommand::TogglePause);
        }
        if !paused_before && self.telemetry_paused {
            self.paused_pending_snapshot = None;
        }
        let ingested = self.poll_snapshots();
        if paused_before && !self.telemetry_paused {
            if let Some(pending) = self.paused_pending_snapshot.take() {
                let last_id = self
                    .latest_snapshot
                    .as_ref()
                    .and_then(|s| s.memory.as_ref())
                    .map(|m| m.snapshot_id);
                let pending_id = pending.memory.as_ref().map(|m| m.snapshot_id);
                let should_apply = self.latest_snapshot.is_none()
                    || pending.uptime_secs
                        > self.latest_snapshot.as_ref().map(|s| s.uptime_secs).unwrap_or(0)
                    || pending_id != last_id;
                if should_apply {
                    let _ = self.ingest_snapshot(&pending);
                }
            }
            self.worker.request_process_scan();
        }
        if ingested {
            ctx.request_repaint();
        }
        self.worker.set_fast_process_cadence(should_use_fast_process_cadence(self.active_tab));
        self.worker.set_paused(self.telemetry_paused);
        crate::modules::network::latency_probe::set_fast_probe_mode(
            self.active_tab == crate::app_tabs::ActiveTab::Network && !self.telemetry_paused,
        );
        if self.active_tab == crate::app_tabs::ActiveTab::Processes
            && self.prev_active_tab != crate::app_tabs::ActiveTab::Processes
        {
            self.apply_command(crate::app_cmd::AppCommand::RequestProcessScan);
        }
        if self.active_tab == crate::app_tabs::ActiveTab::Storage
            && self.prev_active_tab != crate::app_tabs::ActiveTab::Storage
        {
            crate::modules::storage::collector::request_storage_refresh();
        }
        self.prev_active_tab = self.active_tab;
        if self.prev_cpu_sub_view != self.cpu_sub_view {
            if self.cpu_sub_view == CpuSubView::Processes {
                self.worker.request_process_scan();
            }
            self.prev_cpu_sub_view = self.cpu_sub_view;
        }
        let _ = &self.latency_probe;
        self.frame_count = self.frame_count.wrapping_add(1);
        if let Some(rect) = ctx.input(|i| i.viewport().inner_rect.or(i.viewport().outer_rect)) {
            let size = rect.size();
            if crate::ui::mini_hud::should_persist_window_size(
                crate::platform::hotkey::is_mini_hud_active(),
                size.x,
                size.y,
            ) {
                self.last_viewport_size = Some(size);
            }
        }
        let latest_snapshot = self.latest_snapshot.clone();
        let active_processes = latest_snapshot
            .as_ref()
            .map(|s| {
                let cpu_n = s.cpu.as_ref().map(|c| c.process_count).unwrap_or(0);
                let mem_n = s.memory.as_ref().map(|m| m.processes.len()).unwrap_or(0);
                u32::try_from(cpu_n.max(mem_n)).unwrap_or(u32::MAX)
            })
            .unwrap_or(0);
        let uptime = latest_snapshot.as_ref().map(|s| s.uptime_secs).unwrap_or(0);
        let mut cmds = Vec::new();
        render_chrome(
            ctx,
            &mut self.active_tab,
            &mut self.cpu_sub_view,
            &mut self.cpu_table_state,
            &mut self.memory_history,
            &mut self.cpu_history,
            &mut self.storage_view_state,
            &mut self.network_view_state,
            &latest_snapshot,
            uptime,
            &mut self.telemetry_paused,
            &mut self.sidebar_hidden,
            &mut self.close_to_tray,
            self.last_ui_ms,
            active_processes,
            &mut cmds,
        );
        for cmd in cmds {
            self.apply_command(cmd);
        }
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
        self.save_state_on_exit();
        let (resumed, failed) = crate::platform::suspend::resume_all_suspended();
        if failed > 0 {
            eprintln!("pulse: exit resume (resumed={resumed}, failed={failed})");
        }
    }
}

#[cfg(test)]
#[path = "app_tests.rs"]
mod app_tests;
