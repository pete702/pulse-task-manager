//! Unit tests for MemoryHistory ring buffer and accessors.

#[cfg(test)]
mod tests {
    use crate::modules::memory::history::MemoryHistory;
    use crate::modules::memory::model::{ChartWindow, MemorySnapshot};

    fn snap(rt: u64, ru: u64) -> MemorySnapshot {
        MemorySnapshot { ram_total_bytes: rt, ram_used_bytes: ru, ..Default::default() }
    }

    #[test]
    fn test_history_push_and_len() {
        let mut h = MemoryHistory::new();
        assert_eq!(h.len(), 0);
        assert!(h.is_empty());

        let snap =
            MemorySnapshot { ram_total_bytes: 1000, ram_used_bytes: 500, ..Default::default() };
        h.push(&snap);
        assert_eq!(h.len(), 1);
        assert!(!h.is_empty());
    }

    #[test]
    fn test_history_ring_buffer_eviction() {
        let mut h = MemoryHistory::new();
        for i in 0..965 {
            let snap = MemorySnapshot {
                ram_total_bytes: 1000,
                ram_used_bytes: 500 + (i % 10) as u64,
                sample_elapsed_secs: (i + 1) as f64,
                ..Default::default()
            };
            h.push(&snap);
        }
        assert_eq!(h.len(), 960, "ring buffer evicts the oldest samples");
    }

    #[test]
    fn test_history_windowed_points() {
        let mut h = MemoryHistory::new();
        for i in 0..400 {
            h.push(&MemorySnapshot { sample_elapsed_secs: i as f64 + 1.0, ..snap(100, i % 100) });
        }
        h.chart_window = ChartWindow::OneMin;
        assert_eq!(h.windowed_ram_points().count(), 60);
        h.chart_window = ChartWindow::FifteenMin;
        assert_eq!(h.windowed_ram_points().count(), 400);
    }

    #[test]
    fn test_short_history_starts_at_first_sample() {
        let mut h = MemoryHistory::new();
        for i in 1..=10 {
            h.push(&MemorySnapshot {
                ram_total_bytes: 1000,
                ram_used_bytes: 500,
                sample_elapsed_secs: i as f64,
                ..Default::default()
            });
        }
        let series = h.ram_series(ChartWindow::OneMin);
        assert_eq!(series.first().map(|p| p[0]), Some(1.0));
        assert_eq!(series.last().map(|p| p[0]), Some(10.0));
    }

    #[test]
    fn test_long_history_anchored_at_60s_window() {
        let mut h = MemoryHistory::new();
        for i in 1..=70 {
            h.push(&MemorySnapshot {
                ram_total_bytes: 1000,
                ram_used_bytes: 500,
                sample_elapsed_secs: i as f64,
                ..Default::default()
            });
        }
        let series = h.ram_series(ChartWindow::OneMin);
        assert_eq!(series.first().map(|p| p[0]), Some(10.0));
        assert_eq!(series.last().map(|p| p[0]), Some(70.0));
    }
}
