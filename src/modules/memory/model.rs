//! Pure mathematical model for memory telemetry — zero dependencies on egui/sysinfo.
//!
//! This module provides the core data structures and pure functions for memory
//! monitoring. It is completely independent of any GUI or OS-specific libraries,
//! making it trivially testable and portable.

use std::collections::VecDeque;

/// Number of history points to retain (120 points = 60 seconds at 500ms interval).
#[allow(dead_code)]
const HISTORY_CAPACITY: usize = 120;

/// A single snapshot of system memory state.
///
/// All values are in **bytes** (`u64`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MemorySnapshot {
    /// Total physical RAM in bytes.
    pub ram_total_bytes: u64,
    /// Currently used physical RAM in bytes.
    pub ram_used_bytes: u64,
    /// Available physical RAM in bytes (free + standby/cached).
    pub ram_available_bytes: u64,
    /// Free physical RAM in bytes (completely unused).
    pub ram_free_bytes: u64,
    /// Total swap (pagefile) size in bytes.
    pub swap_total_bytes: u64,
    /// Currently used swap in bytes.
    pub swap_used_bytes: u64,
    /// Free swap in bytes.
    pub swap_free_bytes: u64,

    // ===== v0.0.6: Windows Kernel Metrics (from K32GetPerformanceInfo) =====
    /// Total commit charge (pages committed) in bytes.
    pub commit_total_bytes: u64,
    /// Commit limit (max commit charge) in bytes.
    pub commit_limit_bytes: u64,
    /// Kernel paged pool in bytes.
    pub kernel_paged_bytes: u64,
    /// Kernel non-paged pool in bytes.
    pub kernel_nonpaged_bytes: u64,
    /// System-wide handle count.
    pub system_handles: u32,
    /// System-wide thread count.
    pub system_threads: u32,
    /// System-wide process count.
    pub system_processes: u32,
    /// Real Windows System Cache (from K32GetPerformanceInfo.SystemCache * PageSize).
    /// When > 0, this overrides the calculated `available - free` fallback.
    pub ram_cached_bytes_override: u64,
}

impl MemorySnapshot {
    /// Returns the system cache (Standby / Cached RAM) in bytes.
    ///
    /// If `ram_cached_bytes_override` is set (from K32GetPerformanceInfo on Windows),
    /// returns that real kernel value. Otherwise falls back to calculated
    /// `available - free` using **saturating subtraction** to avoid panic
    /// if `free > available` (race conditions in OS reporting).
    ///
    /// # Formula
    /// `cached = override.or(available.saturating_sub(free))`
    #[inline]
    pub fn ram_cached_bytes(&self) -> u64 {
        if self.ram_cached_bytes_override > 0 {
            self.ram_cached_bytes_override
        } else {
            self.ram_available_bytes.saturating_sub(self.ram_free_bytes)
        }
    }

    /// Converts bytes to gibibytes (GiB) as `f64`.
    ///
    /// 1 GiB = 1,073,741,824 bytes (2^30).
    #[inline]
    pub fn bytes_to_gib(bytes: u64) -> f64 {
        bytes as f64 / 1_073_741_824.0
    }

    /// Converts bytes to mebibytes (MiB) as `f64`.
    ///
    /// 1 MiB = 1,048,576 bytes (2^20).
    #[allow(dead_code)]
    #[inline]
    pub fn bytes_to_mib(bytes: u64) -> f64 {
        bytes as f64 / 1_048_576.0
    }

    /// Returns RAM usage as percentage, clamped to `[0.0, 100.0]`.
    ///
    /// Returns `0.0` if `ram_total_bytes == 0` (division by zero guard).
    #[allow(dead_code)]
    #[inline]
    pub fn ram_usage_percent(&self) -> f32 {
        if self.ram_total_bytes == 0 {
            return 0.0;
        }
        let pct = (self.ram_used_bytes as f64 / self.ram_total_bytes as f64) * 100.0;
        pct.clamp(0.0, 100.0) as f32
    }

    /// Returns Swap usage as percentage, clamped to `[0.0, 100.0]`.
    ///
    /// Returns `0.0` if `swap_total_bytes == 0` (swap disabled / division by zero guard).
    #[allow(dead_code)]
    #[inline]
    pub fn swap_usage_percent(&self) -> f32 {
        if self.swap_total_bytes == 0 {
            return 0.0;
        }
        let pct = (self.swap_used_bytes as f64 / self.swap_total_bytes as f64) * 100.0;
        pct.clamp(0.0, 100.0) as f32
    }

