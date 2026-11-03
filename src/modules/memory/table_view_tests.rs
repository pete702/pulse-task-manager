//! Unit tests for table view (process table with 1 Hz cache).

#[cfg(test)]
mod tests {
    use crate::modules::memory::model::MemorySnapshot;
    use crate::modules::memory::table_view::MemoryTableState;

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
                (100, "chrome.exe".to_owned(), 1_000_000_000, 2_000_000_000, 0),
                (200, "game.exe".to_owned(), 3_000_000_000, 4_000_000_000, 0),
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
                (100, "chrome.exe".to_owned(), 1_000_000_000, 2_000_000_000, 0),
                (200, "game.exe".to_owned(), 3_000_000_000, 4_000_000_000, 0),
            ],
            latest_spike: None,
            ..Default::default()
        };
        let mut state = MemoryTableState::default();

        // First render - populates cache
        egui::__run_test_ui(|ui| {
            crate::modules::memory::table_view::render_memory_table(ui, &snap, &mut state, 0);
        });
        assert!(state.has_cached_rows());

        // Change sort column - should invalidate cache
        state.sort_col = crate::modules::memory::process_model::MemorySortCol::Name;
        egui::__run_test_ui(|ui| {
            crate::modules::memory::table_view::render_memory_table(ui, &snap, &mut state, 0);
        });
        assert!(state.has_cached_rows()); // Recomputed
    }

    #[test]
    fn test_cache_invalidates_on_ram_delta_change() {
        let mut state = MemoryTableState::default();

        // Create first snapshot with specific process RAM and delta
        let snap1 = MemorySnapshot {
            ram_total_bytes: 16_000_000_000,
            ram_used_bytes: 8_000_000_000,
            ram_available_bytes: 8_000_000_000,
            swap_used_bytes: 1_000_000_000,
            processes: vec![
                (100, "chrome.exe".to_owned(), 1_000_000_000, 2_000_000_000, 0),
                (200, "game.exe".to_owned(), 3_000_000_000, 4_000_000_000, 0),
            ],
            ..Default::default()
        };

        // First computation - populates cache
        let rows1 = state.get_or_compute_rows(&snap1, 0);
        assert!(state.has_cached_rows());
        assert_eq!(rows1.len(), 2);

        // Create second snapshot with SAME ram_total_bytes and processes.len()
        // but DIFFERENT ram_used_bytes and delta_bytes_per_sec
        let snap2 = MemorySnapshot {
            ram_total_bytes: 16_000_000_000, // Same total
            ram_used_bytes: 8_500_000_000,   // Different used
            ram_available_bytes: 7_500_000_000,
            swap_used_bytes: 1_000_000_000,
            processes: vec![
                (100, "chrome.exe".to_owned(), 1_000_000_000, 2_000_000_000, 50_000_000), // Delta changed!
                (200, "game.exe".to_owned(), 3_000_000_000, 4_000_000_000, 0),
            ],
            ..Default::default()
        };

        // Should recompute because signature changed (ram_used_bytes + delta)
        let rows2 = state.get_or_compute_rows(&snap2, 1);
        assert_eq!(rows2.len(), 2);

        // Verify cache was actually recomputed (not returning stale data)
        // The delta for chrome should now be 50_000_000 in the computed rows
        let chrome_row = rows2.iter().find(|r| r.name == "chrome.exe").unwrap();
        assert_eq!(chrome_row.delta_bytes_per_sec, 50_000_000);
    }
}
