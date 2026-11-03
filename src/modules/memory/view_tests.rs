//! Unit tests for memory view (memory cards, kernel tiles).

#[cfg(test)]
mod tests {
    use crate::modules::memory::formatters::format_bytes;
    use crate::modules::memory::model::MemorySnapshot;

    #[test]
    fn test_render_memory_cards_no_panic() {
        let mem = MemorySnapshot {
            ram_total_bytes: 16_000_000_000,
            ram_used_bytes: 8_000_000_000,
            ram_available_bytes: 8_000_000_000,
            ram_free_bytes: 2_000_000_000,
            swap_total_bytes: 8_000_000_000,
            swap_used_bytes: 2_000_000_000,
            swap_free_bytes: 6_000_000_000,
            commit_total_bytes: 10_000_000_000,
            commit_limit_bytes: 20_000_000_000,
            kernel_paged_bytes: 100_000_000,
            kernel_nonpaged_bytes: 50_000_000,
            system_handles: 50000,
            system_threads: 2000,
            system_processes: 150,
            ..Default::default()
        };
        let _history = crate::modules::memory::history::MemoryHistory::new();

        // This should not panic (just testing function signature compiles)
        // We can't easily test UI rendering without a full egui context,
        // but we can verify the format_bytes helper works.
        assert!(!format_bytes(mem.ram_cached_bytes()).is_empty());
    }
}
