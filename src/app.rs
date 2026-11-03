//! PULSE main application state: UI, telemetry handling, tabs.

use std::sync::mpsc;
use std::time::Instant;

use eframe::egui;

use crate::core::{spawn_telemetry_worker, SystemSnapshot};
use crate::i18n::Key;
use crate::modules::memory::collector::{trim_current_process_working_set, trim_own_working_set};
use crate::modules::memory::history::MemoryHistory;
use crate::modules::memory::page::{self, MemorySubView};
use crate::modules::memory::table_view::MemoryTableState;
use crate::theme::apply_obsidian_theme;
use crate::ui::{chrome, sidebar};

/// Main application state.
pub struct PulseApp {
    snapshot_rx: mpsc::Receiver<SystemSnapshot>,
    latest_snapshot: Option<SystemSnapshot>,
    active_tab: crate::app_tabs::ActiveTab,
    memory_sub_view: MemorySubView,
    table_state: MemoryTableState,
    frame_count: u64,
    last_ui_ms: f32,
    memory_history: MemoryHistory,
    /// Tracks previous minimized state for edge-triggered trim.
    was_minimized: bool,
}

impl Default for PulseApp {
    fn default() -> Self {
        Self {
            snapshot_rx: spawn_telemetry_worker(),
            latest_snapshot: None,
            active_tab: crate::app_tabs::ActiveTab::Memory,
            memory_sub_view: MemorySubView::default(),
            table_state: MemoryTableState::default(),
            frame_count: 0,
            last_ui_ms: 0.0,
            memory_history: MemoryHistory::new(),
            was_minimized: false,
        }
    }
}

impl PulseApp {
    fn render_central_panel(&mut self, ui: &mut egui::Ui, snapshot: &Option<SystemSnapshot>) {
        ui.add_space(16.0);
        match self.active_tab {
            crate::app_tabs::ActiveTab::Memory => {
                if let Some(ref snap) = snapshot {
                    if let Some(ref mem) = snap.memory {
                        page::render_memory_page(
                            ui,
                            mem,
                            &mut self.memory_history,
                            &mut self.memory_sub_view,
                            &mut self.table_state,
                        );
                    } else {
                        Self::render_awaiting(ui);
                    }
                } else {
                    Self::render_awaiting(ui);
                }
            }
            crate::app_tabs::ActiveTab::Cpu => chrome::render_placeholder_tab(ui, Key::TabCpu.t()),
            crate::app_tabs::ActiveTab::Storage => {
                chrome::render_placeholder_tab(ui, Key::TabStorage.t())
            }
            crate::app_tabs::ActiveTab::Network => {
                chrome::render_placeholder_tab(ui, Key::TabNetwork.t())
            }
            crate::app_tabs::ActiveTab::System => {
                chrome::render_placeholder_tab(ui, Key::TabSystem.t())
            }
        }
    }

    fn render_awaiting(ui: &mut egui::Ui) {
        ui.label(
            egui::RichText::new(Key::AwaitingMemoryModel.t())
                .color(crate::theme::colors::TEXT_SECONDARY)
                .size(14.0),
        );
    }
}

impl eframe::App for PulseApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let frame_start = Instant::now();

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
        if self.frame_count == 3 || self.frame_count.is_multiple_of(30) {
            trim_own_working_set();
        }

        apply_obsidian_theme(ctx);

        while let Ok(snap) = self.snapshot_rx.try_recv() {
            if let Some(ref mem) = snap.memory {
                self.memory_history.push(mem);
            }
            self.latest_snapshot = Some(snap);
        }

        let latest_snapshot = self.latest_snapshot.clone();
        let active_processes = latest_snapshot
            .as_ref()
            .and_then(|s| s.memory.as_ref())
            .map(|m| m.system_processes)
            .unwrap_or(0);

        let is_minimized = ctx.input(|i| i.viewport().minimized.unwrap_or(false));

        // Edge-triggered minimize self-trim: only trim when transitioning TO minimized state
        if is_minimized && !self.was_minimized {
            trim_current_process_working_set();
        }
        self.was_minimized = is_minimized;

        if is_minimized {
            ctx.request_repaint_after(std::time::Duration::from_millis(1000));
            return;
        }

        egui::TopBottomPanel::top("top_header").show(ctx, |ui| {
            chrome::render_top_panel(ui, &latest_snapshot, self.last_ui_ms);
        });
        egui::TopBottomPanel::bottom("bottom_status_bar")
            .exact_height(26.0)
            .frame(crate::theme::status_bar_frame())
            .show(ctx, |ui| {
                chrome::render_status_bar(ui, active_processes, &latest_snapshot);
            });
        egui::SidePanel::left("left_sidebar")
            .resizable(false)
            .exact_width(210.0)
            .show(ctx, |ui| {
                sidebar::render_sidebar(ui, &mut self.active_tab, &latest_snapshot);
            });
        egui::CentralPanel::default()
            .frame(egui::Frame::none().inner_margin(egui::Margin {
                left: 12.0,
                right: 14.0,
                top: 10.0,
                bottom: 10.0,
            }))
            .show(ctx, |ui| {
                // Hard clipping: no child widget can ever paint outside CentralPanel's bounds
                ui.set_clip_rect(ui.max_rect());
                self.render_central_panel(ui, &latest_snapshot);
            });

        self.last_ui_ms = frame_start.elapsed().as_secs_f32() * 1000.0;
        ctx.request_repaint_after(std::time::Duration::from_millis(1000));
    }
}

#[cfg(test)]
#[path = "app_tests.rs"]
mod app_tests;
