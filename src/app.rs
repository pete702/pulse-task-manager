//! PULSE main application state: UI, telemetry handling, tabs.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::time::Instant;

use eframe::egui;

use crate::app_render::{
    render_central_panel, render_sidebar, render_status_bar, render_top_panel,
};
use crate::core::{
    spawn_telemetry_worker, spawn_telemetry_worker_with_wakeup, LatestSlot, SystemSnapshot, Wakeup,
};
use crate::modules::memory::action_bar::MemoryActionBarState;
use crate::modules::memory::history::MemoryHistory;
use crate::modules::memory::page::MemorySubView;
use crate::modules::memory::table_state::MemoryTableState;

pub struct PulseApp {
    snapshot_rx: mpsc::Receiver<Arc<SystemSnapshot>>,
    latest_slot: LatestSlot,
    worker_minimized: Arc<AtomicBool>,
    latest_snapshot: Option<Arc<SystemSnapshot>>,
    active_tab: crate::app_tabs::ActiveTab,
    memory_sub_view: MemorySubView,
    table_state: MemoryTableState,
    action_state: MemoryActionBarState,
    frame_count: u64,
    last_ui_ms: f32,
    memory_history: MemoryHistory,
    was_minimized: bool,
    startup_frames: u8,
    telemetry_paused: bool,
}

impl PulseApp {
    fn with_worker(worker: crate::core::TelemetryWorker) -> Self {
        Self {
            snapshot_rx: worker.rx,
            latest_slot: worker.latest,
            worker_minimized: worker.is_minimized,
            latest_snapshot: None,
            active_tab: crate::app_tabs::ActiveTab::Memory,
            memory_sub_view: MemorySubView::default(),
            table_state: MemoryTableState::default(),
            action_state: MemoryActionBarState::default(),
            frame_count: 0,
            last_ui_ms: 0.0,
            memory_history: MemoryHistory::new(),
            was_minimized: false,
            startup_frames: 0,
            telemetry_paused: false,
        }
    }
}

impl Default for PulseApp {
    fn default() -> Self {
        Self::with_worker(spawn_telemetry_worker())
    }
}

/// Creates a new PulseApp with a wakeup callback for event-driven repaints.
impl PulseApp {
    pub fn new(ctx: &egui::Context) -> Self {
        let ui_ctx = ctx.clone();
        let wakeup: Wakeup = Arc::new(move || ui_ctx.request_repaint());
        Self::with_worker(spawn_telemetry_worker_with_wakeup(wakeup))
    }

    fn ingest_snapshot(&mut self, snap: Arc<SystemSnapshot>) {
        let is_new = self.latest_snapshot.as_ref().is_none_or(|prev| !Arc::ptr_eq(prev, &snap));
        if !is_new {
            return;
        }
        if let Some(ref mem) = snap.memory {
            self.memory_history.push(mem);
        }
        self.latest_snapshot = Some(snap);
    }

    fn collect_snapshots(&mut self) {
        if self.telemetry_paused {
            // Drain the channel to prevent memory buildup but don't process
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

        if crate::app_tabs::handle_global_hotkeys(
            ctx,
            &mut self.active_tab,
            &mut self.memory_sub_view,
            &mut self.table_state,
        ) {
            ctx.request_repaint_after(std::time::Duration::from_millis(1000));
            return;
        }
        self.frame_count = self.frame_count.wrapping_add(1);
        self.collect_snapshots();
        let is_minimized = ctx.input(|i| i.viewport().minimized.unwrap_or(false));
        self.worker_minimized.store(is_minimized, Ordering::Relaxed);

        if !self.was_minimized && is_minimized {
            let _ = crate::core::win32::self_trim_working_set();
        }
        self.was_minimized = is_minimized;
        if is_minimized {
            return;
        }

        let latest_snapshot = self.latest_snapshot.clone();
        let active_processes = latest_snapshot
            .as_ref()
            .and_then(|s| s.memory.as_ref())
            .map(|m| m.system_processes)
            .unwrap_or(0);

        egui::TopBottomPanel::top("top_header")
            .show(ctx, |ui| render_top_panel(ui, &latest_snapshot, self.last_ui_ms));
        egui::TopBottomPanel::bottom("bottom_status_bar")
            .exact_height(26.0)
            .frame(crate::theme::status_bar_frame())
            .show(ctx, |ui| render_status_bar(ui, active_processes, &latest_snapshot));
        egui::SidePanel::left("left_sidebar")
            .resizable(false)
            .exact_width(210.0)
            .show(ctx, |ui| render_sidebar(ui, &mut self.active_tab, &latest_snapshot));
        egui::CentralPanel::default()
            .frame(egui::Frame::none().inner_margin(egui::Margin {
                left: 12.0,
                right: 14.0,
                top: 10.0,
                bottom: 10.0,
            }))
            .show(ctx, |ui| {
                ui.set_clip_rect(ui.max_rect());
                render_central_panel(
                    ui,
                    self.active_tab,
                    &mut self.memory_sub_view,
                    &mut self.table_state,
                    &mut self.action_state,
                    &mut self.memory_history,
                    &latest_snapshot,
                    latest_snapshot.as_ref().map(|s| s.uptime_secs).unwrap_or(0),
                    &mut self.telemetry_paused,
                );
            });
        self.last_ui_ms = frame_start.elapsed().as_secs_f32() * 1000.0;
        ctx.request_repaint_after(std::time::Duration::from_millis(2000));
    }
}

#[cfg(test)]
#[path = "app_tests.rs"]
mod app_tests;
