//! Unit tests for table view (process table with 1 Hz cache).

#[cfg(test)]
mod tests {
    use crate::modules::memory::model::MemorySnapshot;
    use crate::modules::memory::table_view::MemoryTableState;

    fn row(
        pid: u32,
        name: &str,
        ram: u64,
        virt: u64,
        delta: i64,
    ) -> (u32, String, u64, u64, i64, Option<String>) {
        (pid, name.to_owned(), ram, virt, delta, None)
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
            processes: vec![
                row(100, "chrome.exe", 1_000_000_000, 2_000_000_000, 0),
                row(200, "game.exe", 3_000_000_000, 4_000_000_000, 0),
            ],
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
            processes: vec![
                row(100, "chrome.exe", 1_000_000_000, 2_000_000_000, 0),
                row(200, "game.exe", 3_000_000_000, 4_000_000_000, 0),
            ],
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
            processes: vec![
                row(100, "chrome.exe", 1_000_000_000, 2_000_000_000, 0),
                row(200, "game.exe", 3_000_000_000, 4_000_000_000, 0),
            ],
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
            processes: vec![
                row(100, "chrome.exe", 1_000_000_000, 2_000_000_000, 50_000_000),
                row(200, "game.exe", 3_000_000_000, 4_000_000_000, 0),
            ],
            ..Default::default()
        };

        let rows2 = state.get_or_compute_rows(&snap2, 1);
        assert_eq!(rows2.len(), 2);
        let chrome_row = rows2.iter().find(|r| r.name == "chrome.exe").unwrap();
        assert_eq!(chrome_row.delta_bytes_per_sec, 50_000_000);
    }
}
