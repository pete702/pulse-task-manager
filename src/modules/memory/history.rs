//! Memory history ring buffer and chart window types.

use std::collections::VecDeque;

use super::model::{ChartWindow, MemorySnapshot, HISTORY_CAPACITY};

/// Ring buffer for memory usage history.
///
/// Stores up to `HISTORY_CAPACITY` (900) data points representing
/// RAM, Swap and Commit usage percentages. At 1s interval this covers
/// 15 minutes of history. Also tracks session peaks.
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
}

impl MemoryHistory {
    #[allow(dead_code)]
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
        }
    }

    #[allow(dead_code)]
    pub fn push(&mut self, snap: &MemorySnapshot) {
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
        let ts = if snap.sample_elapsed_secs > 0.0 {
            snap.sample_elapsed_secs
        } else {
            self.timestamps.back().copied().unwrap_or(0.0) + 1.0
        };
        self.ram_points.push_back(ram_pct);
        self.swap_points.push_back(swap_pct);
        self.commit_points.push_back(commit_pct);
        self.timestamps.push_back(ts);
        self.revision = self.revision.wrapping_add(1);
    }

    #[allow(dead_code)]
    #[inline]
    pub fn len(&self) -> usize {
        self.ram_points.len()
    }
    #[allow(dead_code)]
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.ram_points.is_empty()
    }
    #[allow(dead_code)]
    #[inline]
    pub fn ram_iter(&self) -> impl Iterator<Item = f32> + '_ {
        self.ram_points.iter().copied()
    }
    #[allow(dead_code)]
    #[inline]
    pub fn swap_iter(&self) -> impl Iterator<Item = f32> + '_ {
        self.swap_points.iter().copied()
    }
    #[allow(dead_code)]
    pub fn windowed_ram_points(&self) -> impl Iterator<Item = f32> + '_ {
        let ws = self.chart_window.seconds() as usize;
        let skip = self.ram_points.len().saturating_sub(ws);
        self.ram_points.iter().skip(skip).copied()
    }
    #[allow(dead_code)]
    pub fn windowed_swap_points(&self) -> impl Iterator<Item = f32> + '_ {
        let ws = self.chart_window.seconds() as usize;
        let skip = self.swap_points.len().saturating_sub(ws);
        self.swap_points.iter().skip(skip).copied()
    }
    #[allow(dead_code)]
    #[inline]
    pub fn revision(&self) -> u64 {
        self.revision
    }
    #[inline]
    pub fn latest_timestamp(&self) -> f64 {
        self.timestamps.back().copied().unwrap_or(0.0)
    }

    pub fn windowed_ram_timed(&self) -> impl Iterator<Item = (f64, f32)> + '_ {
        let now = self.latest_timestamp();
        let window = self.chart_window.seconds() as f64;
        self.timestamps
            .iter()
            .zip(self.ram_points.iter())
            .filter(move |(t, _)| now - **t <= window)
            .map(|(t, v)| (*t, *v))
    }
    pub fn windowed_swap_timed(&self) -> impl Iterator<Item = (f64, f32)> + '_ {
        let now = self.latest_timestamp();
        let window = self.chart_window.seconds() as f64;
        self.timestamps
            .iter()
            .zip(self.swap_points.iter())
            .filter(move |(t, _)| now - **t <= window)
            .map(|(t, v)| (*t, *v))
    }
    pub fn windowed_commit_timed(&self) -> impl Iterator<Item = (f64, f32)> + '_ {
        let now = self.latest_timestamp();
        let window = self.chart_window.seconds() as f64;
        self.timestamps
            .iter()
            .zip(self.commit_points.iter())
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
    }

    #[allow(dead_code)]
    #[inline]
    pub fn current_ram_percent(&self) -> f32 {
        self.ram_points.back().copied().unwrap_or(0.0)
    }
    #[allow(dead_code)]
    #[inline]
    pub fn current_swap_percent(&self) -> f32 {
        self.swap_points.back().copied().unwrap_or(0.0)
    }
}
