//! Unit tests for memory model (MemorySnapshot, ChartWindow).

#[cfg(test)]
mod tests {
    use crate::modules::memory::model::{ChartWindow, MemorySnapshot};

    #[test]
    fn test_swap_division_by_zero_guard() {
        // Swap disabled (total == 0) should return 0.0, not panic
        let snap = MemorySnapshot {
            swap_total_bytes: 0,
            swap_used_bytes: 100,
            swap_free_bytes: 0,
            ..Default::default()
        };
        assert_eq!(snap.swap_usage_percent(), 0.0);

        // Normal case
        let snap2 = MemorySnapshot {
            swap_total_bytes: 1000,
            swap_used_bytes: 500,
            swap_free_bytes: 500,
            ..Default::default()
        };
        assert!((snap2.swap_usage_percent() - 50.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_commit_division_by_zero_guard() {
        let snap = MemorySnapshot {
            commit_limit_bytes: 0,
            commit_total_bytes: 100,
            ..Default::default()
        };
        assert_eq!(snap.commit_usage_percent(), 0.0);

        let snap2 = MemorySnapshot {
            commit_limit_bytes: 1000,
            commit_total_bytes: 500,
            ..Default::default()
        };
        assert!((snap2.commit_usage_percent() - 50.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_ram_cached_bytes_saturating_sub() {
        // Normal case: available > free
        let snap = MemorySnapshot {
            ram_available_bytes: 8_000_000_000,
            ram_free_bytes: 2_000_000_000,
            ..Default::default()
        };
        assert_eq!(snap.ram_cached_bytes(), 6_000_000_000);

        // Edge case: free > available (race condition in OS reporting)
        // saturating_sub prevents panic, returns 0
        let snap2 = MemorySnapshot {
            ram_available_bytes: 2_000_000_000,
            ram_free_bytes: 8_000_000_000,
            ..Default::default()
        };
        assert_eq!(snap2.ram_cached_bytes(), 0);
    }

    #[test]
    fn test_bytes_to_gib_mib_conversion() {
        const GIB: u64 = 1_073_741_824;
        const MIB: u64 = 1_048_576;

        assert!((MemorySnapshot::bytes_to_gib(GIB) - 1.0).abs() < f64::EPSILON);
        assert!((MemorySnapshot::bytes_to_gib(2 * GIB) - 2.0).abs() < f64::EPSILON);
        assert!((MemorySnapshot::bytes_to_gib(0) - 0.0).abs() < f64::EPSILON);

        assert!((MemorySnapshot::bytes_to_mib(MIB) - 1.0).abs() < f64::EPSILON);
        assert!((MemorySnapshot::bytes_to_mib(512 * MIB) - 512.0).abs() < f64::EPSILON);
        assert!((MemorySnapshot::bytes_to_mib(0) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_usage_percent_clamping() {
        // Over 100% (should not happen, but clamp protects)
        let snap = MemorySnapshot {
            ram_total_bytes: 1000,
            ram_used_bytes: 1500, // > total
            ..Default::default()
        };
        assert_eq!(snap.ram_usage_percent(), 100.0);

        // Under 0% (impossible, but clamp protects)
        let snap2 = MemorySnapshot {
            ram_total_bytes: 1000,
            ram_used_bytes: 0,
            ..Default::default()
        };
        assert_eq!(snap2.ram_usage_percent(), 0.0);

        // Normal case
        let snap3 = MemorySnapshot {
            ram_total_bytes: 1000,
            ram_used_bytes: 500,
            ..Default::default()
        };
        assert!((snap3.ram_usage_percent() - 50.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_chart_window_seconds() {
        assert_eq!(ChartWindow::OneMin.seconds(), 60);
        assert_eq!(ChartWindow::FiveMin.seconds(), 300);
        assert_eq!(ChartWindow::FifteenMin.seconds(), 900);
    }

    #[test]
    fn test_chart_window_labels_en() {
        assert_eq!(ChartWindow::OneMin.label_en(), "1m");
        assert_eq!(ChartWindow::FiveMin.label_en(), "5m");
        assert_eq!(ChartWindow::FifteenMin.label_en(), "15m");
    }

    #[test]
    fn test_commit_usage_percent() {
        let snap = MemorySnapshot {
            commit_total_bytes: 5_000_000_000,
            commit_limit_bytes: 10_000_000_000,
            ..Default::default()
        };
        assert!((snap.commit_usage_percent() - 50.0).abs() < f32::EPSILON);

        let snap2 = MemorySnapshot {
            commit_total_bytes: 12_000_000_000,
            commit_limit_bytes: 10_000_000_000,
            ..Default::default()
        };
        assert_eq!(snap2.commit_usage_percent(), 100.0);
    }
}
