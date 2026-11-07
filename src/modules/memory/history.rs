//! Memory history ring buffer and chart window types.

use std::collections::VecDeque;
use std::sync::Arc;

use super::model::{ChartWindow, MemorySnapshot, HISTORY_CAPACITY};
use crate::modules::memory::downsampling::lttb_downsample;

/// Ring buffer for memory usage history. Stores up to `HISTORY_CAPACITY` (960) points
/// (RAM/Swap/Commit %) at 1s interval covering 15 min. Tracks session peaks.
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
            ram_points: VecDeque::with_capacity(HISTORY_CAPACITY),
            swap_points: VecDeque::with_capacity(HISTORY_CAPACITY),
            commit_points: VecDeque::with_capacity(HISTORY_CAPACITY),
            peak_ram_percent: 0.0,
            peak_swap_percent: 0.0,
            chart_window: ChartWindow::default(),
            revision: 0,
            timestamps: VecDeque::with_capacity(HISTORY_CAPACITY),
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
            if snap.sample_elapsed_secs <= last {
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
        if self.ram_points.len() == HISTORY_CAPACITY {
            self.ram_points.pop_front();
        }
        if self.swap_points.len() == HISTORY_CAPACITY {
            self.swap_points.pop_front();
        }
        if self.commit_points.len() == HISTORY_CAPACITY {
            self.commit_points.pop_front();
        }
        if self.timestamps.len() == HISTORY_CAPACITY {
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
        let cutoff = now - window_secs;
        if start_idx > 0 && out.len() > 1 && out[0][0] > cutoff {
            let prev_y = points[start_idx - 1] as f64;
            out.insert(0, [cutoff, prev_y]);
        }
        out
    }

    #[inline]
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.ram_points.len()
    }
    #[inline]
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.ram_points.is_empty()
    }
    #[allow(dead_code)]
    pub fn windowed_ram_points(&self) -> impl Iterator<Item = f32> + '_ {
        let ws = self.chart_window.seconds() as usize;
        let skip = self.ram_points.len().saturating_sub(ws);
        self.ram_points.iter().skip(skip).copied()
    }
    #[inline]
    #[allow(dead_code)]
    pub fn revision(&self) -> u64 {
        self.revision
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

    #[allow(dead_code)]
    pub fn windowed_ram_timed(&self) -> impl Iterator<Item = (f64, f32)> + '_ {
        let now = self.latest_timestamp();
        let window = self.chart_window.seconds() as f64;
        self.timestamps
            .iter()
            .zip(self.ram_points.iter())
            .filter(move |(t, _)| now - **t <= window)
            .map(|(t, v)| (*t, *v))
    }
    #[allow(dead_code)]
    pub fn windowed_swap_timed(&self) -> impl Iterator<Item = (f64, f32)> + '_ {
        let now = self.latest_timestamp();
        let window = self.chart_window.seconds() as f64;
        self.timestamps
            .iter()
            .zip(self.swap_points.iter())
            .filter(move |(t, _)| now - **t <= window)
            .map(|(t, v)| (*t, *v))
    }

    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.ram_points.clear();
        self.swap_points.clear();
        self.commit_points.clear();
        self.timestamps.clear();
        self.peak_ram_percent = 0.0;
        self.peak_swap_percent = 0.0;
        self.revision = self.revision.wrapping_add(1);
        self.invalidate_caches();
    }

    #[inline]
    pub fn current_ram_percent(&self) -> f32 {
        self.ram_points.back().copied().unwrap_or(0.0)
    }
    #[inline]
    #[allow(dead_code)]
    pub fn current_swap_percent(&self) -> f32 {
        self.swap_points.back().copied().unwrap_or(0.0)
    }
}
