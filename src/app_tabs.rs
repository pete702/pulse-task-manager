//! Tab definitions for PULSE application.

use crate::i18n::Key;
use crate::modules::cpu::table_state::CpuTableState;
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

    /// All six tabs in display order (used by the compact tab bar).
    pub fn all_ordered() -> Vec<ActiveTab> {
        std::iter::once(ActiveTab::Processes).chain(Self::all().iter().copied()).collect()
    }

    /// Clean ASCII label without navigation brackets.
    pub fn short_label(self) -> &'static str {
        match self {
            ActiveTab::Processes => "Processes",
            ActiveTab::Memory => "Memory",
            ActiveTab::Cpu => "CPU",
            ActiveTab::Storage => "Storage",
            ActiveTab::Network => "Network",
            ActiveTab::System => "System & GPU",
        }
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

/// Applies Escape to the active tab only. Returns true if persisted/filter state changed.
pub(crate) fn apply_escape(active_tab: ActiveTab, cpu_table_state: &mut CpuTableState) -> bool {
    match active_tab {
        ActiveTab::Cpu | ActiveTab::Processes => {
            if !cpu_table_state.search_query.is_empty() {
                cpu_table_state.search_query.clear();
                cpu_table_state.mark_dirty();
                true
            } else if cpu_table_state.selected_pid.is_some()
                || cpu_table_state.selected_name.is_some()
                || cpu_table_state.confirm_kill.is_some()
            {
                cpu_table_state.selected_pid = None;
                cpu_table_state.selected_start_time = None;
                cpu_table_state.selected_name = None;
                cpu_table_state.confirm_kill = None;
                cpu_table_state.mark_dirty();
                true
            } else {
                false
            }
        }
        ActiveTab::Memory | ActiveTab::Storage | ActiveTab::Network | ActiveTab::System => false,
    }
}

/// Handles global hotkeys for the application.
/// Returns true if a hotkey was handled and the UI should repaint immediately.
pub fn handle_global_hotkeys(
    ctx: &egui::Context,
    active_tab: &mut ActiveTab,
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
        crate::core::worker_loop::set_worker_paused(*telemetry_paused);
        if !*telemetry_paused {
            crate::core::request_process_scan();
        }
        handled = true;
    }
    // Ctrl+F: Focus search in the master Processes tab
    if ctrl_f && !wants_input {
        *active_tab = ActiveTab::Processes;
        cpu_table_state.focus_search_next_frame = true;
        ctx.request_repaint();
        handled = true;
    }
    // Escape: tab-scoped clear search or selection only for active tab.
    // The command palette consumes Escape itself when open.
    if esc && !crate::ui::command_palette::is_open() {
        if matches!(*active_tab, ActiveTab::Cpu | ActiveTab::Processes) {
            ctx.memory_mut(|m| {
                let id = egui::Id::new("cpu_search_filter");
                if m.focused() == Some(id) {
                    m.surrender_focus(id);
                }
            });
        }
        if apply_escape(*active_tab, cpu_table_state) {
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

/// Ctrl+B toggles the sidebar; Ctrl+Tab / Ctrl+Shift+Tab cycle tabs.
/// Ignored while a text field wants keyboard input.
pub fn handle_navigation_hotkeys(
    ctx: &egui::Context,
    active_tab: &mut ActiveTab,
    sidebar_hidden: &mut bool,
) -> bool {
    if ctx.wants_keyboard_input() {
        return false;
    }
    let (ctrl, shift, toggle, tab_key) = ctx.input(|i| {
        (
            i.modifiers.ctrl,
            i.modifiers.shift,
            i.key_pressed(egui::Key::B),
            i.key_pressed(egui::Key::Tab),
        )
    });
    let mut handled = false;
    if ctrl && toggle {
        *sidebar_hidden = !*sidebar_hidden;
        handled = true;
    }
    if ctrl && tab_key {
        let order = ActiveTab::all_ordered();
        let idx = order.iter().position(|t| *t == *active_tab).unwrap_or(0);
        let step = if shift { order.len() - 1 } else { 1 };
        *active_tab = order[(idx + step) % order.len()];
        handled = true;
    }
    handled
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escape_is_scoped_to_active_tab() {
        let mut cpu = CpuTableState::default();
        cpu.search_query = "cpuq".to_owned();
        cpu.selected_pid = Some(22);
        cpu.selected_name = Some("proc".to_owned());

        assert!(apply_escape(ActiveTab::Cpu, &mut cpu));
        assert!(cpu.search_query.is_empty());

        cpu.search_query = "cpuq".to_owned();
        assert!(!apply_escape(ActiveTab::Storage, &mut cpu));
        assert_eq!(cpu.search_query, "cpuq");
    }
}
