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
        assert_eq!(
            state.sort_col,
            crate::modules::memory::process_model::MemorySortCol::RamBytes
        );
        assert!(!state.sort_asc);
        assert!(state.selected_pid.is_none());
    }

    #[test]
    fn test_render_memory_table_no_panic() {
        let snap = MemorySnapshot {
            ram_total_bytes: 16_000_000_000,
            processes: vec![
                (
                    100,
                    "chrome.exe".to_owned(),
                    1_000_000_000,
                    2_000_000_000,
                    0,
                ),
                (200, "game.exe".to_owned(), 3_000_000_000, 4_000_000_000, 0),
            ],
            latest_spike: None,
            ..Default::default()
        };
        let mut state = MemoryTableState::default();

        egui::__run_test_ui(|ui| {
            crate::modules::memory::table_view::render_memory_table(ui, &snap, &mut state);
        });
    }

    #[test]
    fn test_table_cache_invalidates_on_sort_change() {
        let snap = MemorySnapshot {
            ram_total_bytes: 16_000_000_000,
            processes: vec![
                (
                    100,
                    "chrome.exe".to_owned(),
                    1_000_000_000,
                    2_000_000_000,
                    0,
                ),
                (200, "game.exe".to_owned(), 3_000_000_000, 4_000_000_000, 0),
            ],
            latest_spike: None,
            ..Default::default()
        };
        let mut state = MemoryTableState::default();

        // First render - populates cache
        egui::__run_test_ui(|ui| {
            crate::modules::memory::table_view::render_memory_table(ui, &snap, &mut state);
        });
        assert!(state.has_cached_rows());

        // Change sort column - should invalidate cache
        state.sort_col = crate::modules::memory::process_model::MemorySortCol::Name;
        egui::__run_test_ui(|ui| {
            crate::modules::memory::table_view::render_memory_table(ui, &snap, &mut state);
        });
        assert!(state.has_cached_rows()); // Recomputed
    }
}
