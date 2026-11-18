//! PULSE rendering helpers -- split from app.rs for modularity and line limits.

use std::sync::Arc;

use eframe::egui;

use crate::app_cmd::AppCommand;
use crate::app_tabs::ActiveTab;
use crate::core::SystemSnapshot;
use crate::i18n::Key;
use crate::modules::cpu::history::CpuHistory;
use crate::modules::cpu::page::{self as cpu_page, CpuSubView};
use crate::modules::cpu::table_state::CpuTableState;
use crate::modules::memory::history::MemoryHistory;
use crate::modules::memory::page::{self as mem_page};
use crate::ui::{chrome, sidebar};

fn s_cpu_scan_id(snap: &SystemSnapshot) -> u64 {
    snap.cpu
        .as_ref()
        .map(|c| c.process_scan_id)
        .or_else(|| snap.memory.as_ref().map(|m| m.process_scan_id))
        .unwrap_or(0)
}

#[allow(clippy::too_many_arguments)]
pub fn render_central_panel(
    ui: &mut egui::Ui,
    active_tab: &mut ActiveTab,
    cpu_sub_view: &mut CpuSubView,
    cpu_table_state: &mut CpuTableState,
    memory_history: &mut MemoryHistory,
    cpu_history: &mut CpuHistory,
    storage_state: &mut crate::modules::storage::view::StorageViewState,
    network_state: &mut crate::modules::network::view::NetworkViewState,
    snapshot: &Option<Arc<SystemSnapshot>>,
    uptime_secs: u64,
    _telemetry_paused: &mut bool,
    cmds: &mut Vec<AppCommand>,
) {
    ui.add_space(16.0);
    if *active_tab == ActiveTab::Memory {
        ui.horizontal(|ui| {
            if ui.button("Trim Memory").clicked() {
                cmds.push(AppCommand::RequestMemoryTrim);
            }
        });
        ui.add_space(4.0);
    }
    match *active_tab {
        ActiveTab::Processes => {
            if let Some(ref snap) = snapshot {
                if let Some(ref cpu) = snap.cpu {
                    let process_scan_id = s_cpu_scan_id(snap);
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
                        uptime_secs,
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
                    let scan_generation = s_cpu_scan_id(snap);
                    cpu_page::render_cpu_page(
                        ui,
                        cpu,
                        cpu_history,
                        cpu_sub_view,
                        cpu_table_state,
                        uptime_secs,
                        active_tab,
                        scan_generation,
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
                crate::modules::storage::view::render_storage_tab(
                    ui,
                    &snap.storage,
                    raw,
                    storage_state,
                );
            } else {
                render_awaiting(ui);
            }
        }
        ActiveTab::Network => {
            if let Some(ref snap) = snapshot {
                crate::modules::network::view::render_network_tab(ui, &snap.network, network_state);
            } else {
                render_awaiting(ui);
            }
        }
        ActiveTab::System => chrome::render_placeholder_tab(ui, Key::TabSystem.t(), "v0.5.0"),
    }
}

fn render_awaiting(ui: &mut egui::Ui) {
    ui.label(
        egui::RichText::new(Key::AwaitingMemoryModel.t())
            .size(14.0)
            .color(crate::theme::colors::TEXT_SECONDARY),
    );
}

pub fn render_top_panel(
    ui: &mut egui::Ui,
    snapshot: &Option<Arc<SystemSnapshot>>,
    last_ui_ms: f32,
    telemetry_paused: bool,
    cmds: &mut Vec<AppCommand>,
) {
    ui.horizontal(|ui| {
        ui.add_space(8.0);
        let label = if telemetry_paused { "Resume" } else { "Pause" };
        if ui.button(label).clicked() {
            cmds.push(AppCommand::TogglePause);
        }
        ui.separator();
        chrome::render_top_panel(ui, snapshot, last_ui_ms);
    });
}

pub fn render_sidebar(
    ui: &mut egui::Ui,
    active_tab: &mut ActiveTab,
    snapshot: &Option<Arc<SystemSnapshot>>,
    cmds: &mut Vec<AppCommand>,
) {
    let old = *active_tab;
    sidebar::render_sidebar(ui, active_tab, snapshot);
    if *active_tab != old {
        cmds.push(AppCommand::SetTab(*active_tab));
    }
}

#[allow(clippy::too_many_arguments)]
pub fn render_chrome(
    ctx: &egui::Context,
    active_tab: &mut ActiveTab,
    cpu_sub_view: &mut CpuSubView,
    cpu_table_state: &mut CpuTableState,
    memory_history: &mut MemoryHistory,
    cpu_history: &mut CpuHistory,
    storage_state: &mut crate::modules::storage::view::StorageViewState,
    network_state: &mut crate::modules::network::view::NetworkViewState,
    snapshot: &Option<Arc<SystemSnapshot>>,
    uptime_secs: u64,
    telemetry_paused: &mut bool,
    last_ui_ms: f32,
    active_processes: u32,
    cmds: &mut Vec<AppCommand>,
) {
    egui::TopBottomPanel::top("top_header")
        .exact_height(36.0)
        .show(ctx, |ui| render_top_panel(ui, snapshot, last_ui_ms, *telemetry_paused, cmds));
    egui::TopBottomPanel::bottom("bottom_status_bar")
        .exact_height(26.0)
        .frame(crate::theme::status_bar_frame())
        .show(ctx, |ui| chrome::render_status_bar(ui, active_processes, snapshot, last_ui_ms));
    egui::SidePanel::left("left_sidebar")
        .resizable(false)
        .exact_width(210.0)
        .show(ctx, |ui| render_sidebar(ui, active_tab, snapshot, cmds));
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
                cpu_sub_view,
                cpu_table_state,
                memory_history,
                cpu_history,
                storage_state,
                network_state,
                snapshot,
                uptime_secs,
                telemetry_paused,
                cmds,
            );
        });
}
