//! Table state management with O(1) cache invalidation via process_scan_id.

use std::sync::Arc;

use crate::modules::memory::model::{MemorySnapshot, RawProcessRow};
use crate::modules::memory::process_model::{
    filter_group_and_sort, MemorySortCol, ProcessMemoryItem,
};

use super::table_cache::{compute_snapshot_signature, hash_search_query, TableCacheKey};

pub use super::table_selection::SelectedGroupSummary;

/// UI state for the process memory table with 1 Hz caching.
#[derive(Debug, Clone)]
pub struct MemoryTableState {
    pub search_query: String,
    pub group_clones: bool,
    pub sort_col: MemorySortCol,
    pub sort_asc: bool,
    pub selected_pid: Option<u32>,
    pub selected_group: Option<String>,
    cached_rows: Option<Arc<[ProcessMemoryItem]>>,
    cache_key: Option<TableCacheKey>,
    cached_scan_id: u64,
    /// Selection cache keyed by `(scan_id, group, pid, grouping_mode)`.
    pub(crate) cached_selection: Option<(u64, String, Option<u32>, bool, SelectedGroupSummary)>,
    /// Set when the grouping mode changed; clears stale action-bar kill confirmation.
    pub(crate) group_clones_toggled: bool,
}

impl Default for MemoryTableState {
    fn default() -> Self {
        Self {
            search_query: String::new(),
            group_clones: true,
            sort_col: MemorySortCol::RamBytes,
            sort_asc: false,
            selected_pid: None,
            selected_group: None,
            cached_rows: None,
            cache_key: None,
            cached_scan_id: 0,
            cached_selection: None,
            group_clones_toggled: false,
        }
    }
}

impl MemoryTableState {
    /// Rebuilds the table rows for the authoritative `process_scan_id` from the snapshot.
    /// Returns immediately if the scan_id matches and filter/sort/group settings are unchanged.
    pub fn rebuild_for_scan(&mut self, scan_id: u64, raw_rows: &[RawProcessRow]) {
        if scan_id != 0 && self.cached_scan_id == scan_id {
            return;
        }
        let new_key = TableCacheKey::from_state(self, raw_rows);
        if self.cache_key.as_ref() == Some(&new_key) {
            self.cached_scan_id = scan_id;
            return;
        }
        let rows: Arc<[ProcessMemoryItem]> = filter_group_and_sort(
            raw_rows,
            0,
            &self.search_query,
            self.group_clones,
            self.sort_col,
            self.sort_asc,
        )
        .into();
        self.cached_rows = Some(Arc::clone(&rows));
        self.cache_key = Some(new_key);
        self.cached_scan_id = scan_id;
        self.cached_selection = None;
    }

    /// Test-only fallback that uses the legacy signature-based invalidation.
    #[cfg(test)]
    pub fn rebuild_if_needed(&mut self, raw_rows: &[RawProcessRow]) {
        self.rebuild_for_scan(0, raw_rows);
    }

    pub(crate) fn get_or_compute_rows(
        &mut self,
        snap: &MemorySnapshot,
        uptime_secs: u64,
    ) -> Arc<[ProcessMemoryItem]> {
        let new_key = TableCacheKey {
            snapshot_id: compute_snapshot_signature(snap, uptime_secs),
            search_hash: hash_search_query(&self.search_query),
            sort_col: self.sort_col,
            sort_asc: self.sort_asc,
            group_clones: self.group_clones,
        };
        if self.cache_key.as_ref() == Some(&new_key) {
            if let Some(ref rows) = self.cached_rows {
                return Arc::clone(rows);
            }
        }
        let rows: Arc<[ProcessMemoryItem]> = filter_group_and_sort(
            &snap.processes,
            snap.ram_total_bytes,
            &self.search_query,
            self.group_clones,
            self.sort_col,
            self.sort_asc,
        )
        .into();
        self.cached_rows = Some(Arc::clone(&rows));
        self.cache_key = Some(new_key);
        rows
    }

    pub(crate) fn invalidate_cache(&mut self) {
        self.cached_rows = None;
        self.cache_key = None;
        self.cached_scan_id = 0;
        self.cached_selection = None;
    }

    #[cfg(test)]
    pub fn has_cached_rows(&self) -> bool {
        self.cached_rows.is_some()
    }
    #[cfg(test)]
    pub fn cached_scan_id(&self) -> u64 {
        self.cached_scan_id
    }
    #[cfg(test)]
    pub fn cached_rows(&self) -> Option<&Arc<[ProcessMemoryItem]>> {
        self.cached_rows.as_ref()
    }
}
