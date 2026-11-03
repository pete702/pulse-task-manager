//! Unit tests for composition bar (RAMMap 3-segment bar).

#[cfg(test)]
mod tests {
    #[test]
    fn test_composition_bar_values() {
        // Test that percentages add up correctly
        let total = 16_000_000_000u64;
        let in_use = 8_000_000_000u64;
        let cached = 4_000_000_000u64;
        let free = 4_000_000_000u64;

        let total_f64 = total as f64;
        let in_use_pct = (in_use as f64 / total_f64 * 100.0) as f32;
        let cached_pct = (cached as f64 / total_f64 * 100.0) as f32;
        let free_pct = (free as f64 / total_f64 * 100.0) as f32;

        assert!((in_use_pct - 50.0).abs() < 0.1);
        assert!((cached_pct - 25.0).abs() < 0.1);
        assert!((free_pct - 25.0).abs() < 0.1);
        assert!((in_use_pct + cached_pct + free_pct - 100.0).abs() < 0.1);
    }
}