    /// Returns Commit usage as percentage, clamped to `[0.0, 100.0]`.
    ///
    /// Returns `0.0` if `commit_limit_bytes == 0`.
    #[allow(dead_code)]
    #[inline]
    pub fn commit_usage_percent(&self) -> f32 {
        if self.commit_limit_bytes == 0 {
            return 0.0;
        }
        let pct = (self.commit_total_bytes as f64 / self.commit_limit_bytes as f64) * 100.0;
        pct.clamp(0.0, 100.0) as f32
    }

    /// Returns RAM total in GiB.
    #[allow(dead_code)]
    #[inline]
    pub fn ram_total_gib(&self) -> f64 {
        Self::bytes_to_gib(self.ram_total_bytes)
    }

    /// Returns RAM used in GiB.
    #[allow(dead_code)]
    #[inline]
    pub fn ram_used_gib(&self) -> f64 {
        Self::bytes_to_gib(self.ram_used_bytes)
    }

    /// Returns RAM available in GiB.
    #[allow(dead_code)]
    #[inline]
    pub fn ram_available_gib(&self) -> f64 {
        Self::bytes_to_gib(self.ram_available_bytes)
    }

    /// Returns RAM cached in GiB.
    #[allow(dead_code)]
    #[inline]
    pub fn ram_cached_gib(&self) -> f64 {
        Self::bytes_to_gib(self.ram_cached_bytes())
    }

    /// Returns Swap total in GiB.
    #[allow(dead_code)]
    #[inline]
    pub fn swap_total_gib(&self) -> f64 {
        Self::bytes_to_gib(self.swap_total_bytes)
    }

    /// Returns Swap used in GiB.
    #[allow(dead_code)]
    #[inline]
    pub fn swap_used_gib(&self) -> f64 {
        Self::bytes_to_gib(self.swap_used_bytes)
    }

    /// Returns Swap free in GiB.
    #[allow(dead_code)]
    #[inline]
    pub fn swap_free_gib(&self) -> f64 {
        Self::bytes_to_gib(self.swap_free_bytes)
    }

    /// Returns Commit total in GiB.
    #[allow(dead_code)]
    #[inline]
    pub fn commit_total_gib(&self) -> f64 {
        Self::bytes_to_gib(self.commit_total_bytes)
    }

    /// Returns Commit limit in GiB.
    #[allow(dead_code)]
    #[inline]
    pub fn commit_limit_gib(&self) -> f64 {
        Self::bytes_to_gib(self.commit_limit_bytes)
    }

    /// Returns Kernel paged pool in MiB.
    #[allow(dead_code)]
    #[inline]
    pub fn kernel_paged_mib(&self) -> f64 {
        Self::bytes_to_mib(self.kernel_paged_bytes)
    }

    /// Returns Kernel non-paged pool in MiB.
    #[allow(dead_code)]
    #[inline]
    pub fn kernel_nonpaged_mib(&self) -> f64 {
        Self::bytes_to_mib(self.kernel_nonpaged_bytes)
    }

    /// Returns total kernel pool (paged + non-paged) in MiB.
    #[allow(dead_code)]
    #[inline]
    pub fn kernel_total_mib(&self) -> f64 {
        Self::bytes_to_mib(self.kernel_paged_bytes + self.kernel_nonpaged_bytes)
    }
}

