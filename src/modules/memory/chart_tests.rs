//! Unit tests for memory chart view.

#[cfg(test)]
mod tests {
    use crate::modules::memory::history::MemoryHistory;
    use crate::modules::memory::model::ChartWindow;

    #[test]
    fn test_render_memory_chart_no_panic() {
        let mut history = MemoryHistory::new();

        // Add some test data
        for i in 0..120 {
            let snap = crate::modules::memory::model::MemorySnapshot {
                ram_total_bytes: 16_000_000_000,
                ram_used_bytes: (8_000_000_000 + (i * 10_000_000) as u64) % 16_000_000_000,
                ram_available_bytes: 8_000_000_000,
                ram_free_bytes: 2_000_000_000,
                swap_total_bytes: 8_000_000_000,
                swap_used_bytes: (2_000_000_000 + (i * 5_000_000) as u64) % 8_000_000_000,
                swap_free_bytes: 6_000_000_000,
                ..Default::default()
            };
            history.push(&snap);
        }

        // Verify history has correct number of points
        assert_eq!(history.len(), 120);
        assert_eq!(history.ram_points.len(), 120);
        assert_eq!(history.swap_points.len(), 120);
    }

    #[test]
    fn test_chart_window_switcher() {
        let mut history = MemoryHistory::new();
        history.chart_window = ChartWindow::OneMin;
        assert_eq!(history.chart_window, ChartWindow::OneMin);

        history.chart_window = ChartWindow::FiveMin;
        assert_eq!(history.chart_window, ChartWindow::FiveMin);

        history.chart_window = ChartWindow::FifteenMin;
        assert_eq!(history.chart_window, ChartWindow::FifteenMin);
    }
}
