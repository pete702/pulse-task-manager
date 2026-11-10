//! CPU chart axis math and window management.

use crate::modules::cpu::history::CpuChartWindow;
use eframe::egui;
use egui_plot::{GridInput, GridMark};
use std::sync::OnceLock;

/// Returns cached Y-axis grid marks for CPU charts.
pub fn cached_y_grid_marks() -> Vec<GridMark> {
    static MARKS: OnceLock<Vec<GridMark>> = OnceLock::new();
    MARKS
        .get_or_init(|| {
            vec![
                GridMark { value: 0.0, step_size: 25.0 },
                GridMark { value: 25.0, step_size: 25.0 },
                GridMark { value: 50.0, step_size: 25.0 },
                GridMark { value: 75.0, step_size: 25.0 },
                GridMark { value: 100.0, step_size: 25.0 },
            ]
        })
        .clone()
}

/// Builds X-axis grid marks for the given CPU chart window.
pub fn build_x_grid_marks(window: CpuChartWindow) -> Vec<GridMark> {
    let span_min = window.seconds() as f64 / 60.0;
    let (major_step, minor_step): (f64, f64) = if span_min <= 1.0 {
        // 1m window: major every 15s (0.25 min), minor every 5s (1/12 min)
        (0.25, 1.0 / 12.0)
    } else if span_min <= 5.0 {
        // 5m window: major every 1m, minor every 15s (0.25 min)
        (1.0, 0.25)
    } else {
        // 15m window: major every 3m, minor every 1m
        (3.0, 1.0)
    };

    let mut marks = Vec::new();
    for i in 0..=(span_min / major_step).ceil() as i32 {
        let val = -i as f64 * major_step;
        marks.push(GridMark { value: val, step_size: major_step });
    }
    let ratio = if minor_step.abs() < 1e-12_f64 {
        1
    } else {
        ((major_step / minor_step).round() as usize).max(1)
    };
    if (major_step - minor_step).abs() >= 1e-6_f64 {
        for i in 0..=(span_min / minor_step).ceil() as i32 {
            if (i as usize).is_multiple_of(ratio) {
                continue;
            }
            let val = -i as f64 * minor_step;
            marks.push(GridMark { value: val, step_size: minor_step });
        }
    }
    marks
}

fn cached_x_grid_marks(window: CpuChartWindow) -> &'static Vec<GridMark> {
    static MARKS: OnceLock<[Vec<GridMark>; 3]> = OnceLock::new();
    let marks = MARKS.get_or_init(|| {
        [
            build_x_grid_marks(CpuChartWindow::OneMin),
            build_x_grid_marks(CpuChartWindow::FiveMin),
            build_x_grid_marks(CpuChartWindow::FifteenMin),
        ]
    });
    match window {
        CpuChartWindow::OneMin => &marks[0],
        CpuChartWindow::FiveMin => &marks[1],
        CpuChartWindow::FifteenMin => &marks[2],
    }
}

/// Custom X-axis grid spacer for consistent major/minor grid lines across all windows.
/// X coordinates are in minutes (negative = minutes ago, 0 = now).
pub fn x_grid_spacer(span_min: f64, _input: &GridInput) -> Vec<GridMark> {
    let window = if span_min <= 1.0 {
        CpuChartWindow::OneMin
    } else if span_min <= 5.0 {
        CpuChartWindow::FiveMin
    } else {
        CpuChartWindow::FifteenMin
    };
    cached_x_grid_marks(window).clone()
}

fn window_id() -> egui::Id {
    egui::Id::new("pulse_cpu_chart_window")
}

pub fn selected_window(ctx: &egui::Context) -> CpuChartWindow {
    let mut window = CpuChartWindow::FiveMin;
    ctx.data_mut(|d| {
        window = *d.get_temp_mut_or_insert_with(window_id(), || CpuChartWindow::FiveMin)
    });
    window
}

pub fn set_window(ctx: &egui::Context, window: CpuChartWindow) {
    ctx.data_mut(|d| d.insert_temp(window_id(), window));
}

/// Anchors series to right edge (now = 0.0).
const ONE_SEC_MIN: f64 = 1.0 / 60.0;

pub fn anchored(mut points: Vec<[f64; 2]>, _span: f64) -> Vec<[f64; 2]> {
    if points.is_empty() {
        return vec![];
    }
    if points.len() == 1 {
        let y = points[0][1];
        return vec![[-ONE_SEC_MIN, y], [0.0, y]];
    }
    if let Some(last) = points.last().copied() {
        if last[0] < 0.0 {
            points.push([0.0, last[1]]);
        }
    }
    points
}

pub fn convert_to_minutes_ago(points: &[[f64; 2]], now: f64) -> Vec<[f64; 2]> {
    let mut out = Vec::with_capacity(points.len() + 1);
    out.extend(points.iter().map(|p| [(p[0] - now) / 60.0, p[1]]));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convert_to_minutes_ago_works() {
        assert!(convert_to_minutes_ago(&[], 10.0).is_empty());
        let pts = [[60.0, 1.0], [120.0, 2.0]];
        let out = convert_to_minutes_ago(&pts, 120.0);
        assert_eq!(out, vec![[-1.0, 1.0], [0.0, 2.0]]);
    }

    #[test]
    fn anchored_works() {
        assert!(anchored(Vec::new(), 1.0).is_empty());
        let one = anchored(vec![[-0.5, 7.0]], 1.0);
        assert_eq!(one.len(), 2);
        assert!((one[0][0] - (-1.0 / 60.0)).abs() < 1e-12);
        assert_eq!(one[1], [0.0, 7.0]);
        let multi = anchored(vec![[-1.0, 1.0], [-0.2, 2.0]], 1.0);
        assert_eq!(multi.first().copied(), Some([-1.0, 1.0]));
        assert_eq!(multi.last().copied(), Some([0.0, 2.0]));
        assert_eq!(multi.iter().filter(|p| (p[0] - 0.0).abs() < 1e-12).count(), 1);
    }

    #[test]
    fn cached_y_grid_marks_has_expected_values() {
        let marks = cached_y_grid_marks();
        let vals: Vec<f64> = marks.iter().map(|m| m.value).collect();
        assert_eq!(vals, vec![0.0, 25.0, 50.0, 75.0, 100.0]);
        assert_eq!(marks.len(), 5);
    }

    #[test]
    fn build_x_grid_marks_has_no_duplicate_coordinates() {
        for w in [CpuChartWindow::OneMin, CpuChartWindow::FiveMin, CpuChartWindow::FifteenMin] {
            let marks = build_x_grid_marks(w);
            for i in 0..marks.len() {
                for j in (i + 1)..marks.len() {
                    assert!(
                        (marks[i].value - marks[j].value).abs() > 1e-5,
                        "duplicate x value {}",
                        marks[i].value
                    );
                }
            }
        }
    }
}
