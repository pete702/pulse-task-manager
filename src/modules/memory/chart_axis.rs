//! Pure axis math and X-axis labels for the memory chart.

use crate::modules::memory::model::ChartWindow;

/// Converts a unix timestamp into minutes relative to the newest sample.
pub fn minutes_ago(ts: f64, now: f64) -> f64 {
    (ts - now) / 60.0
}

/// Chart window duration in minutes: `1.0`, `5.0`, or `15.0`.
pub fn window_minutes(window: ChartWindow) -> f64 {
    window.seconds() as f64 / 60.0
}

/// Formats an X-axis mark in minutes; both `0.0` and `-0.0` become `"0m"`.
#[allow(dead_code)]
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

/// X-axis label for a window. The 1m view only labels `-1m`, `-0.5m`, and `0m`;
/// all other decimarks return an empty string to avoid `-0.9m ... -0.1m` clutter.
#[allow(dead_code)]
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

/// Returns the X-axis tick marks (in minutes) for the given chart window.
/// These are used for in-canvas HUD labels. All values are positive (minutes ago),
/// matching the "time ago" semantic. The plot X coordinates are still negative
/// (minutes ago from now=0), but we store positive values here for cleaner labels.
pub fn x_axis_ticks(window: ChartWindow) -> Vec<f64> {
    match window {
        ChartWindow::OneMin => vec![1.0, 0.75, 0.5, 0.25, 0.0], // 60s, 45s, 30s, 15s, 0s
        ChartWindow::FiveMin => vec![5.0, 4.0, 3.0, 2.0, 1.0, 0.0],
        ChartWindow::FifteenMin => vec![15.0, 12.0, 9.0, 6.0, 3.0, 0.0],
    }
}

/// Formats a tick value (in minutes, positive = minutes ago) to a label string.
/// No minus signs — all labels show "time ago" semantics.
pub fn format_tick_label(window: ChartWindow, tick: f64) -> String {
    if tick.abs() < 0.01 {
        if window.seconds() == 60 {
            "0s".to_owned()
        } else {
            "0m".to_owned()
        }
    } else if window.seconds() == 60 {
        // tick is positive minutes ago (e.g., 1.0 = 60s ago, 0.25 = 15s ago)
        format!("{:.0}s", tick * 60.0)
    } else {
        format!("{:.0}m", tick)
    }
}
