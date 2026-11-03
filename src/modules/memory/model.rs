//! Pure mathematical model for memory telemetry — zero dependencies on egui/sysinfo.
//!
//! This module provides the core data structures and pure functions for memory
//! monitoring. It is completely independent of any GUI or OS-specific libraries,
//! making it trivially testable and portable.

use crate::modules::memory::process_model::MemorySpikeEvent;

/// Number of history points to retain (900 points = 15 minutes at 1s interval).
#[allow(dead_code)]
pub const HISTORY_CAPACITY: usize = 900;

/// Time window for chart display.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[allow(clippy::enum_variant_names)]
pub enum ChartWindow {
    /// 1 minute (60 seconds / 60 points at 1s interval).
    #[default]
    OneMin = 60,
    /// 5 minutes (300 seconds / 300 points at 1s interval).
    FiveMin = 300,
    /// 15 minutes (900 seconds / 900 points at 1s interval).
    FifteenMin = 900,
}

impl ChartWindow {
    /// Returns the window duration in seconds.
    #[inline]
    pub fn seconds(self) -> u32 {
        self as u32
    }

    /// Returns English label for the button.
    pub fn label_en(self) -> &'static str {
        match self {
            ChartWindow::OneMin => "1m",
            ChartWindow::FiveMin => "5m",
            ChartWindow::FifteenMin => "15m",
        }
    }
}

/// A single snapshot of system memory state.
///
/// All values are in **bytes** (`u64`).
#[derive(Clone, Debug, Default, PartialEq)]
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

    // ===== v0.0.9: Process table data =====
    /// Process memory rows: (PID, name, physical RAM bytes, virtual bytes).
    pub processes: Vec<(u32, String, u64, u64)>,
    /// Most recent memory spike (RAM >= 85%), kept as a "black box" event.
    pub latest_spike: Option<MemorySpikeEvent>,
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
}
