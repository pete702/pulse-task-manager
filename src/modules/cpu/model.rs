//! Pure mathematical model for CPU telemetry — zero dependencies on egui/sysinfo.

use std::sync::Arc;

/// CPU hog threshold percentage. Processes using >= this % of total system capacity are flagged as hogs.
pub const CPU_HOG_THRESHOLD_PCT: f32 = 20.0;

/// One raw process row with stable identity metadata and CPU-specific fields.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RawProcessCpuRow {
    pub pid: u32,
    pub name: Arc<str>,
    pub cpu_usage: f32, // Normalized 0.0..=100.0 of total system capacity
    pub ram_bytes: u64,
    pub disk_read_mb_s: f32,
    pub disk_write_mb_s: f32,
    pub eco_qos: Option<bool>,
    pub exe_path: Option<Arc<str>>,
    pub drive_letter: Option<u8>,
    pub creation_time: Option<u64>,
    pub start_time: u64, // sysinfo start_time (Unix seconds)
}

/// A single process CPU entry for display and analysis.
/// Pre-formatted fields avoid format! calls during frame rendering.
#[derive(Clone, Debug, Default, PartialEq)]
#[allow(dead_code)]
pub struct ProcessCpuItem {
    pub pid: u32,
    pub start_time: u64,
    pub creation_time: Option<u64>,
    pub drive_letter: Option<u8>,
    pub name: Arc<str>,
    pub exe_path: Arc<str>,
    pub cpu_pct: f32, // Normalized 0.0..=100.0 of total system capacity
    pub ram_bytes: u64,
    pub disk_read_mb_s: f32,
    pub disk_write_mb_s: f32,
    pub eco_qos: Option<bool>,
    /// Projected working-set change over a 10-second window.
    pub ram_delta_10s: i64,
    pub clone_count: usize,
    pub is_cpu_hog: bool,

    // Pre-formatted display strings (Arc<str> for zero-copy sharing)
    pub pid_text: Arc<str>,
    pub cpu_text: Arc<str>,
    pub ram_text: Arc<str>,
    pub delta_text: Arc<str>,
    pub clone_text: Arc<str>,
    pub disk_io_text: Arc<str>,

    // Internal tracking for conditional formatting (avoids re-allocation when values unchanged)
    last_fmt_cpu_pct: f32,
    last_fmt_ram_bytes: u64,
    last_fmt_delta: i64,
    last_fmt_process_count: usize,
    last_fmt_io_total: f32,
}

impl ProcessCpuItem {
    #[allow(dead_code)]
    pub fn new(
        pid: u32,
        name: Arc<str>,
        cpu_pct: f32,
        ram_bytes: u64,
        creation_time: Option<u64>,
    ) -> Self {
        let cpu_text = Arc::from(format!("{cpu_pct:.1}%"));
        let ram_text = Arc::from(crate::modules::memory::formatters::format_bytes(ram_bytes));
        let pid_text = Arc::from(pid.to_string());
        let delta_text = Arc::from("Stable");
        let clone_text = Arc::from("1x");
        let disk_io_text = Arc::from("0.0 MB/s");
        Self {
            pid,
            start_time: 0,
            creation_time,
            drive_letter: None,
            name: Arc::clone(&name),
            exe_path: Arc::from(""),
            cpu_pct,
            ram_bytes,
            disk_read_mb_s: 0.0,
            disk_write_mb_s: 0.0,
            eco_qos: None,
            ram_delta_10s: 0,
            clone_count: 1,
            is_cpu_hog: cpu_pct >= CPU_HOG_THRESHOLD_PCT,
            pid_text,
            cpu_text,
            ram_text,
            delta_text,
            clone_text,
            disk_io_text,
            // Initialize tracking fields to force first formatting
            last_fmt_cpu_pct: f32::MAX,
            last_fmt_ram_bytes: u64::MAX,
            last_fmt_delta: i64::MAX,
            last_fmt_process_count: usize::MAX,
            last_fmt_io_total: f32::MAX,
        }
    }

