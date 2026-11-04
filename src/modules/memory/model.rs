//! Pure mathematical model for memory telemetry — zero dependencies on egui/sysinfo.

use std::sync::Arc;

use crate::modules::memory::process_model::MemorySpikeEvent;

/// One raw process row with stable identity metadata.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RawProcessRow {
    pub pid: u32,
    pub name: String,
    pub ram_bytes: u64,
    pub virtual_bytes: u64,
    pub delta_bytes_per_sec: i64,
    pub exe_path: Option<String>,
    pub creation_time: Option<u64>,
}

/// Incoming process sample: (PID, name, RAM, virtual, exe path, sysinfo start time).
pub type ProcessSample = (u32, String, u64, u64, Option<String>, u64);

/// Number of history points to retain (900 points = 15 minutes at 1s interval).
#[allow(dead_code)]
pub const HISTORY_CAPACITY: usize = 900;

/// Time window for chart display.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[allow(clippy::enum_variant_names)]
pub enum ChartWindow {
    /// 1 minute (60 points at 1s interval).
    #[default]
    OneMin = 60,
    /// 5 minutes (300 points at 1s interval).
    FiveMin = 300,
    /// 15 minutes (900 points at 1s interval).
    FifteenMin = 900,
}

impl ChartWindow {
    #[inline]
    pub fn seconds(self) -> u32 {
        self as u32
    }

    pub fn label_en(self) -> &'static str {
        match self {
            ChartWindow::OneMin => "1m",
            ChartWindow::FiveMin => "5m",
            ChartWindow::FifteenMin => "15m",
        }
    }
}

/// A single snapshot of system memory state. All byte values use `u64`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MemorySnapshot {
    pub ram_total_bytes: u64,
    pub ram_used_bytes: u64,
    pub ram_available_bytes: u64,
    pub ram_free_bytes: u64,
    pub swap_total_bytes: u64,
    pub swap_used_bytes: u64,
    pub swap_free_bytes: u64,

    // Windows kernel metrics (K32GetPerformanceInfo).
    pub commit_total_bytes: u64,
    pub commit_limit_bytes: u64,
    pub kernel_paged_bytes: u64,
    pub kernel_nonpaged_bytes: u64,
    pub system_handles: u32,
    pub system_threads: u32,
    pub system_processes: u32,
    pub ram_cached_bytes_override: u64,
    pub kernel_metrics_valid: bool,

    // Telemetry identity and timing.
    pub snapshot_id: u64,
    pub process_scan_id: u64,
    pub sample_elapsed_secs: f64,
    pub scan_duration_ms: f32,
    pub channel_overflows: u64,

    // Shared raw process rows (zero-copy on fast RAM-only ticks).
    pub processes: Arc<Vec<RawProcessRow>>,
    pub latest_spike: Option<MemorySpikeEvent>,
    pub has_heavy_process: bool,
}

impl MemorySnapshot {
    /// Returns real System Cache, falling back to `available - free`.
    #[inline]
    pub fn ram_cached_bytes(&self) -> u64 {
        if self.ram_cached_bytes_override > 0 {
            self.ram_cached_bytes_override
        } else {
            self.ram_available_bytes.saturating_sub(self.ram_free_bytes)
        }
    }

    /// Returns RAM usage as percentage, clamped to `[0.0, 100.0]`.
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
    #[allow(dead_code)]
    #[inline]
    pub fn commit_usage_percent(&self) -> f32 {
        if self.commit_limit_bytes == 0 {
            return 0.0;
        }
        let pct = (self.commit_total_bytes as f64 / self.commit_limit_bytes as f64) * 100.0;
        pct.clamp(0.0, 100.0) as f32
    }

    /// Alias for the Three Walls badge: Commit Charge percentage.
    #[allow(dead_code)]
    #[inline]
    pub fn commit_charge_pct(&self) -> f32 {
        self.commit_usage_percent()
    }
}
