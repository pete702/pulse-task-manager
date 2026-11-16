//! Memory history ring buffer and chart window types.

use std::collections::VecDeque;
use std::sync::Arc;

use super::model::{ChartWindow, MemorySnapshot};
use crate::core::downsampling::lttb_downsample;
use crate::core::ring_history;

/// Ring buffer for memory usage history. Stores up to `ring_history::HISTORY_CAPACITY` (1920) points
/// (RAM/Swap/Commit %) covering 15 min at 2 Hz. Tracks session peaks.
#[derive(Clone, Debug, Default)]
pub struct MemoryHistory {
    pub ram_points: VecDeque<f32>,
    pub swap_points: VecDeque<f32>,
    pub commit_points: VecDeque<f32>,
    pub peak_ram_percent: f32,
    pub peak_swap_percent: f32,
    pub chart_window: ChartWindow,
    pub revision: u64,
    pub timestamps: VecDeque<f64>,
    cached_ram_1m: Option<Arc<Vec<[f64; 2]>>>,
    cached_ram_5m: Option<Arc<Vec<[f64; 2]>>>,
    cached_ram_15m: Option<Arc<Vec<[f64; 2]>>>,
    cached_swap_1m: Option<Arc<Vec<[f64; 2]>>>,
    cached_swap_5m: Option<Arc<Vec<[f64; 2]>>>,
    cached_swap_15m: Option<Arc<Vec<[f64; 2]>>>,
    cached_commit_1m: Option<Arc<Vec<[f64; 2]>>>,
    cached_commit_5m: Option<Arc<Vec<[f64; 2]>>>,
    cached_commit_15m: Option<Arc<Vec<[f64; 2]>>>,
    cached_revision: u64,
}

impl MemoryHistory {
    #[inline]
    pub fn new() -> Self {
        Self {
            ram_points: VecDeque::with_capacity(ring_history::HISTORY_CAPACITY),
            swap_points: VecDeque::with_capacity(ring_history::HISTORY_CAPACITY),
            commit_points: VecDeque::with_capacity(ring_history::HISTORY_CAPACITY),
            peak_ram_percent: 0.0,
            peak_swap_percent: 0.0,
            chart_window: ChartWindow::default(),
            revision: 0,
            timestamps: VecDeque::with_capacity(ring_history::HISTORY_CAPACITY),
            cached_ram_1m: None,
            cached_ram_5m: None,
            cached_ram_15m: None,
            cached_swap_1m: None,
            cached_swap_5m: None,
            cached_swap_15m: None,
            cached_commit_1m: None,
            cached_commit_5m: None,
            cached_commit_15m: None,
            cached_revision: 0,
        }
    }

    pub fn push(&mut self, snap: &MemorySnapshot) {
        if let Some(last) = self.timestamps.back().copied() {
            if !ring_history::is_newer(Some(last), snap.sample_elapsed_secs) {
                return;
            }
            if snap.sample_elapsed_secs - last < 0.85 {
                let ram_pct = snap.ram_usage_percent();
                let swap_pct = snap.swap_usage_percent();
                let commit_pct = snap.commit_usage_percent();
                if let Some(v) = self.ram_points.back_mut() {
                    *v = ram_pct;
                }
                if let Some(v) = self.swap_points.back_mut() {
                    *v = swap_pct;
                }
                if let Some(v) = self.commit_points.back_mut() {
                    *v = commit_pct;
                }
                if let Some(t) = self.timestamps.back_mut() {
                    *t = snap.sample_elapsed_secs;
                }
                self.peak_ram_percent = self.ram_points.iter().copied().fold(0.0_f32, f32::max);
                self.peak_swap_percent = self.swap_points.iter().copied().fold(0.0_f32, f32::max);
                self.revision = self.revision.wrapping_add(1);
                self.invalidate_caches();
                return;
            }
        }
        let ram_pct = snap.ram_usage_percent();
        let swap_pct = snap.swap_usage_percent();
        let commit_pct = snap.commit_usage_percent();
        if ram_pct > self.peak_ram_percent {
            self.peak_ram_percent = ram_pct;
        }
        if swap_pct > self.peak_swap_percent {
            self.peak_swap_percent = swap_pct;
        }
        if self.ram_points.len() == ring_history::HISTORY_CAPACITY {
            self.ram_points.pop_front();
        }
        if self.swap_points.len() == ring_history::HISTORY_CAPACITY {
            self.swap_points.pop_front();
        }
        if self.commit_points.len() == ring_history::HISTORY_CAPACITY {
            self.commit_points.pop_front();
        }
        if self.timestamps.len() == ring_history::HISTORY_CAPACITY {
            self.timestamps.pop_front();
        }
        self.ram_points.push_back(ram_pct);
        self.swap_points.push_back(swap_pct);
        self.commit_points.push_back(commit_pct);
        self.timestamps.push_back(snap.sample_elapsed_secs);
        self.revision = self.revision.wrapping_add(1);
        self.invalidate_caches();
    }

    #[inline]
    fn invalidate_caches(&mut self) {
        self.cached_ram_1m = None;
        self.cached_ram_5m = None;
        self.cached_ram_15m = None;
        self.cached_swap_1m = None;
        self.cached_swap_5m = None;
        self.cached_swap_15m = None;
        self.cached_commit_1m = None;
        self.cached_commit_5m = None;
        self.cached_commit_15m = None;
        self.cached_revision = self.revision;
    }