/// Ring buffer (кільцевий буфер) for memory usage history.
///
/// Stores up to `HISTORY_CAPACITY` (120) data points representing
/// RAM and Swap usage percentages. At 500ms interval this covers
/// 60 seconds of history.
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

    /// Clears all history and resets peaks.
    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.ram_points.clear();
        self.swap_points.clear();
        self.peak_ram_percent = 0.0;
        self.peak_swap_percent = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_swap_division_by_zero_guard() {
        // Swap disabled (total == 0) should return 0.0, not panic
        let snap = MemorySnapshot {
            swap_total_bytes: 0,
            swap_used_bytes: 100,
            swap_free_bytes: 0,
            ..Default::default()
        };
        assert_eq!(snap.swap_usage_percent(), 0.0);

        // Normal case
        let snap2 = MemorySnapshot {
            swap_total_bytes: 1000,
            swap_used_bytes: 500,
            swap_free_bytes: 500,
            ..Default::default()
        };
        assert!((snap2.swap_usage_percent() - 50.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_commit_division_by_zero_guard() {
        let snap = MemorySnapshot {
            commit_limit_bytes: 0,
            commit_total_bytes: 100,
            ..Default::default()
        };
        assert_eq!(snap.commit_usage_percent(), 0.0);

        let snap2 = MemorySnapshot {
            commit_limit_bytes: 1000,
            commit_total_bytes: 500,
            ..Default::default()
        };
        assert!((snap2.commit_usage_percent() - 50.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_ram_cached_bytes_saturating_sub() {
        // Normal case: available > free
        let snap = MemorySnapshot {
            ram_available_bytes: 8_000_000_000,
            ram_free_bytes: 2_000_000_000,
            ..Default::default()
        };
        assert_eq!(snap.ram_cached_bytes(), 6_000_000_000);

        // Edge case: free > available (race condition in OS reporting)
        // saturating_sub prevents panic, returns 0
        let snap2 = MemorySnapshot {
            ram_available_bytes: 2_000_000_000,
            ram_free_bytes: 8_000_000_000,
            ..Default::default()
        };
        assert_eq!(snap2.ram_cached_bytes(), 0);
    }

    #[test]
    fn test_bytes_to_gib_mib_conversion() {
        const GIB: u64 = 1_073_741_824;
        const MIB: u64 = 1_048_576;

        assert!((MemorySnapshot::bytes_to_gib(GIB) - 1.0).abs() < f64::EPSILON);
        assert!((MemorySnapshot::bytes_to_gib(2 * GIB) - 2.0).abs() < f64::EPSILON);
        assert!((MemorySnapshot::bytes_to_gib(0) - 0.0).abs() < f64::EPSILON);

        assert!((MemorySnapshot::bytes_to_mib(MIB) - 1.0).abs() < f64::EPSILON);
        assert!((MemorySnapshot::bytes_to_mib(512 * MIB) - 512.0).abs() < f64::EPSILON);
        assert!((MemorySnapshot::bytes_to_mib(0) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_usage_percent_clamping() {
        // Over 100% (should not happen, but clamp protects)
        let snap = MemorySnapshot {
            ram_total_bytes: 1000,
            ram_used_bytes: 1500, // > total
            ..Default::default()
        };
        assert_eq!(snap.ram_usage_percent(), 100.0);

        // Under 0% (impossible, but clamp protects)
        let snap2 = MemorySnapshot {
            ram_total_bytes: 1000,
            ram_used_bytes: 0,
            ..Default::default()
        };
        assert_eq!(snap2.ram_usage_percent(), 0.0);

        // Normal case
        let snap3 = MemorySnapshot {
            ram_total_bytes: 1000,
            ram_used_bytes: 500,
            ..Default::default()
        };
        assert!((snap3.ram_usage_percent() - 50.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_memory_history_ring_buffer_120() {
        let mut history = MemoryHistory::new();
        let base_snap = MemorySnapshot {
            ram_total_bytes: 1000,
            ram_used_bytes: 500,
            swap_total_bytes: 1000,
            swap_used_bytes: 250,
            ..Default::default()
        };

        // Push 150 points (capacity is 120)
        for i in 0..150 {
            let mut snap = base_snap.clone();
            snap.ram_used_bytes = 500 + (i % 10) as u64 * 10; // Vary slightly
            snap.swap_used_bytes = 250 + (i % 5) as u64 * 10;
            history.push(&snap);
        }

        // Should only keep last 120 points
        assert_eq!(history.len(), HISTORY_CAPACITY);
        assert_eq!(history.ram_points.len(), HISTORY_CAPACITY);
        assert_eq!(history.swap_points.len(), HISTORY_CAPACITY);

        // Peaks should be preserved from the 150 points
        assert!(history.peak_ram_percent >= 50.0);
        assert!(history.peak_swap_percent >= 25.0);
    }

    #[test]
    fn test_lang_set_uk_en() {
        use crate::i18n::{current_lang, set_lang, Lang};

        set_lang(Lang::Uk);
        assert_eq!(current_lang(), Lang::Uk);

        set_lang(Lang::En);
        assert_eq!(current_lang(), Lang::En);

        set_lang(Lang::Uk);
        assert_eq!(current_lang(), Lang::Uk);
    }

    #[test]
    fn test_commit_usage_percent() {
        let snap = MemorySnapshot {
            commit_total_bytes: 5_000_000_000,
            commit_limit_bytes: 10_000_000_000,
            ..Default::default()
        };
        assert!((snap.commit_usage_percent() - 50.0).abs() < f32::EPSILON);

        let snap2 = MemorySnapshot {
            commit_total_bytes: 12_000_000_000,
            commit_limit_bytes: 10_000_000_000,
            ..Default::default()
        };
        assert_eq!(snap2.commit_usage_percent(), 100.0);
    }
}
