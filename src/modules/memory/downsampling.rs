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

    // Preserve global peak so 15m view never shaves true spikes.
    let peak_idx = points
        .iter()
        .enumerate()
        .max_by(|a, b| a.1[1].total_cmp(&b.1[1]))
        .map(|(i, _)| i)
        .unwrap_or(0);

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
            if j == peak_idx {
                max_idx = j;
                break;
            }
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

/// 9-step decade ladder with 5% boundary tolerance, no upper cap.
pub fn nice_ceiling(peak: f64, min: f64) -> f64 {
    let need = peak.max(min);
    if !need.is_finite() || need <= 0.0 {
        return min;
    }
    let exp = need.log10().floor() as i32;
    let norm = need / 10_f64.powi(exp);
    const STEPS: [f64; 9] = [1.0, 1.5, 2.0, 2.5, 3.0, 4.0, 5.0, 7.5, 10.0];
    let mult = STEPS.into_iter().find(|s| norm <= s * 1.05).unwrap_or(10.0);
    let v = 10_f64.powi(exp) * mult;
    v.max(min)
}

/// LTTB that keeps NAN gaps broken: each finite run is sampled alone.
pub fn lttb_segments(points: &[[f64; 2]], threshold: usize) -> Vec<[f64; 2]> {
    if points.iter().all(|p| p[1].is_finite()) {
        return lttb_downsample(points, threshold);
    }
    let total_finite = points.iter().filter(|p| p[1].is_finite()).count();
    if total_finite <= threshold {
        return points.to_vec();
    }
    let mut out = Vec::with_capacity(points.len().min(threshold + 32));
    let mut start = 0usize;
    while start < points.len() {
        if !points[start][1].is_finite() {
            out.push(points[start]);
            start += 1;
            continue;
        }
        let mut end = start;
        while end < points.len() && points[end][1].is_finite() {
            end += 1;
        }
        let seg = &points[start..end];
        let budget = (seg.len() * threshold / total_finite.max(1)).clamp(2, threshold);
        out.extend(lttb_downsample(seg, budget.max(2)));
        start = end;
    }
    out
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

    #[test]
    fn test_nice_ceiling_with_tolerance() {
        assert_eq!(nice_ceiling(90.0, 1.0), 100.0);
        assert_eq!(nice_ceiling(105.0, 1.0), 100.0);
        assert_eq!(nice_ceiling(900.0, 1.0), 1000.0);
        assert_eq!(nice_ceiling(1050.0, 1.0), 1000.0);
    }

    #[test]
    fn test_nice_ceiling_ping_ladder() {
        assert_eq!(nice_ceiling(4.0, 5.0), 5.0);
        assert_eq!(nice_ceiling(12.0, 5.0), 15.0);
        assert_eq!(nice_ceiling(19.0, 5.0), 20.0);
        assert_eq!(nice_ceiling(23.0, 5.0), 25.0);
    }
}
