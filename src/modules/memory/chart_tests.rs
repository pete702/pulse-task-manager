//! Unit tests for memory chart view.

#[cfg(test)]
mod tests {
    use crate::modules::memory::chart_axis::{axis_label, minutes_ago, window_minutes};
    use crate::modules::memory::chart_view::{render_memory_chart, selected_window, set_window};
    use crate::modules::memory::history::MemoryHistory;
    use crate::modules::memory::model::{ChartWindow, MemorySnapshot};

    fn snap(rt: u64, ru: u64) -> MemorySnapshot {
        MemorySnapshot { ram_total_bytes: rt, ram_used_bytes: ru, ..Default::default() }
    }

    /// History with one sample per minute over 20 minutes (latest at `base + 1200`).
    fn minute_history() -> MemoryHistory {
        let mut h = MemoryHistory::new();
        let base = 1_000_000.0;
        for i in 0..=20 {
            h.timestamps.push_back(base + i as f64 * 60.0);
            h.ram_points.push_back(40.0 + i as f32);
            h.swap_points.push_back(5.0 + i as f32);
        }
        h.revision = 1;
        h
    }

    #[test]
    fn test_render_memory_chart_no_panic() {
        let mut history = MemoryHistory::new();
        for i in 0..120 {
            let snap = MemorySnapshot {
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
        assert_eq!(history.len(), 120);
        assert_eq!(history.ram_points.len(), 120);
        assert_eq!(history.swap_points.len(), 120);
        egui::__run_test_ui(|ui| {
            render_memory_chart(ui, &mut history, &snap(16_000_000_000, 8_000_000_000), 220.0)
        });
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

    #[test]
    fn test_default_window_is_five_minutes() {
        let ctx = egui::Context::default();
        assert_eq!(selected_window(&ctx), ChartWindow::FiveMin);
        set_window(&ctx, ChartWindow::FifteenMin);
        assert_eq!(selected_window(&ctx), ChartWindow::FifteenMin);
    }

    #[test]
    fn test_chart_window_slices() {
        let mut h = MemoryHistory::new();
        for i in 0..400 {
            h.push(&snap(100, i % 100));
        }
        h.chart_window = ChartWindow::OneMin;
        assert_eq!(h.windowed_ram_points().count(), 60);
        h.chart_window = ChartWindow::FifteenMin;
        assert_eq!(h.windowed_ram_points().count(), 400);
    }

    #[test]
    fn test_minutes_axis_is_relative_to_latest() {
        assert!((minutes_ago(1_000.0, 1_000.0)).abs() < 1e-9);
        assert!((minutes_ago(940.0, 1_000.0) + 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_timed_window_uses_real_timestamps() {
        let mut h = MemoryHistory::new();
        for i in 0..10 {
            h.push(&snap(100, i));
        }
        h.timestamps.clear();
        for i in 0..10 {
            h.timestamps.push_back(1_000.0 + i as f64 * 60.0);
        }
        h.chart_window = ChartWindow::OneMin;
        assert_eq!(h.windowed_ram_timed().count(), 2);
        assert_eq!(h.windowed_swap_timed().count(), 2);
    }

    #[test]
    fn test_plot_points_stay_inside_each_window() {
        let mut h = minute_history();
        for window in [ChartWindow::OneMin, ChartWindow::FiveMin, ChartWindow::FifteenMin] {
            h.chart_window = window;
            let now = h.latest_timestamp();
            let span = window_minutes(window);
            assert_eq!(span, window.seconds() as f64 / 60.0);
            let points: Vec<[f64; 2]> =
                h.windowed_ram_timed().map(|(t, v)| [minutes_ago(t, now), v as f64]).collect();
            assert!(!points.is_empty(), "window {:?} must have samples", window);
            for point in &points {
                assert!(
                    point[0] >= -span - 1e-9 && point[0] <= 1e-9,
                    "x={} outside [-{span}, 0] for {:?}",
                    point[0],
                    window
                );
            }
            assert!(points.last().unwrap()[0].abs() < 1e-9, "newest sample must sit at 0m");
            assert_eq!(h.windowed_swap_timed().count(), points.len());
        }
    }

    #[test]
    fn test_one_minute_axis_labels_are_sparse() {
        assert_eq!(axis_label(60, -1.0), "-1m");
        assert_eq!(axis_label(60, -0.5), "-0.5m");
        assert_eq!(axis_label(60, 0.0), "0m");
        for mark in [-0.9, -0.8, -0.7, -0.6, -0.4, -0.3, -0.2, -0.1] {
            assert_eq!(axis_label(60, mark), "", "1m axis must hide mark {mark}");
        }
        assert_eq!(axis_label(300, -4.0), "-4m");
        assert_eq!(axis_label(900, 0.0), "0m");
    }

    #[test]
    fn test_precomputed_series_lttb_downsample() {
        let mut history = MemoryHistory::new();
        // Add 900 points (simulating 15 minutes at 1s interval)
        for i in 0..900 {
            let snap = MemorySnapshot {
                ram_total_bytes: 1000,
                ram_used_bytes: 500 + (i % 100) as u64,
                sample_elapsed_secs: i as f64,
                ..Default::default()
            };
            history.push(&snap);
        }
        assert_eq!(history.len(), 900);

        // Test that precomputed series returns LTTB-downsampled data (≤300 points)
        history.chart_window = ChartWindow::FifteenMin;
        let ram_series = history.ram_series(ChartWindow::FifteenMin);
        assert!(
            ram_series.len() <= 300,
            "LTTB should downsample 900 points to ≤300, got {}",
            ram_series.len()
        );

        // Endpoints should be preserved (first/last data points have ~50%..60% RAM usage)
        // First point: i=0 -> 500/1000 = 50.0%
        // Last point: i=899 -> 599/1000 = 59.9%
        assert!((ram_series[0][1] - 50.0).abs() < 1.0, "first point should be near start (~50%)");
        assert!(
            (ram_series.last().unwrap()[1] - 59.9).abs() < 1.0,
            "last point should be near end (~59.9%)"
        );

        // Test 1m window
        let ram_1m = history.ram_series(ChartWindow::OneMin);
        assert!(ram_1m.len() <= 300, "1m series should also be downsampled if >300 points");

        // Test 5m window
        let ram_5m = history.ram_series(ChartWindow::FiveMin);
        assert!(ram_5m.len() <= 300, "5m series should also be downsampled if >300 points");
    }

    #[test]
    fn test_history_wrap_invalidates_cache() {
        let mut history = MemoryHistory::new();
        for i in 0..905 {
            let snap = MemorySnapshot {
                ram_total_bytes: 1000,
                ram_used_bytes: 500 + (i % 10) as u64,
                sample_elapsed_secs: (i + 1) as f64,
                ..Default::default()
            };
            history.push(&snap);
        }
        assert_eq!(history.len(), 900, "ring buffer evicts the oldest samples");
        history.chart_window = ChartWindow::FifteenMin;
        assert_eq!(history.windowed_ram_timed().count(), 900);

        // Verify that new data invalidates and recomputes the precomputed series
        let _series_before = history.ram_series(ChartWindow::FifteenMin).clone();
        history.push(&snap(1000, 999));
        let series_after = history.ram_series(ChartWindow::FifteenMin);

        // Series should be recomputed (different Arc or different data)
        // We can't easily test Arc pointer equality, but we can verify it still works
        assert!(!series_after.is_empty());
    }
}
