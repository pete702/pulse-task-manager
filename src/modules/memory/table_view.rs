//! Virtualized process memory table with search, clone grouping, sorting, and selection.
//! Features 1 Hz caching to avoid re-sorting on every UI frame.

use eframe::egui;
use std::sync::Arc;

use crate::modules::memory::model::MemorySnapshot;
use crate::modules::memory::process_model::{filter_group_and_sort, MemorySortCol};
use crate::theme::colors;

use super::table_body::render_table;
use super::table_widgets::{empty_label, render_spike_banner, render_toolbar};

/// Cache key for 1 Hz process table optimization.
/// Recompute only when snapshot changes, search query changes, or sort/group settings change.
#[derive(Debug, Clone, PartialEq, Eq)]
struct TableCacheKey {
    snapshot_id: u64, // Fast 64-bit signature of dynamic telemetry
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
    /// Cached filtered/sorted rows (Arc for zero-copy per-frame access).
    cached_rows: Option<Arc<[crate::modules::memory::process_model::ProcessMemoryItem]>>,
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

/// Computes a fast 64-bit signature of dynamic telemetry for cache invalidation.
/// Combines RAM/swap metrics with a rolling hash of per-process deltas.
fn compute_snapshot_signature(snap: &MemorySnapshot) -> u64 {
    let mut h = snap.ram_used_bytes;
    h = h.wrapping_add(snap.ram_available_bytes.rotate_left(13));
    h = h.wrapping_add(snap.swap_used_bytes.rotate_left(29));
    let cached = snap.ram_cached_bytes();
    h = h.wrapping_add(cached.rotate_left(41));

    // Rolling hash of delta_bytes_per_sec / used_bytes across processes
    for (_, _, ram, _, delta) in &snap.processes {
        if *ram > 0 {
            let ratio = (*delta as i128 * 1_000_000) / (*ram as i128); // scaled ratio
            h = h.wrapping_add((ratio as u64).rotate_left(7));
        }
    }
    h
}

impl MemoryTableState {
    /// Get or compute the filtered/sorted rows using 1 Hz cache.
    /// Returns an Arc<[ProcessMemoryItem]> for zero-copy per-frame access.
    pub(crate) fn get_or_compute_rows(
        &mut self,
        snap: &MemorySnapshot,
    ) -> Arc<[crate::modules::memory::process_model::ProcessMemoryItem]> {
        // Fast 64-bit signature of dynamic telemetry - changes every 1 Hz tick
        let snapshot_id = compute_snapshot_signature(snap);

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
                return Arc::clone(rows);
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

        let rows_arc: Arc<[crate::modules::memory::process_model::ProcessMemoryItem]> = rows.into();
        self.cached_rows = Some(Arc::clone(&rows_arc));
        self.cache_key = Some(new_key);
        rows_arc
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
