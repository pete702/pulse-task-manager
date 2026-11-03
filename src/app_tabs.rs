//! Tab definitions for PULSE application.

use crate::i18n::Key;
use crate::modules::memory::page::MemorySubView;
use crate::modules::memory::table_view::MemoryTableState;
use eframe::egui;

/// Active tab in the left sidebar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveTab {
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
    /// Returns all available tabs.
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
) -> bool {
    let mut handled = false;
    ctx.input(|i| {
        // Ctrl+F: Focus search in Memory > Processes
        if i.modifiers.ctrl && i.key_pressed(egui::Key::F) {
            *active_tab = ActiveTab::Memory;
            *memory_sub_view = MemorySubView::Processes;
            table_state.search_query.clear();
            handled = true;
        }
        // Escape: Clear search query
        if i.key_pressed(egui::Key::Escape) && !table_state.search_query.is_empty() {
            table_state.search_query.clear();
            handled = true;
        }
    });
    handled
}
