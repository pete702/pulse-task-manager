//! Unit tests for MemoryHistory ring buffer and windowing.

#[cfg(test)]
mod tests {
    use crate::modules::memory::history::MemoryHistory;
    use crate::modules::memory::model::{ChartWindow, MemorySnapshot};

    #[test]
    fn test_memory_history_ring_buffer_900() {
        let mut history = MemoryHistory::new();
        let base_snap = MemorySnapshot {
            ram_total_bytes: 1000,
            ram_used_bytes: 500,
            swap_total_bytes: 1000,
            swap_used_bytes: 250,
            ..Default::default()
        };

        // Push 1000 points (capacity is 900)
        for i in 0..1000 {
            let mut snap = base_snap.clone();
            snap.ram_used_bytes = 500 + (i % 10) as u64 * 10; // Vary slightly
            snap.swap_used_bytes = 250 + (i % 5) as u64 * 10;
            history.push(&snap);
        }

        // Should only keep last 900 points
        assert_eq!(history.len(), super::super::model::HISTORY_CAPACITY);
        assert_eq!(
            history.ram_points.len(),
            super::super::model::HISTORY_CAPACITY
        );
        assert_eq!(
            history.swap_points.len(),
            super::super::model::HISTORY_CAPACITY
        );

        // Peaks should be preserved from the 1000 points
        assert!(history.peak_ram_percent >= 50.0);
        assert!(history.peak_swap_percent >= 25.0);
    }

    #[test]
    fn test_windowed_points() {
        let mut history = MemoryHistory::new();
        let base_snap = MemorySnapshot {
            ram_total_bytes: 1000,
            ram_used_bytes: 500,
            swap_total_bytes: 1000,
            swap_used_bytes: 250,
            ..Default::default()
        };

        // Push 100 points
        for i in 0..100 {
            let mut snap = base_snap.clone();
            snap.ram_used_bytes = 500 + i as u64;
            history.push(&snap);
        }

        history.chart_window = ChartWindow::OneMin; // 60 points
        let ram_windowed: Vec<f32> = history.windowed_ram_points().collect();
        assert_eq!(ram_windowed.len(), 60);

        history.chart_window = ChartWindow::FiveMin; // 300 points, but only 100 available
        let ram_windowed: Vec<f32> = history.windowed_ram_points().collect();
        assert_eq!(ram_windowed.len(), 100);
    }
}
