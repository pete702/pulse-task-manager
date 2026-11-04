//! Table state management with O(1) cache invalidation via process_scan_id.

use std::sync::Arc;

use crate::modules::memory::model::RawProcessRow;
use crate::modules::memory::process_model::{
    filter_group_and_sort, MemorySortCol, ProcessMemoryItem,
};
use crate::modules::memory::table_selection::SelectedGroupSummary;

use super::table_cache::TableCacheKey;

/// Type alias for the selection cache tuple to reduce clippy::type_complexity.
type SelectionCacheEntry = (u64, String, Option<u32>, bool, Arc<SelectedGroupSummary>);

/// UI state for the process memory table with 1 Hz caching.
#[derive(Debug, Clone)]
pub struct MemoryTableState {
    pub search_query: String,
    pub group_clones: bool,
    pub sort_col: MemorySortCol,
    pub sort_asc: bool,
    pub selected_pid: Option<u32>,
    pub selected_group: Option<String>,
    /// Stored process creation time for single-PID selection identity verification.
    /// When `group_clones == false`, this must match the live process to prevent PID reuse attacks.
    pub(crate) selected_creation_time: Option<u64>,
    /// Stored process name for single-PID selection identity verification.
    pub(crate) selected_name: Option<String>,
    cached_rows: Option<Arc<[ProcessMemoryItem]>>,
    cache_key: Option<TableCacheKey>,
    cached_scan_id: u64,
    /// Selection cache keyed by `(scan_id, group, pid, grouping_mode)`.
    /// Uses `Arc` for O(1) clone on cache hits.
    pub(crate) cached_selection: Option<SelectionCacheEntry>,
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
            selected_creation_time: None,
            selected_name: None,
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
    pub fn rebuild_for_scan(
        &mut self,
        scan_id: u64,
        raw_rows: &[RawProcessRow],
        ram_total_bytes: u64,
    ) {
        if scan_id != 0 && self.cached_scan_id == scan_id {
            return;
        }
        let new_key = TableCacheKey::from_state(self, raw_rows, ram_total_bytes, scan_id);
        if self.cache_key.as_ref() == Some(&new_key) {
            self.cached_scan_id = scan_id;
            return;
        }
        let rows: Arc<[ProcessMemoryItem]> = filter_group_and_sort(
            raw_rows,
            ram_total_bytes,
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
    #[allow(dead_code)]
    pub fn rebuild_if_needed(&mut self, raw_rows: &[RawProcessRow]) {
        self.rebuild_for_scan(0, raw_rows, 0);
    }

    /// Returns the cached rows for the current scan, or an empty slice if not yet built.
    /// The authoritative path is `rebuild_for_scan`; this method assumes it was called first.
    pub(crate) fn cached_rows(&self) -> Option<Arc<[ProcessMemoryItem]>> {
        self.cached_rows.clone()
    }

    pub(crate) fn invalidate_cache(&mut self) {
        self.cached_rows = None;
        self.cache_key = None;
        self.cached_scan_id = 0;
        self.cached_selection = None;
    }

    /// Clears all selection state including single-PID identity verification fields.
    pub(crate) fn clear_selection_state(&mut self) {
        self.selected_pid = None;
        self.selected_group = None;
        self.selected_creation_time = None;
        self.selected_name = None;
        self.cached_selection = None;
        self.group_clones_toggled = false;
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
    #[allow(dead_code)]
    pub fn cached_rows_ref(&self) -> Option<&Arc<[ProcessMemoryItem]>> {
        self.cached_rows.as_ref()
    }
}
