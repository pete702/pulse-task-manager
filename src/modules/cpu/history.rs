//! CPU history ring buffer and chart window types.
//! Stores 960 samples (15 minutes at 1s interval) of dual series: total_pct + hottest_core_pct.
use crate::modules::memory::downsampling::lttb_downsample;
use crate::modules::memory::model::ChartWindow;
use std::collections::VecDeque;
use std::sync::Arc;
pub const CPU_HISTORY_CAPACITY: usize = 960;
/// Re-export memory's ChartWindow for CPU charts (identical values: 60, 300, 900).
pub type CpuChartWindow = ChartWindow;
#[derive(Clone, Debug, Default)]
pub struct CpuHistory {
    pub total_points: VecDeque<f32>,
    pub hottest_points: VecDeque<f32>,
    pub peak_total_percent: f32,
    pub peak_hottest_percent: f32,
    pub chart_window: CpuChartWindow,
    pub revision: u64,
    pub timestamps: VecDeque<f64>,
    pub show_core_grid: bool,
    cached_total_1m: Option<Arc<Vec<[f64; 2]>>>,
    cached_total_5m: Option<Arc<Vec<[f64; 2]>>>,
    cached_total_15m: Option<Arc<Vec<[f64; 2]>>>,
    cached_hottest_1m: Option<Arc<Vec<[f64; 2]>>>,
    cached_hottest_5m: Option<Arc<Vec<[f64; 2]>>>,
    cached_hottest_15m: Option<Arc<Vec<[f64; 2]>>>,
    cached_revision: u64,
}
impl CpuHistory {
    #[inline]
    pub fn new() -> Self {
        Self {
            total_points: VecDeque::with_capacity(CPU_HISTORY_CAPACITY),
            hottest_points: VecDeque::with_capacity(CPU_HISTORY_CAPACITY),
            peak_total_percent: 0.0,
            peak_hottest_percent: 0.0,
            chart_window: CpuChartWindow::default(),
            revision: 0,
            timestamps: VecDeque::with_capacity(CPU_HISTORY_CAPACITY),
            show_core_grid: false,
            cached_total_1m: None,
            cached_total_5m: None,
            cached_total_15m: None,
            cached_hottest_1m: None,
            cached_hottest_5m: None,
            cached_hottest_15m: None,
            cached_revision: 0,
        }
    }
    pub fn push(&mut self, total_pct: f32, hottest_pct: f32, timestamp_s: f64) {
        if let Some(last_ts) = self.timestamps.back() {
            if timestamp_s <= *last_ts {
                return;
            }
        }
        self.peak_total_percent = self.peak_total_percent.max(total_pct);
        self.peak_hottest_percent = self.peak_hottest_percent.max(hottest_pct);
        if self.total_points.len() == CPU_HISTORY_CAPACITY {
            self.total_points.pop_front();
        }
        if self.hottest_points.len() == CPU_HISTORY_CAPACITY {
            self.hottest_points.pop_front();
        }
        if self.timestamps.len() == CPU_HISTORY_CAPACITY {
            self.timestamps.pop_front();
        }
        self.total_points.push_back(total_pct);
        self.hottest_points.push_back(hottest_pct);
        self.timestamps.push_back(timestamp_s);
        self.revision = self.revision.wrapping_add(1);
        self.invalidate_caches();
    }
    #[inline]
    fn invalidate_caches(&mut self) {
        self.cached_total_1m = None;
        self.cached_total_5m = None;
        self.cached_total_15m = None;
        self.cached_hottest_1m = None;
        self.cached_hottest_5m = None;
        self.cached_hottest_15m = None;
        self.cached_revision = self.revision;
    }
    #[allow(dead_code)]
    fn get_total_series(&mut self, window: CpuChartWindow) -> Arc<Vec<[f64; 2]>> {
        let cached = match window {
            CpuChartWindow::OneMin => self.cached_total_1m.clone(),
            CpuChartWindow::FiveMin => self.cached_total_5m.clone(),
            CpuChartWindow::FifteenMin => self.cached_total_15m.clone(),
        };
        if let Some(c) = cached {
            return c;
        }
        let points = self.build_windowed_series(&self.total_points, window);
        let arc = Arc::new(lttb_downsample(&points, 920));
        match window {
            CpuChartWindow::OneMin => self.cached_total_1m = Some(arc.clone()),
            CpuChartWindow::FiveMin => self.cached_total_5m = Some(arc.clone()),
            CpuChartWindow::FifteenMin => self.cached_total_15m = Some(arc.clone()),
        };
        arc
    }
    #[allow(dead_code)]
    fn get_hottest_series(&mut self, window: CpuChartWindow) -> Arc<Vec<[f64; 2]>> {
        let cached = match window {
            CpuChartWindow::OneMin => self.cached_hottest_1m.clone(),
            CpuChartWindow::FiveMin => self.cached_hottest_5m.clone(),
            CpuChartWindow::FifteenMin => self.cached_hottest_15m.clone(),
        };
        if let Some(c) = cached {
            return c;
        }
        let points = self.build_windowed_series(&self.hottest_points, window);
        let arc = Arc::new(lttb_downsample(&points, 920));
        match window {
            CpuChartWindow::OneMin => self.cached_hottest_1m = Some(arc.clone()),
            CpuChartWindow::FiveMin => self.cached_hottest_5m = Some(arc.clone()),
            CpuChartWindow::FifteenMin => self.cached_hottest_15m = Some(arc.clone()),
        };
        arc
    }
    #[allow(dead_code, clippy::needless_range_loop)]
    fn build_windowed_series(
        &self,
        points: &VecDeque<f32>,
        window: CpuChartWindow,
    ) -> Vec<[f64; 2]> {
        if self.timestamps.is_empty() || points.is_empty() {
            return Vec::new();
        }
        let now = self.latest_timestamp();
        let window_secs = window.seconds() as f64;
        let cutoff = now - window_secs;
        let len = points.len();
        let timestamps = &self.timestamps;
        let mut start_idx = 0;
        for i in 0..len {
            if now - timestamps[i] <= window_secs {
                start_idx = i;
                break;
            }
        }
        let mut out = Vec::with_capacity(len - start_idx + 1);
        for i in start_idx..len {
            out.push([timestamps[i], points[i] as f64]);
        }
        // Clean startup: only anchor when real history predates cutoff.
        if start_idx > 0 && out.len() > 1 && out[0][0] > cutoff {
            let prev_y = points[start_idx - 1] as f64;
            out.insert(0, [cutoff, prev_y]);
        }
        if let Some(last) = out.last().copied() {
            if last[0] < now {
                out.push([now, last[1]]);
            }
        }
        out
    }
    #[inline]
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.total_points.len()
    }
    #[inline]
    pub fn latest_timestamp(&self) -> f64 {
        self.timestamps.back().copied().unwrap_or(0.0)
    }

    pub fn show_core_grid(&self) -> bool {
        self.show_core_grid
    }

    pub fn set_show_core_grid(&mut self, show: bool) {
        self.show_core_grid = show;
    }
    #[allow(dead_code)]
    pub fn total_series(&mut self, window: CpuChartWindow) -> Arc<Vec<[f64; 2]>> {
        self.get_total_series(window)
    }
    #[allow(dead_code)]
    pub fn hottest_series(&mut self, window: CpuChartWindow) -> Arc<Vec<[f64; 2]>> {
        self.get_hottest_series(window)
    }
    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.total_points.clear();
        self.hottest_points.clear();
        self.timestamps.clear();
        self.peak_total_percent = 0.0;
        self.peak_hottest_percent = 0.0;
        self.revision = self.revision.wrapping_add(1);
        self.invalidate_caches();
    }
    #[inline]
    #[allow(dead_code)]
    pub fn current_total_percent(&self) -> f32 {
        self.total_points.back().copied().unwrap_or(0.0)
    }
    #[inline]
    #[allow(dead_code)]
    pub fn current_hottest_percent(&self) -> f32 {
        self.hottest_points.back().copied().unwrap_or(0.0)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_cpu_history_push() {
        let mut hist = CpuHistory::new();
        hist.push(10.0, 20.0, 1.0);
        hist.push(20.0, 30.0, 2.0);
        hist.push(30.0, 40.0, 3.0);
        assert_eq!(hist.len(), 3);
        assert_eq!(hist.current_total_percent(), 30.0);
        assert_eq!(hist.current_hottest_percent(), 40.0);
        assert_eq!(hist.peak_total_percent, 30.0);
        assert_eq!(hist.peak_hottest_percent, 40.0);
    }
    #[test]
    fn test_cpu_history_windowed_output() {
        let mut hist = CpuHistory::new();
        for i in 0..100 {
            hist.push(i as f32 * 0.5, i as f32 * 0.3, i as f64);
        }
        let series = hist.total_series(CpuChartWindow::OneMin);
        assert!(!series.is_empty());
        let series5 = hist.total_series(CpuChartWindow::FiveMin);
        assert!(!series5.is_empty());
        let series15 = hist.total_series(CpuChartWindow::FifteenMin);
        assert!(!series15.is_empty());
    }
    #[test]
    fn test_short_history_starts_at_first_sample() {
        let mut hist = CpuHistory::new();
        for i in 0..10 {
            hist.push(10.0, 20.0, i as f64 - 9.0);
        }
        let series = hist.total_series(CpuChartWindow::OneMin);
        assert!(!series.is_empty());
        assert_eq!(series.first().map(|p| p[0]), Some(-9.0));
        assert_eq!(series.last().map(|p| p[0]), Some(0.0));
        let mut hist2 = CpuHistory::new();
        for i in 0..70 {
            hist2.push(i as f32, i as f32, i as f64);
        }
        let series2 = hist2.total_series(CpuChartWindow::OneMin);
        assert_eq!(series2.first().map(|p| p[0]), Some(9.0));
        assert_eq!(series2.last().map(|p| p[0]), Some(69.0));
    }
    #[test]
    fn test_duplicate_timestamp_ignored() {
        let mut hist = CpuHistory::new();
        hist.push(10.0, 20.0, 1.0);
        hist.push(99.0, 99.0, 1.0);
        hist.push(99.0, 99.0, 0.5);
        assert_eq!(hist.len(), 1);
        assert_eq!(hist.current_total_percent(), 10.0);
    }
}
