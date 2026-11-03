//! Memory history ring buffer and chart window types.

use std::collections::VecDeque;

use super::model::{ChartWindow, MemorySnapshot, HISTORY_CAPACITY};

/// Ring buffer for memory usage history.
///
/// Stores up to `HISTORY_CAPACITY` (900) data points representing
/// RAM and Swap usage percentages. At 1s interval this covers
/// 15 minutes of history.
///
/// Also tracks session peaks for both RAM and Swap.
#[derive(Clone, Debug, Default)]
pub struct MemoryHistory {
    /// RAM usage percentage history (0.0–100.0).
    pub ram_points: VecDeque<f32>,
    /// Swap usage percentage history (0.0–100.0).
    pub swap_points: VecDeque<f32>,
    /// Peak RAM usage percentage this session.
    pub peak_ram_percent: f32,
    /// Peak Swap usage percentage this session.
    pub peak_swap_percent: f32,
    /// Currently selected chart time window.
    pub chart_window: ChartWindow,
}

impl MemoryHistory {
    /// Creates a new empty history buffer.
    #[allow(dead_code)]
    #[inline]
    pub fn new() -> Self {
        Self {
            ram_points: VecDeque::with_capacity(HISTORY_CAPACITY),
            swap_points: VecDeque::with_capacity(HISTORY_CAPACITY),
            peak_ram_percent: 0.0,
            peak_swap_percent: 0.0,
            chart_window: ChartWindow::default(),
        }
    }

    /// Pushes a new snapshot into the history ring buffer.
    ///
    /// If the buffer is at capacity (`HISTORY_CAPACITY`), the oldest
    /// point is evicted (FIFO) before pushing the new one.
    /// Session peaks are updated if the new values exceed them.
    #[allow(dead_code)]
    pub fn push(&mut self, snap: &MemorySnapshot) {
        let ram_pct = snap.ram_usage_percent();
        let swap_pct = snap.swap_usage_percent();

        // Update session peaks
        if ram_pct > self.peak_ram_percent {
            self.peak_ram_percent = ram_pct;
        }
        if swap_pct > self.peak_swap_percent {
            self.peak_swap_percent = swap_pct;
        }

        // Evict oldest if at capacity (ring buffer behavior)
        if self.ram_points.len() == HISTORY_CAPACITY {
            self.ram_points.pop_front();
        }
        if self.swap_points.len() == HISTORY_CAPACITY {
            self.swap_points.pop_front();
        }

        self.ram_points.push_back(ram_pct);
        self.swap_points.push_back(swap_pct);
    }

    /// Returns the current number of stored points.
    #[allow(dead_code)]
    #[inline]
    pub fn len(&self) -> usize {
        self.ram_points.len()
    }

    /// Returns `true` if history is empty.
    #[allow(dead_code)]
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.ram_points.is_empty()
    }

    /// Returns an iterator over RAM history points (oldest first).
    #[allow(dead_code)]
    #[inline]
    pub fn ram_iter(&self) -> impl Iterator<Item = f32> + '_ {
        self.ram_points.iter().copied()
    }

    /// Returns an iterator over Swap history points (oldest first).
    #[allow(dead_code)]
    #[inline]
    pub fn swap_iter(&self) -> impl Iterator<Item = f32> + '_ {
        self.swap_points.iter().copied()
    }

    /// Returns points for the currently selected chart window.
    #[allow(dead_code)]
    pub fn windowed_ram_points(&self) -> impl Iterator<Item = f32> + '_ {
        let window_size = self.chart_window.seconds() as usize;
        let skip = self.ram_points.len().saturating_sub(window_size);
        self.ram_points.iter().skip(skip).copied()
    }

    /// Returns points for the currently selected chart window.
    #[allow(dead_code)]
    pub fn windowed_swap_points(&self) -> impl Iterator<Item = f32> + '_ {
        let window_size = self.chart_window.seconds() as usize;
        let skip = self.swap_points.len().saturating_sub(window_size);
        self.swap_points.iter().skip(skip).copied()
    }

    /// Clears all history and resets peaks.
    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.ram_points.clear();
        self.swap_points.clear();
        self.peak_ram_percent = 0.0;
        self.peak_swap_percent = 0.0;
    }
}
