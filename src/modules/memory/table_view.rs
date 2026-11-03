//! Virtualized process memory table with search, clone grouping, sorting, and selection.
//! Features 1 Hz caching to avoid re-sorting on every UI frame.

use eframe::egui;

use crate::modules::memory::model::MemorySnapshot;
use crate::modules::memory::process_model::{filter_group_and_sort, MemorySortCol};
use crate::theme::colors;

use super::table_body::render_table;
use super::table_widgets::{empty_label, render_spike_banner, render_toolbar};

/// Cache key for 1 Hz process table optimization.
/// Recompute only when snapshot changes, search query changes, or sort/group settings change.
#[derive(Debug, Clone, PartialEq, Eq)]
struct TableCacheKey {
    snapshot_id: u64, // Unique ID per snapshot (e.g., uptime or sequence)
    search_query: String,
    sort_col: MemorySortCol,
    sort_asc: bool,
    group_clones: bool,
}

/// UI state for the process memory table with 1 Hz caching.
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
    /// Cached filtered/sorted rows.
    cached_rows: Option<Vec<crate::modules::memory::process_model::ProcessMemoryItem>>,
    /// Cache key for validation.
    cache_key: Option<TableCacheKey>,
}

impl Default for MemoryTableState {
    fn default() -> Self {
        Self {
            search_query: String::new(),
            group_clones: true,
            sort_col: MemorySortCol::RamBytes,
            sort_asc: false,
            selected_pid: None,
            cached_rows: None,
            cache_key: None,
        }
    }
}

impl MemoryTableState {
    /// Get or compute the filtered/sorted rows using 1 Hz cache.
    /// Returns an owned Vec to avoid borrow conflicts during rendering.
    pub(crate) fn get_or_compute_rows(
        &mut self,
        snap: &MemorySnapshot,
    ) -> Vec<crate::modules::memory::process_model::ProcessMemoryItem> {
        // Use snapshot uptime + process count as a simple snapshot ID
        let snapshot_id = snap
            .ram_total_bytes
            .wrapping_add(snap.processes.len() as u64);

        let new_key = TableCacheKey {
            snapshot_id,
            search_query: self.search_query.clone(),
            sort_col: self.sort_col,
            sort_asc: self.sort_asc,
            group_clones: self.group_clones,
        };

        // Check if cache is valid
        let cache_valid = self.cache_key.as_ref() == Some(&new_key);

        if cache_valid {
            if let Some(ref rows) = self.cached_rows {
                return rows.clone();
            }
        }

        // Recompute
        let rows = filter_group_and_sort(
            &snap.processes,
            snap.ram_total_bytes,
            &self.search_query,
            self.group_clones,
            self.sort_col,
            self.sort_asc,
        );

        self.cached_rows = Some(rows.clone());
        self.cache_key = Some(new_key);
        rows
    }

    /// Invalidate cache when sort/group settings change.
    pub(crate) fn invalidate_cache(&mut self) {
        self.cached_rows = None;
        self.cache_key = None;
    }

    /// Returns true if cache is populated (for testing).
    #[cfg(test)]
    pub fn has_cached_rows(&self) -> bool {
        self.cached_rows.is_some()
    }
}

/// Renders the spike banner, toolbar, and virtualized process table.
pub fn render_memory_table(ui: &mut egui::Ui, snap: &MemorySnapshot, state: &mut MemoryTableState) {
    // Get rows (may recompute cache) - returns owned Vec, no borrow conflict
    let rows = state.get_or_compute_rows(snap);
    let clone_groups = rows.iter().filter(|row| row.process_count > 1).count();
    let total_processes = snap.processes.len();
    let shown_rows = rows.len();

    crate::theme::card_frame().show(ui, |ui| {
        ui.vertical(|ui| {
            if let Some(ref spike) = snap.latest_spike {
                render_spike_banner(ui, spike);
                ui.add_space(8.0);
            }

            // Capture sort/group state before toolbar (to detect changes for next frame)
            let old_sort_col = state.sort_col;
            let old_sort_asc = state.sort_asc;
            let old_group_clones = state.group_clones;

            // Toolbar mutates state (search, group, sort)
            render_toolbar(ui, total_processes, shown_rows, clone_groups, state);
            ui.add_space(6.0);

            // Invalidate cache if sort/group changed
            if state.sort_col != old_sort_col
                || state.sort_asc != old_sort_asc
                || state.group_clones != old_group_clones
            {
                state.invalidate_cache();
            }

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
}
