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
            let pair = snapshot.as_deref().and_then(|s| s.cpu.as_ref().map(|c| (s, c)));
            let Some((snap, cpu)) = pair else { return render_awaiting(ui) };
            let id = s_cpu_scan_id(snap);
            cpu_page::render_processes_page(ui, cpu, cpu_table_state, id);
        }
        ActiveTab::Memory => {
            let mem = snapshot.as_deref().and_then(|s| s.memory.as_ref());
            let Some(mem) = mem else { return render_awaiting(ui) };
            mem_page::render_memory_page(
                ui,
                mem,
                memory_history,
                uptime_secs,
                active_tab,
                cpu_table_state,
            );
        }
        ActiveTab::Cpu => {
            let pair = snapshot.as_deref().and_then(|s| s.cpu.as_ref().map(|c| (s, c)));
            let Some((snap, cpu)) = pair else { return render_awaiting(ui) };
            let gen = s_cpu_scan_id(snap);
            cpu_page::render_cpu_page(
                ui,
                cpu,
                cpu_history,
                cpu_sub_view,
                cpu_table_state,
                uptime_secs,
                active_tab,
                gen,
            );
        }
        ActiveTab::Storage => {
            let Some(snap) = snapshot.as_deref() else { return render_awaiting(ui) };
            let raw = snap.cpu.as_ref().map(|c| c.raw_processes.as_ref()).unwrap_or(&[]);
            crate::modules::storage::view::render_storage_tab(
                ui,
                &snap.storage,
                raw,
                storage_state,
            );
        }
        ActiveTab::Network => {
            let Some(snap) = snapshot.as_deref() else { return render_awaiting(ui) };
            crate::modules::network::view::render_network_tab(ui, &snap.network, network_state);
        }
        ActiveTab::System => {
            // A vertical child Ui with bounded width keeps the GPU cockpit and
            // both read-only tables inside the central panel's clip rectangle.
            ui.vertical(|ui| {
                crate::ui::system_gpu_tab::render_system_gpu_tab(
                    ui,
                    snapshot,
                    cpu_table_state,
                    cmds,
                );
            });
        }
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
    close_to_tray: &mut bool,
    cmds: &mut Vec<AppCommand>,
) {
    ui.horizontal(|ui| {
        ui.add_space(8.0);
        let label = if telemetry_paused { "Resume" } else { "Pause" };
        if ui.button(label).clicked() {
            cmds.push(AppCommand::TogglePause);
        }
        ui.separator();
        chrome::render_top_panel(ui, snapshot, last_ui_ms, close_to_tray, cmds);
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
    close_to_tray: &mut bool,
    last_ui_ms: f32,
    active_processes: u32,
    cmds: &mut Vec<AppCommand>,
) {
    crate::core::blackbox::observe(snapshot.as_deref());
    crate::modules::cpu::table_tools::poll_pick_into_table(cpu_table_state, cmds);
    crate::modules::cpu::table_tools::handle_dropped_files(ctx, cpu_table_state, cmds);
    let mini_hud_active = crate::platform::hotkey::is_mini_hud_active();
    crate::ui::mini_hud::sync_viewport(ctx, mini_hud_active);
    if mini_hud_active {
        egui::CentralPanel::default()
            .frame(egui::Frame::none().inner_margin(egui::Margin::same(4.0)))
            .show(ctx, |ui| crate::ui::mini_hud::render_mini_hud(ui, snapshot, cmds));
        crate::modules::cpu::table_tools::render_pick_mode_feedback(ctx);
        return;
    }
    let ctrl_k = ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::K));
    if ctrl_k {
        crate::ui::command_palette::toggle();
        ctx.request_repaint();
    }
    let palette_processes: &[crate::modules::cpu::model::RawProcessCpuRow] = snapshot
        .as_ref()
        .and_then(|s| s.cpu.as_ref())
        .map(|c| c.raw_processes.as_ref())
        .unwrap_or(&[]);
    crate::ui::command_palette::render_command_palette(
        ctx,
        active_tab,
        cpu_table_state,
        cmds,
        palette_processes,
    );
    egui::TopBottomPanel::top("top_header").exact_height(36.0).show(ctx, |ui| {
        render_top_panel(ui, snapshot, last_ui_ms, *telemetry_paused, close_to_tray, cmds)
    });
    egui::TopBottomPanel::bottom("bottom_status_bar")
        .exact_height(26.0)
        .frame(crate::theme::status_bar_frame())
        .show(ctx, |ui| {
            chrome::render_status_bar(
                ui,
                active_processes,
                snapshot,
                last_ui_ms,
                crate::core::worker_io::dropped_snapshots(),
            )
        });
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
    crate::modules::cpu::table_tools::render_pick_mode_feedback(ctx);
}
