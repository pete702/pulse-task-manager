//! CPU chart axis math and window management.

use crate::modules::cpu::history::CpuChartWindow;
use eframe::egui;
use egui_plot::{GridInput, GridMark};

/// Custom X-axis grid spacer for consistent major/minor grid lines across all windows.
/// X coordinates are in minutes (negative = minutes ago, 0 = now).
pub fn x_grid_spacer(span_min: f64, _input: &GridInput) -> Vec<GridMark> {
    let (major_step, minor_step) = if span_min <= 1.0 {
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
    for i in 0..=(span_min / minor_step).ceil() as i32 {
        let val = -i as f64 * minor_step;
        marks.push(GridMark { value: val, step_size: minor_step });
    }
    marks
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

pub fn anchored(points: &[[f64; 2]], _span: f64) -> Vec<[f64; 2]> {
    if points.is_empty() {
        return vec![];
    }
    if points.len() == 1 {
        let y = points[0][1];
        return vec![[-ONE_SEC_MIN, y], [0.0, y]];
    }
    let mut out = points.to_vec();
    if let Some(last) = out.last().copied() {
        if last[0] < 0.0 {
            out.push([0.0, last[1]]);
        }
    }
    out
}

pub fn convert_to_minutes_ago(points: &[[f64; 2]], now: f64) -> Vec<[f64; 2]> {
    points.iter().map(|p| [(p[0] - now) / 60.0, p[1]]).collect()
}
