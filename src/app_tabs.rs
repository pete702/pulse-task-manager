//! Tab definitions for PULSE application.

use crate::i18n::Key;
use crate::modules::cpu::table_state::CpuTableState;
use crate::modules::memory::page::MemorySubView;
use crate::modules::memory::table_state::MemoryTableState;
use eframe::egui;

/// Active tab in the left sidebar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveTab {
    /// Master unified process table tab.
    Processes,
    /// Memory tab.
    Memory,
    /// CPU tab.
    Cpu,
    /// Storage tab.
    Storage,
    /// Network tab.
    Network,
    /// System tab.
    System,
}

impl ActiveTab {
    /// Returns the hardware tabs. The master Processes tab is prepended by the sidebar.
    pub const fn all() -> &'static [ActiveTab] {
        &[
            ActiveTab::Memory,
            ActiveTab::Cpu,
            ActiveTab::Storage,
            ActiveTab::Network,
            ActiveTab::System,
        ]
    }

    /// Button label for the tab (localized).
    pub fn label(self) -> &'static str {
        match self {
            ActiveTab::Processes => "[00] Processes",
            ActiveTab::Memory => Key::TabMemory.t(),
            ActiveTab::Cpu => Key::TabCpu.t(),
            ActiveTab::Storage => Key::TabStorage.t(),
            ActiveTab::Network => Key::TabNetwork.t(),
            ActiveTab::System => Key::TabSystem.t(),
        }
    }
}

/// Handles global hotkeys for the application.
/// Returns true if a hotkey was handled and the UI should repaint immediately.
pub fn handle_global_hotkeys(
    ctx: &egui::Context,
    active_tab: &mut ActiveTab,
    memory_sub_view: &mut MemorySubView,
    table_state: &mut MemoryTableState,
    cpu_table_state: &mut CpuTableState,
    telemetry_paused: &mut bool,
) -> bool {
    let wants_input = ctx.wants_keyboard_input();
    let (ctrl_f, esc, num_idx, f5, f10) = ctx.input(|i| {
        let ctrl = i.modifiers.ctrl;
        let keys = [
            egui::Key::Num1,
            egui::Key::Num2,
            egui::Key::Num3,
            egui::Key::Num4,
            egui::Key::Num5,
            egui::Key::Num6,
        ];
        let num_idx = if ctrl { keys.iter().position(|k| i.key_pressed(*k)) } else { None };
        let f5 = i.key_pressed(egui::Key::F5);
        let f10 = i.key_pressed(egui::Key::F10);
        (ctrl && i.key_pressed(egui::Key::F), i.key_pressed(egui::Key::Escape), num_idx, f5, f10)
    });
    let mut handled = false;
    if f5 && !wants_input {
        crate::core::request_process_scan();
        handled = true;
    }
    if f10 && !wants_input {
        *telemetry_paused = !*telemetry_paused;
        crate::core::worker_loop::set_worker_paused(crate::core::worker_loop::WorkerConfig {
            paused: *telemetry_paused,
        });
        handled = true;
    }
    // Ctrl+F: Focus search in the master Processes tab
    if ctrl_f && !wants_input {
        *active_tab = ActiveTab::Processes;
        *memory_sub_view = MemorySubView::Overview;
        cpu_table_state.focus_search_next_frame = true;
        table_state.focus_search_next_frame = true;
        ctx.request_repaint();
        handled = true;
    }
    // Escape: Clear search or release focus and clear selection
    if esc {
        if *active_tab == ActiveTab::Cpu {
            ctx.memory_mut(|m| m.surrender_focus(egui::Id::new("cpu_search_filter")));
        }
        let has_query =
            !table_state.search_query.is_empty() || !cpu_table_state.search_query.is_empty();
        if has_query {
            if !table_state.search_query.is_empty() {
                table_state.clear_search();
                ctx.request_repaint();
                handled = true;
            } else if !cpu_table_state.search_query.is_empty() {
                cpu_table_state.clear_search();
                ctx.request_repaint();
                handled = true;
            }
        } else {
            ctx.memory_mut(|m| {
                if let Some(id) = m.focused() {
                    m.surrender_focus(id);
                }
            });
            table_state.clear_selection();
            cpu_table_state.selected_pid = None;
            cpu_table_state.selected_name = None;
            cpu_table_state.confirm_kill = None;
            cpu_table_state.mark_dirty();
            if *active_tab == ActiveTab::Storage {
                crate::modules::storage::view::clear_selected();
            } else if *active_tab == ActiveTab::Network {
                crate::modules::network::view::set_scope_all();
            }
            handled = true;
        }
    }
    // Ctrl+1..Ctrl+6: switch tabs quickly
    if !wants_input {
        if let Some(idx) = num_idx {
            let tabs = [
                ActiveTab::Processes,
                ActiveTab::Memory,
                ActiveTab::Cpu,
                ActiveTab::Storage,
                ActiveTab::Network,
                ActiveTab::System,
            ];
            *active_tab = tabs[idx];
            ctx.request_repaint();
            handled = true;
        }
    }
    handled
}
