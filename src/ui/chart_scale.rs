//! Shared scaling and bounds helpers for time-series charts.

use egui_plot::{GridInput, GridMark, PlotBounds, PlotUi};
use std::sync::OnceLock;

const STEPS: [f64; 15] =
    [1.0, 1.2, 1.6, 2.0, 2.4, 2.8, 3.2, 3.6, 4.0, 4.8, 5.6, 6.0, 7.2, 8.0, 10.0];

pub const ONE_SEC_MIN: f64 = 1.0 / 60.0;

pub const PERCENT_Y_GRID: [GridMark; 5] = [
    GridMark { value: 0.0, step_size: 25.0 },
    GridMark { value: 25.0, step_size: 25.0 },
    GridMark { value: 50.0, step_size: 25.0 },
    GridMark { value: 75.0, step_size: 25.0 },
    GridMark { value: 100.0, step_size: 25.0 },
];

pub(crate) fn clean_4_axis_max(raw_peak: f64, min_floor: f64) -> f64 {
    let peak = if raw_peak.is_finite() { raw_peak.max(0.0) } else { 0.0 };
    let floor = if min_floor.is_finite() { min_floor.max(0.0) } else { 0.0 };
    let target = (peak * 1.08).max(floor.max(1.0));
    let exp = target.log10().floor();
    let scale = 10_f64.powf(exp);
    let norm = target / scale;
    let step = STEPS.into_iter().find(|s| *s >= norm - 1e-9).unwrap_or(10.0);
    let out = step * scale;
    if out < floor {
        floor
    } else {
        out
    }
}

pub(crate) fn format_y_tick(val: f64, y_max: f64, unit: &str) -> String {
    let v = if val.is_finite() { val } else { 0.0 };
    let step = if y_max.is_finite() && y_max > 0.0 { y_max / 4.0 } else { 0.0 };
    let step_is_int = (step - step.round()).abs() < 1e-9;
    let mut s = if y_max < 4.0 {
        format!("{v:.1}")
    } else if step_is_int {
        format!("{v:.0}")
    } else {
        format!("{v:.1}")
    };
    if !unit.is_empty() {
        s.push(' ');
        s.push_str(unit);
    }
    s
}

pub fn window_secs_to_span_min(window_secs: usize) -> f64 {
    (window_secs.max(60) as f64) / 60.0
}

pub fn lock_time_chart_bounds(plot_ui: &mut PlotUi, window_secs: usize, y_max: f64) {
    let span = window_secs_to_span_min(window_secs);
    let top = if y_max.is_finite() && y_max > 0.0 { y_max } else { 1.0 };
    plot_ui.set_plot_bounds(PlotBounds::from_min_max([-span, 0.0], [0.0, top]));
}

pub fn minutes_ago(sample_ts: f64, now_ts: f64) -> f64 {
    (sample_ts - now_ts) / 60.0
}

pub fn convert_to_minutes_ago(points: &[[f64; 2]], now_ts: f64) -> Vec<[f64; 2]> {
    points.iter().map(|p| [minutes_ago(p[0], now_ts), p[1]]).collect()
}

pub fn anchored(mut points: Vec<[f64; 2]>) -> Vec<[f64; 2]> {
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

pub fn build_x_grid_marks(span_min: f64) -> Vec<GridMark> {
    let (major_step, minor_step): (f64, f64) = if span_min <= 1.0 {
        (0.25, 1.0 / 12.0)
    } else if span_min <= 5.0 {
        (1.0, 0.25)
    } else {
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

fn cached_x_grid_marks(span_min: f64) -> &'static Vec<GridMark> {
    static MARKS: OnceLock<[Vec<GridMark>; 3]> = OnceLock::new();
    let marks = MARKS.get_or_init(|| {
        [build_x_grid_marks(1.0), build_x_grid_marks(5.0), build_x_grid_marks(15.0)]
    });
    if span_min <= 1.0 {
        &marks[0]
    } else if span_min <= 5.0 {
        &marks[1]
    } else {
        &marks[2]
    }
}

pub fn x_grid_spacer(span_min: f64, _input: &GridInput) -> Vec<GridMark> {
    cached_x_grid_marks(span_min).clone()
}

/// Returns the X-axis tick marks (in minutes) for the given chart window.
/// Positive values mean minutes ago; the plot X coordinates use negative minutes.
/// window_secs: <= 60 for 1m, <= 300 for 5m, otherwise 15m.
pub fn x_axis_ticks(window_secs: usize) -> Vec<f64> {
    if window_secs <= 60 {
        vec![1.0, 0.75, 0.5, 0.25, 0.0]
    } else if window_secs <= 300 {
        vec![5.0, 4.0, 3.0, 2.0, 1.0, 0.0]
    } else {
        vec![15.0, 12.0, 9.0, 6.0, 3.0, 0.0]
    }
}

/// Formats a tick value (in minutes, positive = minutes ago) to a label string.
/// No minus signs -- all labels show "time ago" semantics.
/// window_secs: <= 60 for 1m, <= 300 for 5m, otherwise 15m.
pub fn format_tick_label(window_secs: usize, tick: f64) -> String {
    if tick.abs() < 0.01 {
        if window_secs <= 60 {
            "0s".to_owned()
        } else {
            "0m".to_owned()
        }
    } else if window_secs <= 60 {
        format!("{:.0}s", tick * 60.0)
    } else {
        format!("{:.0}m", tick)
    }
}

/// Formats an X-axis mark in minutes; both `0.0` and `-0.0` become `"0m"`.
#[cfg(test)]
pub fn format_minutes(mark: f64) -> String {
    let rounded = mark.round();
    if mark.abs() < 0.05 {
        "0m".to_owned()
    } else if (mark - rounded).abs() < 0.05 {
        format!("{rounded:.0}m")
    } else {
        format!("{mark:.1}m")
    }
}

/// X-axis label for a window. The 1m view only labels `-1m`, `-0.5m`, and `0m`.
#[cfg(test)]
pub fn axis_label(window_secs: u32, mark: f64) -> String {
    if window_secs == 60 {
        if (mark + 1.0).abs() < 0.01 {
            "-1m".to_owned()
        } else if (mark + 0.5).abs() < 0.01 {
            "-0.5m".to_owned()
        } else if mark.abs() < 0.01 {
            "0m".to_owned()
        } else {
            String::new()
        }
    } else {
        format_minutes(mark)
    }
}

#[cfg(test)]
#[path = "chart_scale_tests.rs"]
mod tests;
