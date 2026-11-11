//! CPU history ring buffer and chart window types.
//! Stores 960 samples (15 minutes at 1s interval) of dual series: total_pct + hottest_core_pct.
use crate::modules::memory::downsampling::lttb_downsample;
use crate::modules::memory::model::ChartWindow;
use std::collections::VecDeque;
use std::sync::Arc;

pub const CPU_HISTORY_CAPACITY: usize = 960;
/// Re-export memory's ChartWindow for CPU charts (identical values: 60, 300, 900).
pub type CpuChartWindow = ChartWindow;
type SeriesCache = Option<(u64, Arc<Vec<[f64; 2]>>)>;

fn push_gap_anchored(pts: &mut Vec<[f64; 2]>, x: f64, y: f64) {
    if let Some(&[px, _]) = pts.last() {
        if x - px > 4.5 {
            pts.extend([[px + 0.01, 0.0], [x - 0.01, 0.0]]);
        }
    }
    pts.push([x, y]);
}

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
    cache_total_1m: SeriesCache,
    cache_total_5m: SeriesCache,
    cache_total_15m: SeriesCache,
    cache_hottest_1m: SeriesCache,
    cache_hottest_5m: SeriesCache,
    cache_hottest_15m: SeriesCache,
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
            cache_total_1m: None,
            cache_total_5m: None,
            cache_total_15m: None,
            cache_hottest_1m: None,
            cache_hottest_5m: None,
            cache_hottest_15m: None,
        }
    }

    pub fn push(&mut self, total_pct: f32, hottest_pct: f32, timestamp_s: f64) {
        if let Some(last_ts) = self.timestamps.back() {
            if timestamp_s <= *last_ts {
                return;
            }
        }
        let evicted = if self.total_points.len() >= CPU_HISTORY_CAPACITY {
            let et = self.total_points.pop_front();
            let eh = self.hottest_points.pop_front();
            let _ = self.timestamps.pop_front();
            match (et, eh) {
                (Some(t), Some(h)) => Some((t, h)),
                _ => None,
            }
        } else {
            None
        };
        self.total_points.push_back(total_pct);
        self.hottest_points.push_back(hottest_pct);
        self.timestamps.push_back(timestamp_s);
        self.peak_total_percent = self.peak_total_percent.max(total_pct);
        self.peak_hottest_percent = self.peak_hottest_percent.max(hottest_pct);
        if let Some((ev_t, ev_h)) = evicted {
            if ev_t >= self.peak_total_percent || ev_h >= self.peak_hottest_percent {
                self.peak_total_percent = self.total_points.iter().copied().fold(0.0_f32, f32::max);
                self.peak_hottest_percent =
                    self.hottest_points.iter().copied().fold(0.0_f32, f32::max);
            }
        }
        self.revision = self.revision.wrapping_add(1);
    }

    #[allow(dead_code)]
    fn get_series(&mut self, window: CpuChartWindow, hottest: bool) -> Arc<Vec<[f64; 2]>> {
        let rev = self.revision;
        let slot_ro: &SeriesCache = match (hottest, window) {
            (false, CpuChartWindow::OneMin) => &self.cache_total_1m,
            (false, CpuChartWindow::FiveMin) => &self.cache_total_5m,
            (false, CpuChartWindow::FifteenMin) => &self.cache_total_15m,
            (true, CpuChartWindow::OneMin) => &self.cache_hottest_1m,
            (true, CpuChartWindow::FiveMin) => &self.cache_hottest_5m,
            (true, CpuChartWindow::FifteenMin) => &self.cache_hottest_15m,
        };
        if let Some((r, arc)) = slot_ro {
            if *r == rev {
                return Arc::clone(arc);
            }
        }
        let src = if hottest { &self.hottest_points } else { &self.total_points };
        let arc = Arc::new(lttb_downsample(&self.build_windowed_series(src, window), 920));
        let slot: &mut SeriesCache = match (hottest, window) {
            (false, CpuChartWindow::OneMin) => &mut self.cache_total_1m,
            (false, CpuChartWindow::FiveMin) => &mut self.cache_total_5m,
            (false, CpuChartWindow::FifteenMin) => &mut self.cache_total_15m,
            (true, CpuChartWindow::OneMin) => &mut self.cache_hottest_1m,
            (true, CpuChartWindow::FiveMin) => &mut self.cache_hottest_5m,
            (true, CpuChartWindow::FifteenMin) => &mut self.cache_hottest_15m,
        };
        *slot = Some((rev, Arc::clone(&arc)));
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
            push_gap_anchored(&mut out, timestamps[i], points[i] as f64);
        }
        if start_idx > 0 && out.len() > 1 && out[0][0] > cutoff {
            out.insert(0, [cutoff, points[start_idx - 1] as f64]);
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
        self.get_series(window, false)
    }
    #[allow(dead_code)]
    pub fn hottest_series(&mut self, window: CpuChartWindow) -> Arc<Vec<[f64; 2]>> {
        self.get_series(window, true)
    }
    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.total_points.clear();
        self.hottest_points.clear();
        self.timestamps.clear();
        self.peak_total_percent = 0.0;
        self.peak_hottest_percent = 0.0;
        self.revision = self.revision.wrapping_add(1);
        self.cache_total_1m = None;
        self.cache_total_5m = None;
        self.cache_total_15m = None;
        self.cache_hottest_1m = None;
        self.cache_hottest_5m = None;
        self.cache_hottest_15m = None;
    }
    #[inline]
    #[allow(dead_code)]
    pub fn current_total_percent(&self) -> f32 {
        self.total_points.back().copied().unwrap_or(0.0)
    }
    #[inline]
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
        assert!(!hist.total_series(CpuChartWindow::OneMin).is_empty());
        assert!(!hist.total_series(CpuChartWindow::FiveMin).is_empty());
        assert!(!hist.total_series(CpuChartWindow::FifteenMin).is_empty());
    }
    #[test]
    fn test_short_history_starts_at_first_sample() {
        let mut hist = CpuHistory::new();
        for i in 0..10 {
            hist.push(10.0, 20.0, i as f64 - 9.0);
        }
        let series = hist.total_series(CpuChartWindow::OneMin);
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
    #[test]
    fn test_peak_total_percent_decays_after_spike_evicted() {
        let mut hist = CpuHistory::new();
        hist.push(99.0, 50.0, 1.0);
        for i in 2..=(CPU_HISTORY_CAPACITY as i32 + 1) {
            hist.push(10.0, 5.0, i as f64);
        }
        assert_eq!(hist.len(), CPU_HISTORY_CAPACITY);
        assert_eq!(hist.peak_total_percent, 10.0);
    }
}
