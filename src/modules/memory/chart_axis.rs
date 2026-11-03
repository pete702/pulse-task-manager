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
