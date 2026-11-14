//! Shared chart helpers for PULSE telemetry plots.

/// Finds the point nearest to `hover_x` using binary search (O(log N)).
pub fn find_nearest_point(points: &[[f64; 2]], hover_x: f64, max_dist: f64) -> Option<[f64; 2]> {
    if points.is_empty() {
        return None;
    }
    let idx = points.partition_point(|p| p[0] < hover_x);
    let mut best: Option<[f64; 2]> = None;
    if idx > 0 {
        best = Some(points[idx - 1]);
    }
    if idx < points.len() {
        let candidate = points[idx];
        if best.is_none_or(|p| (candidate[0] - hover_x).abs() < (p[0] - hover_x).abs()) {
            best = Some(candidate);
        }
    }
    best.filter(|p| (p[0] - hover_x).abs() <= max_dist)
}

/// Returns the index of the point nearest to `hover_x`.
pub fn find_nearest_index(points: &[[f64; 2]], hover_x: f64) -> Option<usize> {
    if points.is_empty() {
        return None;
    }
    let idx = points.partition_point(|p| p[0] < hover_x);
    let mut best = if idx > 0 { Some(idx - 1) } else { None };
    if idx < points.len()
        && best.is_none_or(|i| (points[idx][0] - hover_x).abs() < (points[i][0] - hover_x).abs())
    {
        best = Some(idx);
    }
    best
}

/// Unified plot baseline (y_min = 0.0).
pub fn baseline_y() -> f64 {
    0.0
}

/// Unified dark chart container styling.
pub fn chart_frame() -> eframe::egui::Frame {
    crate::theme::card_frame()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn nearest_point_basic() {
        let pts = [[0.0, 1.0], [1.0, 2.0], [2.0, 3.0]];
        assert_eq!(find_nearest_point(&pts, 1.2, 10.0), Some([1.0, 2.0]));
    }
    #[test]
    fn nearest_point_empty() {
        let pts: [[f64; 2]; 0] = [];
        assert_eq!(find_nearest_point(&pts, 0.0, 10.0), None);
    }
    #[test]
    fn nearest_point_max_dist() {
        let pts = [[0.0, 1.0], [10.0, 2.0]];
        assert_eq!(find_nearest_point(&pts, 5.0, 1.0), None);
    }
}
