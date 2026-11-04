//! Cache invalidation signatures for the process table.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use crate::modules::memory::model::{MemorySnapshot, RawProcessRow};

/// Fast 64-bit signature of dynamic telemetry for cache invalidation (test fallback only).
pub fn compute_snapshot_signature(snap: &MemorySnapshot, uptime_secs: u64) -> u64 {
    let mut h = uptime_secs;
    h = h.wrapping_add(snap.ram_used_bytes.rotate_left(13));
    h = h.wrapping_add(snap.ram_available_bytes.rotate_left(29));
    h = h.wrapping_add(snap.swap_used_bytes.rotate_left(41));
    h = h.wrapping_add(snap.ram_cached_bytes().rotate_left(53));
    for row in snap.processes.iter() {
        if row.ram_bytes > 0 {
            let ratio = (row.delta_bytes_per_sec as i128 * 1_000_000) / (row.ram_bytes as i128);
            h = h.wrapping_add((ratio as u64).rotate_left(7));
        }
    }
    h
}

pub fn hash_search_query(query: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    query.hash(&mut hasher);
    hasher.finish()
}

/// Creates a minimal MemorySnapshot for cache key computation from raw rows.
pub fn make_signature_snapshot(raw_rows: &[RawProcessRow]) -> MemorySnapshot {
    MemorySnapshot {
        ram_total_bytes: 0,
        ram_used_bytes: 0,
        ram_available_bytes: 0,
        ram_free_bytes: 0,
        swap_total_bytes: 0,
        swap_used_bytes: 0,
        swap_free_bytes: 0,
        commit_total_bytes: 0,
        commit_limit_bytes: 0,
        kernel_paged_bytes: 0,
        kernel_nonpaged_bytes: 0,
        system_handles: 0,
        system_threads: 0,
        system_processes: 0,
        ram_cached_bytes_override: 0,
        kernel_metrics_valid: false,
        snapshot_id: 0,
        process_scan_id: 0,
        sample_elapsed_secs: 0.0,
        scan_duration_ms: 0.0,
        channel_overflows: 0,
        processes: Arc::new(raw_rows.to_vec()),
        latest_spike: None,
        has_heavy_process: false,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TableCacheKey {
    pub snapshot_id: u64,
    pub search_hash: u64,
    pub sort_col: crate::modules::memory::process_model::MemorySortCol,
    pub sort_asc: bool,
    pub group_clones: bool,
}

impl TableCacheKey {
    pub fn from_state(
        state: &crate::modules::memory::table_state::MemoryTableState,
        raw_rows: &[RawProcessRow],
    ) -> Self {
        let snap = make_signature_snapshot(raw_rows);
        Self {
            snapshot_id: compute_snapshot_signature(&snap, 0),
            search_hash: hash_search_query(&state.search_query),
            sort_col: state.sort_col,
            sort_asc: state.sort_asc,
            group_clones: state.group_clones,
        }
    }
}
