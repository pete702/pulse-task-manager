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
        egui::__run_test_ui(|ui| render_memory_chart(ui, &mut history));
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
}