    fn get_ram_series(&mut self, window: ChartWindow) -> Arc<Vec<[f64; 2]>> {
        let cached = match window {
            ChartWindow::OneMin => self.cached_ram_1m.clone(),
            ChartWindow::FiveMin => self.cached_ram_5m.clone(),
            ChartWindow::FifteenMin => self.cached_ram_15m.clone(),
        };
        if let Some(c) = cached {
            return c;
        }
        let points = self.build_windowed_series(&self.ram_points, window);
        let arc = Arc::new(lttb_downsample(&points, 920));
        match window {
            ChartWindow::OneMin => self.cached_ram_1m = Some(arc.clone()),
            ChartWindow::FiveMin => self.cached_ram_5m = Some(arc.clone()),
            ChartWindow::FifteenMin => self.cached_ram_15m = Some(arc.clone()),
        };
        arc
    }

    fn get_swap_series(&mut self, window: ChartWindow) -> Arc<Vec<[f64; 2]>> {
        let cached = match window {
            ChartWindow::OneMin => self.cached_swap_1m.clone(),
            ChartWindow::FiveMin => self.cached_swap_5m.clone(),
            ChartWindow::FifteenMin => self.cached_swap_15m.clone(),
        };
        if let Some(c) = cached {
            return c;
        }
        let points = self.build_windowed_series(&self.swap_points, window);
        let arc = Arc::new(lttb_downsample(&points, 920));
        match window {
            ChartWindow::OneMin => self.cached_swap_1m = Some(arc.clone()),
            ChartWindow::FiveMin => self.cached_swap_5m = Some(arc.clone()),
            ChartWindow::FifteenMin => self.cached_swap_15m = Some(arc.clone()),
        };
        arc
    }

    fn get_commit_series(&mut self, window: ChartWindow) -> Arc<Vec<[f64; 2]>> {
        let cached = match window {
            ChartWindow::OneMin => self.cached_commit_1m.clone(),
            ChartWindow::FiveMin => self.cached_commit_5m.clone(),
            ChartWindow::FifteenMin => self.cached_commit_15m.clone(),
        };
        if let Some(c) = cached {
            return c;
        }
        let points = self.build_windowed_series(&self.commit_points, window);
        let arc = Arc::new(lttb_downsample(&points, 920));
        match window {
            ChartWindow::OneMin => self.cached_commit_1m = Some(arc.clone()),
            ChartWindow::FiveMin => self.cached_commit_5m = Some(arc.clone()),
            ChartWindow::FifteenMin => self.cached_commit_15m = Some(arc.clone()),
        };
        arc
    }

    /// Build raw windowed [timestamp, value] points. Clean startup avoids fake flat lines.
    #[allow(clippy::needless_range_loop)]
    fn build_windowed_series(&self, points: &VecDeque<f32>, window: ChartWindow) -> Vec<[f64; 2]> {
        if self.timestamps.is_empty() || points.is_empty() {
            return Vec::new();
        }
        let now = self.latest_timestamp();
        let window_secs = window.seconds() as f64;
        let len = points.len();
        let timestamps = &self.timestamps;
        let (start_idx, _) = ring_history::window_bounds(len, now, timestamps, window_secs);
        let mut out = Vec::with_capacity(len.saturating_sub(start_idx) + 1);
        for i in start_idx..len {
            out.push([timestamps[i], points[i] as f64]);
        }
        let cutoff = now - window_secs;
        if start_idx > 0 && out.len() > 1 && out[0][0] > cutoff {
            out.insert(0, [cutoff, points[start_idx - 1] as f64]);
        }
        // Startup single-point guard: treat <= 1 point as warm-up (no series).
        if out.len() <= 1 {
            return Vec::new();
        }
        out
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.ram_points.len()
    }
    #[cfg(test)]
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.ram_points.is_empty()
    }
    #[cfg(test)]
    pub fn windowed_ram_points(&self) -> impl Iterator<Item = f32> + '_ {
        let ws = self.chart_window.seconds() as usize;
        let skip = self.ram_points.len().saturating_sub(ws);
        self.ram_points.iter().skip(skip).copied()
    }
    #[cfg(test)]
    pub fn windowed_ram_timed(&self) -> impl Iterator<Item = (f64, f32)> + '_ {
        let now = self.latest_timestamp();
        let window = self.chart_window.seconds() as f64;
        self.timestamps
            .iter()
            .zip(self.ram_points.iter())
            .filter(move |(t, _)| now - **t <= window)
            .map(|(t, v)| (*t, *v))
    }
    #[cfg(test)]
    pub fn windowed_swap_timed(&self) -> impl Iterator<Item = (f64, f32)> + '_ {
        let now = self.latest_timestamp();
        let window = self.chart_window.seconds() as f64;
        self.timestamps
            .iter()
            .zip(self.swap_points.iter())
            .filter(move |(t, _)| now - **t <= window)
            .map(|(t, v)| (*t, *v))
    }
    #[inline]
    pub fn latest_timestamp(&self) -> f64 {
        self.timestamps.back().copied().unwrap_or(0.0)
    }
    pub fn ram_series(&mut self, window: ChartWindow) -> Arc<Vec<[f64; 2]>> {
        self.get_ram_series(window)
    }
    pub fn swap_series(&mut self, window: ChartWindow) -> Arc<Vec<[f64; 2]>> {
        self.get_swap_series(window)
    }
    pub fn commit_series(&mut self, window: ChartWindow) -> Arc<Vec<[f64; 2]>> {
        self.get_commit_series(window)
    }
    #[inline]
    pub fn current_ram_percent(&self) -> f32 {
        self.ram_points.back().copied().unwrap_or(0.0)
    }
}
