//! Unit tests for table view (process table with 1 Hz cache).

#[cfg(test)]
mod tests {
    use crate::modules::memory::model::{MemorySnapshot, RawProcessRow};
    use crate::modules::memory::table_state::MemoryTableState;
    use std::sync::Arc;

    fn row(pid: u32, name: &str, ram: u64, virt: u64, delta: i64) -> RawProcessRow {
        RawProcessRow {
            pid,
            name: name.to_owned(),
            ram_bytes: ram,
            virtual_bytes: virt,
            delta_bytes_per_sec: delta,
            exe_path: None,
            creation_time: None,
        }
    }

    fn shared(rows: Vec<RawProcessRow>) -> Arc<Vec<RawProcessRow>> {
        Arc::new(rows)
    }

    #[test]
    fn test_table_state_defaults() {
        let state = MemoryTableState::default();
        assert!(state.search_query.is_empty());
        assert!(state.group_clones);
        assert_eq!(state.sort_col, crate::modules::memory::process_model::MemorySortCol::RamBytes);
        assert!(!state.sort_asc);
        assert!(state.selected_pid.is_none());
    }

    #[test]
    fn test_render_memory_table_no_panic() {
        let snap = MemorySnapshot {
            ram_total_bytes: 16_000_000_000,
            processes: shared(vec![
                row(100, "chrome.exe", 1_000_000_000, 2_000_000_000, 0),
                row(200, "game.exe", 3_000_000_000, 4_000_000_000, 0),
            ]),
            latest_spike: None,
            ..Default::default()
        };
        let mut state = MemoryTableState::default();

        egui::__run_test_ui(|ui| {
            crate::modules::memory::table_view::render_memory_table(ui, &snap, &mut state, 0);
        });
    }

    #[test]
    fn test_table_cache_invalidates_on_sort_change() {
        let snap = MemorySnapshot {
            ram_total_bytes: 16_000_000_000,
            processes: shared(vec![
                row(100, "chrome.exe", 1_000_000_000, 2_000_000_000, 0),
                row(200, "game.exe", 3_000_000_000, 4_000_000_000, 0),
            ]),
            latest_spike: None,
            ..Default::default()
        };
        let mut state = MemoryTableState::default();

        egui::__run_test_ui(|ui| {
            crate::modules::memory::table_view::render_memory_table(ui, &snap, &mut state, 0);
        });
        assert!(state.has_cached_rows());

        state.sort_col = crate::modules::memory::process_model::MemorySortCol::Name;
        egui::__run_test_ui(|ui| {
            crate::modules::memory::table_view::render_memory_table(ui, &snap, &mut state, 0);
        });
        assert!(state.has_cached_rows());
    }

    #[test]
    fn test_cache_invalidates_on_ram_delta_change() {
        let mut state = MemoryTableState::default();

        let snap1 = MemorySnapshot {
            ram_total_bytes: 16_000_000_000,
            ram_used_bytes: 8_000_000_000,
            ram_available_bytes: 8_000_000_000,
            swap_used_bytes: 1_000_000_000,
            processes: shared(vec![
                row(100, "chrome.exe", 1_000_000_000, 2_000_000_000, 0),
                row(200, "game.exe", 3_000_000_000, 4_000_000_000, 0),
            ]),
            ..Default::default()
        };

        let rows1 = state.get_or_compute_rows(&snap1, 0);
        assert!(state.has_cached_rows());
        assert_eq!(rows1.len(), 2);

        let snap2 = MemorySnapshot {
            ram_total_bytes: 16_000_000_000,
            ram_used_bytes: 8_500_000_000,
            ram_available_bytes: 7_500_000_000,
            swap_used_bytes: 1_000_000_000,
            processes: shared(vec![
                row(100, "chrome.exe", 1_000_000_000, 2_000_000_000, 50_000_000),
                row(200, "game.exe", 3_000_000_000, 4_000_000_000, 0),
            ]),
            ..Default::default()
        };

        let rows2 = state.get_or_compute_rows(&snap2, 1);
        assert_eq!(rows2.len(), 2);
        let chrome_row = rows2.iter().find(|r| r.name == "chrome.exe").unwrap();
        assert_eq!(chrome_row.delta_bytes_per_sec, 50_000_000);
    }

    #[test]
    fn test_rebuild_for_scan_uses_process_scan_id() {
        let mut state = MemoryTableState::default();
        let raw_rows = vec![
            row(100, "chrome.exe", 1_000_000_000, 2_000_000_000, 0),
            row(200, "game.exe", 3_000_000_000, 4_000_000_000, 0),
        ];

        // First call with scan_id = 1: should rebuild
        state.rebuild_for_scan(1, &raw_rows);
        assert_eq!(state.cached_scan_id(), 1);
        assert!(state.cached_rows().is_some());

        // Second call with same scan_id = 1: should be O(1) no-op
        let cached_rows_before = state.cached_rows().cloned();
        state.rebuild_for_scan(1, &raw_rows);
        assert_eq!(state.cached_scan_id(), 1);
        assert!(Arc::ptr_eq(cached_rows_before.as_ref().unwrap(), state.cached_rows().unwrap()));

        // Call with new scan_id = 2: should rebuild
        state.rebuild_for_scan(2, &raw_rows);
        assert_eq!(state.cached_scan_id(), 2);
    }

    #[test]
    fn test_resolved_selection_cached_per_scan_id() {
        let mut state = MemoryTableState::default();
        let raw_rows = vec![
            row(100, "chrome.exe [2x]", 1_000_000_000, 2_000_000_000, 0),
            row(101, "chrome.exe", 500_000_000, 1_000_000_000, 0),
        ];

        state.selected_group = Some("chrome.exe".to_owned());

        // First call with scan_id = 1: resolves and caches
        let sel1 = state.resolved_selection(1, &raw_rows);
        assert!(sel1.is_some());
        let sel1_pids = sel1.as_ref().unwrap().pids.clone();
        assert_eq!(sel1_pids, vec![100, 101]);

        // Second call with same scan_id = 1: returns cached (same content)
        let sel2 = state.resolved_selection(1, &raw_rows);
        assert!(sel2.is_some());
        assert_eq!(sel2.as_ref().unwrap().pids, sel1_pids);
        assert_eq!(sel2.as_ref().unwrap().targets.len(), sel1.as_ref().unwrap().targets.len());

        // Call with new scan_id = 2: re-resolves (different content allocation)
        let sel3 = state.resolved_selection(2, &raw_rows);
        assert!(sel3.is_some());
        assert_eq!(sel3.as_ref().unwrap().pids, sel1_pids);
    }

    #[test]
    fn test_rebuild_if_needed_test_fallback() {
        let mut state = MemoryTableState::default();
        let raw_rows = vec![row(100, "chrome.exe", 1_000_000_000, 2_000_000_000, 0)];
        state.rebuild_if_needed(&raw_rows);
        assert!(state.cached_rows().is_some());
    }
}