    /// Updates pre-formatted fields only when underlying numeric values change.
    /// Avoids allocating new `Arc<str>` on every scan for stable processes.
    pub fn update_formatted(&mut self) {
        let new_is_hog = self.cpu_pct >= CPU_HOG_THRESHOLD_PCT;

        // Compute change flags BEFORE updating tracking fields
        let cpu_changed = (self.cpu_pct - self.last_fmt_cpu_pct).abs() > f32::EPSILON;
        let ram_changed = self.ram_bytes != self.last_fmt_ram_bytes;
        let delta_changed = self.ram_delta_10s != self.last_fmt_delta;
        let count_changed = self.clone_count != self.last_fmt_process_count;

        // pid_text never changes (PID is stable), skip reformatting
        // cpu_text: only if cpu_pct changed
        if cpu_changed {
            self.cpu_text = Arc::from(format!("{:.1}%", self.cpu_pct));
            self.last_fmt_cpu_pct = self.cpu_pct;
        }
        // ram_text: only if ram_bytes changed
        if ram_changed {
            self.ram_text =
                Arc::from(crate::modules::memory::formatters::format_bytes(self.ram_bytes));
            self.last_fmt_ram_bytes = self.ram_bytes;
        }
        // delta_text: only if projected 10-second delta changed
        if delta_changed {
            const SPIKE_BYTES: u64 = 512 * 1024;
            let abs = self.ram_delta_10s.unsigned_abs();
            self.delta_text = if abs < SPIKE_BYTES {
                Arc::from("Stable")
            } else if self.ram_delta_10s > 0 {
                Arc::from(format!("+{}", crate::modules::memory::formatters::format_bytes(abs)))
            } else {
                Arc::from(format!("-{}", crate::modules::memory::formatters::format_bytes(abs)))
            };
            self.last_fmt_delta = self.ram_delta_10s;
        }
        // clone_text: only if process_count changed
        if count_changed {
            self.clone_text = if self.clone_count > 1 {
                Arc::from(format!("[{}]x", self.clone_count))
            } else {
                Arc::from("1x")
            };
            self.last_fmt_process_count = self.clone_count;
        }
        // disk_io_text: only if total I/O rate changed
        let io_total = self.disk_read_mb_s + self.disk_write_mb_s;
        if (io_total - self.last_fmt_io_total).abs() > f32::EPSILON {
            self.disk_io_text = Arc::from(format!("{:.1} MB/s", io_total));
            self.last_fmt_io_total = io_total;
        }

        self.is_cpu_hog = new_is_hog;
    }
}

/// Builds a display item from one raw process CPU row.
#[allow(dead_code)]
pub fn item_from_raw_cpu(row: &RawProcessCpuRow) -> ProcessCpuItem {
    let mut item = ProcessCpuItem::new(
        row.pid,
        Arc::clone(&row.name),
        row.cpu_usage,
        row.ram_bytes,
        row.creation_time,
    );
    item.start_time = row.start_time;
    item.drive_letter = row.drive_letter;
    item.disk_read_mb_s = row.disk_read_mb_s;
    item.disk_write_mb_s = row.disk_write_mb_s;
    item.eco_qos = row.eco_qos;
    item.exe_path = row.exe_path.clone().unwrap_or_default();
    item.update_formatted();
    item
}

/// A single snapshot of system CPU state.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CpuSnapshot {
    pub total_pct: f32,                         // 0.0..=100.0
    pub per_core_pct: Arc<[f32]>,               // Live usage 0.0..=100.0 for each logical processor
    pub hottest_core_idx: usize,                // Index of hottest core
    pub hottest_core_pct: f32,                  // Highest utilization among all logical threads
    pub frequency_mhz: u64, // Live current clock speed in MHz (Turbo/SpeedShift)
    pub base_mhz: u64,      // Nominal base clock speed in MHz
    pub logical_cores: usize, // Number of logical processors
    pub brand: Arc<str>,    // CPU model name cached once
    pub raw_processes: Arc<[RawProcessCpuRow]>, // Raw rows for table grouping/sorting
    pub process_count: usize, // Total processes in the latest scan (stable between scans)
}

impl CpuSnapshot {
    #[inline]
    #[allow(dead_code)]
    pub fn empty() -> Self {
        Self::default()
    }

    /// Returns the raw process rows for table grouping/sorting.
    #[inline]
    pub fn processes_raw(&self) -> &[RawProcessCpuRow] {
        &self.raw_processes
    }

    /// True when one logical thread or total compute load is saturated.
    #[inline]
    pub fn single_thread_bottleneck(&self) -> bool {
        self.hottest_core_pct >= 85.0 || self.total_pct >= 85.0
    }
}

#[cfg(test)]
#[path = "model_tests.rs"]
mod tests;
