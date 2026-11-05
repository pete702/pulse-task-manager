//! Cache invalidation signatures for the process table.

use crate::modules::memory::model::RawProcessRow;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TableCacheKey {
    /// Authoritative cache key: process_scan_id + settings + ram_total_bytes + search_query.
    /// When process_scan_id != 0, this is the sole invalidation signal.
    pub process_scan_id: u64,
    pub ram_total_bytes: u64,
    pub search_query: String,
    pub sort_col: crate::modules::memory::process_model::MemorySortCol,
    pub sort_asc: bool,
    pub group_clones: bool,
}

impl TableCacheKey {
    /// Creates a cache key from the authoritative process_scan_id and current settings.
    /// Pass `ram_total_bytes` so that ram_pct is always accurate on the first pass.
    pub fn from_state(
        state: &crate::modules::memory::table_state::MemoryTableState,
        _raw_rows: &[RawProcessRow],
        ram_total_bytes: u64,
        process_scan_id: u64,
    ) -> Self {
        Self {
            process_scan_id,
            ram_total_bytes,
            search_query: state.search_query.clone(),
            sort_col: state.sort_col,
            sort_asc: state.sort_asc,
            group_clones: state.group_clones,
        }
    }
}
