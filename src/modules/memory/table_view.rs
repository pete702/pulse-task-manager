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
#[derive(Debug, Clone, PartialEq, Eq)]
struct TableCacheKey {
    snapshot_id: u64,
    search_query: String,
    sort_col: MemorySortCol,
    sort_asc: bool,
    group_clones: bool,
}

/// UI state for the process memory table with 1 Hz caching.
#[derive(Debug, Clone)]
pub struct MemoryTableState {
    pub search_query: String,
    pub group_clones: bool,
    pub sort_col: MemorySortCol,
    pub sort_asc: bool,
    pub selected_pid: Option<u32>,
    cached_rows: Option<Arc<[crate::modules::memory::process_model::ProcessMemoryItem]>>,
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
fn compute_snapshot_signature(snap: &MemorySnapshot) -> u64 {
    let mut h = snap.ram_used_bytes;
    h = h.wrapping_add(snap.ram_available_bytes.rotate_left(13));
    h = h.wrapping_add(snap.swap_used_bytes.rotate_left(29));
    let cached = snap.ram_cached_bytes();
    h = h.wrapping_add(cached.rotate_left(41));
    for (_, _, ram, _, delta) in &snap.processes {
        if *ram > 0 {
            let ratio = (*delta as i128 * 1_000_000) / (*ram as i128);
            h = h.wrapping_add((ratio as u64).rotate_left(7));
        }
    }
    h
}

impl MemoryTableState {
    pub(crate) fn get_or_compute_rows(
        &mut self,
        snap: &MemorySnapshot,
    ) -> Arc<[crate::modules::memory::process_model::ProcessMemoryItem]> {
        let snapshot_id = compute_snapshot_signature(snap);
        let new_key = TableCacheKey {
            snapshot_id,
            search_query: self.search_query.clone(),
            sort_col: self.sort_col,
            sort_asc: self.sort_asc,
            group_clones: self.group_clones,
        };
        if self.cache_key.as_ref() == Some(&new_key) {
            if let Some(ref rows) = self.cached_rows {
                return Arc::clone(rows);
            }
        }
        let rows = filter_group_and_sort(
            &snap.processes,
            snap.ram_total_bytes,
            &self.search_query,
            self.group_clones,
            self.sort_col,
            self.sort_asc,
        );
        let rows_arc: Arc<[_]> = rows.into();
        self.cached_rows = Some(Arc::clone(&rows_arc));
        self.cache_key = Some(new_key);
        rows_arc
    }

    pub(crate) fn invalidate_cache(&mut self) {
        self.cached_rows = None;
        self.cache_key = None;
    }

    #[cfg(test)]
    pub fn has_cached_rows(&self) -> bool {
        self.cached_rows.is_some()
    }
}

/// Renders the spike banner, toolbar, and virtualized process table.
pub fn render_memory_table(ui: &mut egui::Ui, snap: &MemorySnapshot, state: &mut MemoryTableState) {
    let rows = state.get_or_compute_rows(snap);
    let clone_groups = rows.iter().filter(|r| r.process_count > 1).count();
    let total_processes = snap.processes.len();
    let shown_rows = rows.len();

    crate::theme::card_frame().show(ui, |ui| {
        ui.vertical(|ui| {
            if let Some(ref spike) = snap.latest_spike {
                render_spike_banner(ui, spike);
                ui.add_space(8.0);
            }
            let (old_sort_col, old_sort_asc, old_group_clones) =
                (state.sort_col, state.sort_asc, state.group_clones);
            render_toolbar(ui, total_processes, shown_rows, clone_groups, state);
            ui.add_space(6.0);
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
