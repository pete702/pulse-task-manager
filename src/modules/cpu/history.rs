//! CPU history ring buffer and chart window types.
//! Stores up to 1920 samples (15 minutes at 2 Hz) of dual series: total_pct + hottest_core_pct.
use crate::core::downsampling::lttb_downsample;
use crate::core::ring_history;
use crate::modules::memory::model::ChartWindow;

pub const CPU_HISTORY_CAPACITY: usize = ring_history::HISTORY_CAPACITY;
use std::collections::VecDeque;
use std::sync::Arc;

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
    cache: [SeriesCache; 6],
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
            cache: Default::default(),
        }
    }

    fn invalidate_series_cache(&mut self) {
        self.cache = Default::default();
    }

    pub fn push(&mut self, total_pct: f32, hottest_pct: f32, timestamp_s: f64) {
        if let Some(&last_ts) = self.timestamps.back() {
            if !ring_history::is_newer(Some(last_ts), timestamp_s) {
                return;
            }
            if timestamp_s - last_ts < 0.85 {
                if let (Some(tp), Some(hp), Some(ts)) = (
                    self.total_points.back_mut(),
                    self.hottest_points.back_mut(),
                    self.timestamps.back_mut(),
                ) {
                    *tp = total_pct;
                    *hp = hottest_pct;
                    *ts = timestamp_s;
                }
                self.peak_total_percent = self.total_points.iter().copied().fold(0.0_f32, f32::max);
                self.peak_hottest_percent =
                    self.hottest_points.iter().copied().fold(0.0_f32, f32::max);
                self.revision = self.revision.wrapping_add(1);
                self.invalidate_series_cache();
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
        self.invalidate_series_cache();
    }

    fn get_series(&mut self, window: CpuChartWindow, hottest: bool) -> Arc<Vec<[f64; 2]>> {
        let rev = self.revision;
        let idx = match (hottest, window) {
            (false, CpuChartWindow::OneMin) => 0,
            (false, CpuChartWindow::FiveMin) => 1,
            (false, CpuChartWindow::FifteenMin) => 2,
            (true, CpuChartWindow::OneMin) => 3,
            (true, CpuChartWindow::FiveMin) => 4,
            (true, CpuChartWindow::FifteenMin) => 5,
        };
        if let Some((r, arc)) = &self.cache[idx] {
            if *r == rev {
                return Arc::clone(arc);
            }
        }
        let src = if hottest { &self.hottest_points } else { &self.total_points };
        let arc = Arc::new(lttb_downsample(&self.build_windowed_series(src, window), 920));
        self.cache[idx] = Some((rev, Arc::clone(&arc)));
        arc
    }

    #[allow(clippy::needless_range_loop)]
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
        let ts = &self.timestamps;
        let (start_idx, _) = ring_history::window_bounds(len, now, ts, window_secs);
        let mut out = Vec::with_capacity(len.saturating_sub(start_idx) + 1);
        for i in start_idx..len {
            push_gap_anchored(&mut out, ts[i], points[i] as f64);
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

    #[cfg(test)]
    #[inline]
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
    pub fn total_series(&mut self, window: CpuChartWindow) -> Arc<Vec<[f64; 2]>> {
        self.get_series(window, false)
    }
    pub fn hottest_series(&mut self, window: CpuChartWindow) -> Arc<Vec<[f64; 2]>> {
        self.get_series(window, true)
    }
    #[cfg(test)]
    #[inline]
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
        let mut h = CpuHistory::new();
        h.push(10.0, 20.0, 1.0);
        h.push(20.0, 30.0, 2.0);
        h.push(30.0, 40.0, 3.0);
        assert_eq!(h.len(), 3);
        assert_eq!(h.current_total_percent(), 30.0);
        assert_eq!(h.current_hottest_percent(), 40.0);
        assert_eq!(h.peak_total_percent, 30.0);
        assert_eq!(h.peak_hottest_percent, 40.0);
    }
    #[test]
    fn test_cpu_history_windowed_output() {
        let mut h = CpuHistory::new();
        for i in 0..100 {
            h.push(i as f32 * 0.5, i as f32 * 0.3, i as f64);
        }
        assert!(!h.total_series(CpuChartWindow::OneMin).is_empty());
        assert!(!h.total_series(CpuChartWindow::FiveMin).is_empty());
        assert!(!h.total_series(CpuChartWindow::FifteenMin).is_empty());
    }
    #[test]
    fn test_short_history_starts_at_first_sample() {
        let mut h = CpuHistory::new();
        for i in 0..10 {
            h.push(10.0, 20.0, i as f64 - 9.0);
        }
        let s = h.total_series(CpuChartWindow::OneMin);
        assert_eq!(s.first().map(|p| p[0]), Some(-9.0));
        assert_eq!(s.last().map(|p| p[0]), Some(0.0));
        let mut h2 = CpuHistory::new();
        for i in 0..70 {
            h2.push(i as f32, i as f32, i as f64);
        }
        let s2 = h2.total_series(CpuChartWindow::OneMin);
        assert_eq!(s2.first().map(|p| p[0]), Some(9.0));
        assert_eq!(s2.last().map(|p| p[0]), Some(69.0));
    }
    #[test]
    fn test_duplicate_timestamp_ignored() {
        let mut h = CpuHistory::new();
        h.push(10.0, 20.0, 1.0);
        h.push(99.0, 99.0, 1.0);
        h.push(99.0, 99.0, 0.5);
        assert_eq!(h.len(), 1);
        assert_eq!(h.current_total_percent(), 10.0);
    }
    #[test]
    fn test_peak_total_percent_decays_after_spike_evicted() {
        let mut h = CpuHistory::new();
        h.push(99.0, 50.0, 1.0);
        for i in 2..=(CPU_HISTORY_CAPACITY as i32 + 1) {
            h.push(10.0, 5.0, i as f64);
        }
        assert_eq!(h.len(), CPU_HISTORY_CAPACITY);
        assert_eq!(h.peak_total_percent, 10.0);
    }
}
