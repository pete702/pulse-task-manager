//! Memory chart window helpers.

use crate::modules::memory::model::ChartWindow;

#[cfg(test)]
pub use crate::ui::chart_scale::{axis_label, minutes_ago};

/// Chart window duration in seconds: `60`, `300`, or `900`.
pub fn window_seconds(window: ChartWindow) -> usize {
    window.seconds() as usize
}

/// Chart window duration in minutes: `1.0`, `5.0`, or `15.0`.
pub fn window_minutes(window: ChartWindow) -> f64 {
    window.seconds() as f64 / 60.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn window_seconds_values() {
        assert_eq!(window_seconds(ChartWindow::OneMin), 60);
        assert_eq!(window_seconds(ChartWindow::FiveMin), 300);
        assert_eq!(window_seconds(ChartWindow::FifteenMin), 900);
    }

    #[test]
    fn window_minutes_values() {
        let cases = [
            (ChartWindow::OneMin, 1.0_f64),
            (ChartWindow::FiveMin, 5.0_f64),
            (ChartWindow::FifteenMin, 15.0_f64),
        ];
        for (w, expected) in cases {
            let got = window_minutes(w);
            assert!((got - expected).abs() < 1e-9, "got {got} expected {expected}");
        }
    }
}
