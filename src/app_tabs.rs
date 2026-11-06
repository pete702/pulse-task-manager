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
/// Returns true if a hotkey was handled and the app should skip normal update logic.
pub fn handle_global_hotkeys(
    ctx: &egui::Context,
    active_tab: &mut ActiveTab,
    memory_sub_view: &mut MemorySubView,
    table_state: &mut MemoryTableState,
    cpu_table_state: &mut CpuTableState,
) -> bool {
    let mut handled = false;
    ctx.input(|i| {
        // Ctrl+F: Focus search in the master Processes tab
        if i.modifiers.ctrl && i.key_pressed(egui::Key::F) {
            *active_tab = ActiveTab::Processes;
            *memory_sub_view = MemorySubView::Overview;
            cpu_table_state.focus_search_next_frame = true;
            table_state.focus_search_next_frame = true;
            ctx.request_repaint();
            handled = true;
        }
        // Escape: Clear search query if present, otherwise clear selection
        if i.key_pressed(egui::Key::Escape) {
            if !table_state.search_query.is_empty() {
                table_state.clear_search();
                ctx.request_repaint();
            } else if !cpu_table_state.search_query.is_empty() {
                cpu_table_state.clear_search();
                ctx.request_repaint();
            } else {
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
            }
            handled = true;
        }
        // Ctrl+1..Ctrl+6: switch tabs quickly
        if i.modifiers.ctrl {
            let tabs = [
                ActiveTab::Processes,
                ActiveTab::Memory,
                ActiveTab::Cpu,
                ActiveTab::Storage,
                ActiveTab::Network,
                ActiveTab::System,
            ];
            let keys = [
                egui::Key::Num1,
                egui::Key::Num2,
                egui::Key::Num3,
                egui::Key::Num4,
                egui::Key::Num5,
                egui::Key::Num6,
            ];
            for (idx, key) in keys.iter().enumerate() {
                if i.key_pressed(*key) {
                    *active_tab = tabs[idx];
                    ctx.request_repaint();
                    handled = true;
                    break;
                }
            }
        }
    });
    handled
}
