//! Virtualized process memory table with search, clone grouping, sorting, and selection.

use eframe::egui;

use crate::modules::memory::model::MemorySnapshot;
use crate::modules::memory::process_model::{filter_group_and_sort, MemorySortCol};
use crate::theme::colors;

use super::table_body::render_table;
use super::table_widgets::{empty_label, render_spike_banner, render_toolbar};

/// UI state for the process memory table.
#[derive(Debug, Clone)]
pub struct MemoryTableState {
    /// Live search query (process name or PID).
    pub search_query: String,
    /// Whether processes with the same name are grouped.
    pub group_clones: bool,
    /// Active sort column.
    pub sort_col: MemorySortCol,
    /// Ascending sort order.
    pub sort_asc: bool,
    /// Selected process (representative PID).
    pub selected_pid: Option<u32>,
}

impl Default for MemoryTableState {
    fn default() -> Self {
        Self {
            search_query: String::new(),
            group_clones: true,
            sort_col: MemorySortCol::RamBytes,
            sort_asc: false,
            selected_pid: None,
        }
    }
}

/// Renders the spike banner, toolbar, and virtualized process table.
pub fn render_memory_table(ui: &mut egui::Ui, snap: &MemorySnapshot, state: &mut MemoryTableState) {
    let rows = filter_group_and_sort(
        &snap.processes,
        snap.ram_total_bytes,
        &state.search_query,
        state.group_clones,
        state.sort_col,
        state.sort_asc,
    );
    let clone_groups = rows.iter().filter(|row| row.process_count > 1).count();

    crate::theme::card_frame().show(ui, |ui| {
        ui.vertical(|ui| {
            if let Some(ref spike) = snap.latest_spike {
                render_spike_banner(ui, spike);
                ui.add_space(8.0);
            }
            render_toolbar(ui, snap.processes.len(), rows.len(), clone_groups, state);
            ui.add_space(6.0);

            if rows.is_empty() {
                ui.centered_and_justified(|ui| {
                    ui.label(
                        egui::RichText::new(empty_label())
                            .monospace()
                            .size(12.0)
                            .color(colors::TEXT_SECONDARY),
                    );
                });
            } else {
                let height = (ui.available_height() - 4.0).max(120.0);
                render_table(ui, &rows, state, height);
            }
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_table_state_defaults() {
        let state = MemoryTableState::default();
        assert!(state.search_query.is_empty());
        assert!(state.group_clones);
        assert_eq!(state.sort_col, MemorySortCol::RamBytes);
        assert!(!state.sort_asc);
        assert!(state.selected_pid.is_none());
    }

    #[test]
    fn test_render_memory_table_no_panic() {
        use crate::modules::memory::process_model::MemorySpikeEvent;

        let snap = MemorySnapshot {
            ram_total_bytes: 16_000_000_000,
            processes: vec![
                (100, "chrome.exe".to_owned(), 1_000_000_000, 2_000_000_000),
                (200, "game.exe".to_owned(), 3_000_000_000, 4_000_000_000),
            ],
            latest_spike: Some(MemorySpikeEvent {
                timestamp: "2026-11-02 12:00:00".to_owned(),
                peak_ram_pct: 87.5,
                culprit_name: "game.exe".to_owned(),
                culprit_pid: 200,
                culprit_bytes: 3_000_000_000,
            }),
            ..Default::default()
        };
        let mut state = MemoryTableState::default();

        egui::__run_test_ui(|ui| {
            render_memory_table(ui, &snap, &mut state);
        });
    }
}
