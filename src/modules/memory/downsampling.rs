//! Pure-Rust LTTB (Largest-Triangle-Three-Buckets) downsampling for time series.

/// Pure-Rust LTTB (Largest-Triangle-Three-Buckets) downsampling for time series.
/// Reduces a series of [timestamp, value] points to at most `threshold` points
/// while preserving visual shape. Endpoints are always preserved.
/// Returns a new vector with downsampled points (≤ threshold).
#[allow(clippy::needless_range_loop)]
pub fn lttb_downsample(points: &[[f64; 2]], threshold: usize) -> Vec<[f64; 2]> {
    let n = points.len();
    if n <= threshold || threshold <= 2 {
        return points.to_vec();
    }

    // Number of buckets (excluding first and last point which are always kept)
    let bucket_count = threshold - 2;
    let bucket_size = (n - 2) as f64 / bucket_count as f64;

    let mut sampled = Vec::with_capacity(threshold);
    // Always keep first point
    sampled.push(points[0]);

    for i in 0..bucket_count {
        // Bucket range (excluding first point)
        let start = 1 + (i as f64 * bucket_size).floor() as usize;
        let end = 1 + ((i + 1) as f64 * bucket_size).ceil() as usize;
        let end = end.min(n - 1); // Don't include last point

        if start >= end {
            continue;
        }

        // First point of this bucket
        let first = points[start];
        // Last point of next bucket (or last point overall)
        let last_idx = if i == bucket_count - 1 {
            n - 1
        } else {
            1 + ((i + 1) as f64 * bucket_size).floor() as usize
        };
        let last = points[last_idx.min(n - 1)];

        // Find point in current bucket that forms largest triangle with first and last
        let mut max_area = -1.0;
        let mut max_idx = start;

        for j in start..end {
            let p = points[j];
            // Triangle area = 0.5 * |(x2-x1)(y3-y1) - (x3-x1)(y2-y1)|
            // We can skip the 0.5 factor since we only compare
            let area = ((first[0] - last[0]) * (p[1] - first[1])
                - (p[0] - first[0]) * (first[1] - last[1]))
                .abs();
            if area > max_area {
                max_area = area;
                max_idx = j;
            }
        }

        sampled.push(points[max_idx]);
    }

    // Always keep last point
    sampled.push(points[n - 1]);
    sampled
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lttb_preserves_endpoints() {
        let points: Vec<[f64; 2]> = (0..100).map(|i| [i as f64, (i as f64).sin()]).collect();
        let downsampled = lttb_downsample(&points, 10);
        assert_eq!(downsampled.len(), 10);
        assert_eq!(downsampled[0], points[0]);
        assert_eq!(downsampled[9], points[99]);
    }

    #[test]
    fn test_lttb_reduces_900_to_300() {
        let points: Vec<[f64; 2]> = (0..900).map(|i| [i as f64, (i as f64).sin()]).collect();
        let downsampled = lttb_downsample(&points, 300);
        assert_eq!(downsampled.len(), 300);
        assert_eq!(downsampled[0], points[0]);
        assert_eq!(downsampled[299], points[899]);
    }

    #[test]
    fn test_lttb_small_input() {
        let points: Vec<[f64; 2]> = vec![[0.0, 1.0], [1.0, 2.0], [2.0, 3.0]];
        let downsampled = lttb_downsample(&points, 10);
        assert_eq!(downsampled.len(), 3);
    }
}
