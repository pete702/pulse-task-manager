//! PULSE rendering helpers — split from app.rs for modularity and line limits.

use std::sync::Arc;

use eframe::egui;

use crate::app_tabs::ActiveTab;
use crate::core::SystemSnapshot;
use crate::i18n::Key;
use crate::modules::cpu::history::CpuHistory;
use crate::modules::cpu::page::{self as cpu_page, CpuSubView};
use crate::modules::cpu::table_state::CpuTableState;
use crate::modules::memory::action_bar::MemoryActionBarState;
use crate::modules::memory::history::MemoryHistory;
use crate::modules::memory::page::{self as mem_page, MemorySubView};
use crate::modules::memory::table_state::MemoryTableState;
use crate::ui::{chrome, sidebar};

/// Renders the central panel content based on the active tab.
#[allow(clippy::too_many_arguments)]
pub fn render_central_panel(
    ui: &mut egui::Ui,
    active_tab: &mut ActiveTab,
    memory_sub_view: &mut MemorySubView,
    cpu_sub_view: &mut CpuSubView,
    table_state: &mut MemoryTableState,
    cpu_table_state: &mut CpuTableState,
    action_state: &mut MemoryActionBarState,
    memory_history: &mut MemoryHistory,
    cpu_history: &mut CpuHistory,
    snapshot: &Option<Arc<SystemSnapshot>>,
    uptime_secs: u64,
    telemetry_paused: &mut bool,
) {
    ui.add_space(16.0);
    // Background ingestion: one point per genuine process scan, even on other tabs.
    if let Some(ref snap) = snapshot {
        let raw: &[crate::modules::cpu::model::RawProcessCpuRow] =
            snap.cpu.as_ref().map(|c| c.raw_processes.as_ref()).unwrap_or(&[]);
        crate::modules::storage::view::ingest_storage_snapshot(
            &snap.storage,
            raw,
            *telemetry_paused,
        );
        crate::modules::network::view::ingest_network_snapshot(&snap.network, *telemetry_paused);
    }
    match *active_tab {
        ActiveTab::Processes => {
            if let Some(ref snap) = snapshot {
                if let Some(ref cpu) = snap.cpu {
                    let process_scan_id =
                        snap.memory.as_ref().map(|m| m.process_scan_id).unwrap_or(0);
                    cpu_page::render_processes_page(ui, cpu, cpu_table_state, process_scan_id);
                } else {
                    render_awaiting(ui);
                }
            } else {
                render_awaiting(ui);
            }
        }
        ActiveTab::Memory => {
            if let Some(ref snap) = snapshot {
                if let Some(ref mem) = snap.memory {
                    mem_page::render_memory_page(
                        ui,
                        mem,
                        memory_history,
                        memory_sub_view,
                        table_state,
                        action_state,
                        uptime_secs,
                        telemetry_paused,
                        active_tab,
                        cpu_table_state,
                    );
                } else {
                    render_awaiting(ui);
                }
            } else {
                render_awaiting(ui);
            }
        }
        ActiveTab::Cpu => {
            if let Some(ref snap) = snapshot {
                if let Some(ref cpu) = snap.cpu {
                    cpu_page::render_cpu_page(
                        ui,
                        cpu,
                        cpu_history,
                        cpu_sub_view,
                        cpu_table_state,
                        uptime_secs,
                        active_tab,
                    );
                } else {
                    render_awaiting(ui);
                }
            } else {
                render_awaiting(ui);
            }
        }
        ActiveTab::Storage => {
            if let Some(ref snap) = snapshot {
                let raw: &[crate::modules::cpu::model::RawProcessCpuRow] =
                    snap.cpu.as_ref().map(|c| c.raw_processes.as_ref()).unwrap_or(&[]);
                crate::modules::storage::view::render_storage_tab(ui, &snap.storage, raw);
            } else {
                render_awaiting(ui);
            }
        }
        ActiveTab::Network => {
            if let Some(ref snap) = snapshot {
                crate::modules::network::view::render_network_tab(ui, &snap.network);
            } else {
                render_awaiting(ui);
            }
        }
        ActiveTab::System => chrome::render_placeholder_tab(ui, Key::TabSystem.t(), "v0.5.0"),
    }
}

/// Renders the awaiting memory model message.
fn render_awaiting(ui: &mut egui::Ui) {
    let text = egui::RichText::new(Key::AwaitingMemoryModel.t()).size(14.0);
    ui.label(text.color(crate::theme::colors::TEXT_SECONDARY));
}

/// Renders the top header panel.
pub fn render_top_panel(
    ui: &mut egui::Ui,
    snapshot: &Option<Arc<SystemSnapshot>>,
    last_ui_ms: f32,
) {
    chrome::render_top_panel(ui, snapshot, last_ui_ms);
}

/// Renders the bottom status bar.
pub fn render_status_bar(
    ui: &mut egui::Ui,
    active_processes: u32,
    snapshot: &Option<Arc<SystemSnapshot>>,
    last_ui_ms: f32,
) {
    chrome::render_status_bar(ui, active_processes, snapshot, last_ui_ms);
}

/// Renders the left sidebar.
pub fn render_sidebar(
    ui: &mut egui::Ui,
    active_tab: &mut ActiveTab,
    snapshot: &Option<Arc<SystemSnapshot>>,
) {
    sidebar::render_sidebar(ui, active_tab, snapshot);
}

/// Renders all frame chrome (top, bottom, sidebar, central) in one call.
#[allow(clippy::too_many_arguments)]
pub fn render_chrome(
    ctx: &egui::Context,
    active_tab: &mut ActiveTab,
    memory_sub_view: &mut MemorySubView,
    cpu_sub_view: &mut CpuSubView,
    table_state: &mut MemoryTableState,
    cpu_table_state: &mut CpuTableState,
    action_state: &mut MemoryActionBarState,
    memory_history: &mut MemoryHistory,
    cpu_history: &mut CpuHistory,
    snapshot: &Option<Arc<SystemSnapshot>>,
    uptime_secs: u64,
    telemetry_paused: &mut bool,
    last_ui_ms: f32,
    active_processes: u32,
) {
    egui::TopBottomPanel::top("top_header")
        .show(ctx, |ui| render_top_panel(ui, snapshot, last_ui_ms));
    egui::TopBottomPanel::bottom("bottom_status_bar")
        .exact_height(26.0)
        .frame(crate::theme::status_bar_frame())
        .show(ctx, |ui| render_status_bar(ui, active_processes, snapshot, last_ui_ms));
    egui::SidePanel::left("left_sidebar")
        .resizable(false)
        .exact_width(210.0)
        .show(ctx, |ui| render_sidebar(ui, active_tab, snapshot));
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
                active_tab,
                memory_sub_view,
                cpu_sub_view,
                table_state,
                cpu_table_state,
                action_state,
                memory_history,
                cpu_history,
                snapshot,
                uptime_secs,
                telemetry_paused,
            );
        });
}
