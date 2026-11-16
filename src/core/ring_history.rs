//! Shared ring-buffer history helpers for PULSE telemetry series.
use std::collections::VecDeque;

pub const HISTORY_CAPACITY: usize = 1920;

/// Monotonic timestamp guard: returns true if `ts` is strictly greater than `last`.
pub fn is_newer(last: Option<f64>, ts: f64) -> bool {
    last.is_none_or(|l| ts > l)
}

/// Window slice indices (start, end) for a given window in seconds.
#[allow(clippy::needless_range_loop)]
pub fn window_bounds(
    len: usize,
    now: f64,
    timestamps: &VecDeque<f64>,
    window_secs: f64,
) -> (usize, usize) {
    if len == 0 {
        return (0, 0);
    }
    let _ = now - window_secs;
    let mut start = 0usize;
    for i in 0..len {
        if now - timestamps[i] <= window_secs {
            start = i;
            break;
        }
        start = i + 1;
    }
    if start >= len {
        start = len.saturating_sub(1);
    }
    (start, len)
}

/// Revision-based series cache key.
#[cfg(test)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct SeriesCacheKey {
    pub scope_hash: u64,
    pub window_secs: u64,
    pub first_ts_bits: u64,
    pub last_ts_bits: u64,
    pub revision: u64,
}

#[cfg(test)]
impl SeriesCacheKey {
    pub fn new(
        scope_hash: u64,
        window_secs: u64,
        first_ts: f64,
        last_ts: f64,
        revision: u64,
    ) -> Self {
        Self {
            scope_hash,
            window_secs,
            first_ts_bits: first_ts.to_bits(),
            last_ts_bits: last_ts.to_bits(),
            revision,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_is_newer() {
        assert!(is_newer(None, 1.0));
        assert!(is_newer(Some(1.0), 2.0));
        assert!(!is_newer(Some(2.0), 2.0));
        assert!(!is_newer(Some(2.0), 1.0));
    }
    #[test]
    fn test_window_bounds_empty() {
        let ts: VecDeque<f64> = VecDeque::new();
        assert_eq!(window_bounds(0, 10.0, &ts, 60.0), (0, 0));
    }
    #[test]
    fn test_series_cache_key_equality() {
        let k1 = SeriesCacheKey::new(1, 60, 0.0, 10.0, 5);
        let k2 = SeriesCacheKey::new(1, 60, 0.0, 10.0, 5);
        assert_eq!(k1, k2);
        let k3 = SeriesCacheKey::new(1, 60, 0.0, 10.0, 6);
        assert_ne!(k1, k3);
    }
}
