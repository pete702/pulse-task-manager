//! Process memory item with pre-formatted display fields for zero-allocation rendering.

use std::sync::Arc;

use crate::modules::memory::formatters::{format_bytes, format_delta};

/// A single process memory entry for display and analysis.
/// Pre-formatted fields (pid_text, ram_text, etc.) avoid format! calls during frame rendering.
#[derive(Clone, Debug, Default, PartialEq)]
#[allow(dead_code)]
pub struct ProcessMemoryItem {
    pub pid: u32,
    pub name: Arc<str>,
    pub ram_bytes: u64,
    pub virtual_bytes: u64,
    pub ram_pct: f32,
    pub process_count: usize,
    pub is_hog: bool,
    /// Per-second working set delta (bytes/sec). Positive = growing (leak), negative = shrinking (reclaim).
    pub delta_bytes_per_sec: i64,
    pub exe_path: Option<Arc<str>>,
    /// Stable process creation time used for PID-reuse-safe actions.
    pub creation_time: Option<u64>,

    // Pre-formatted display strings (Arc<str> for zero-copy sharing)
    pub pid_text: Arc<str>,
    pub ram_text: Arc<str>,
    pub virt_text: Arc<str>,
    pub pct_text: Arc<str>,
    pub delta_text: Arc<str>,
    pub clone_text: Arc<str>,
}

impl ProcessMemoryItem {
    #[allow(dead_code)]
    pub fn new(
        pid: u32,
        name: Arc<str>,
        ram_bytes: u64,
        virtual_bytes: u64,
        total_ram: u64,
    ) -> Self {
        let ram_pct =
            if total_ram > 0 { (ram_bytes as f64 / total_ram as f64 * 100.0) as f32 } else { 0.0 };
        Self {
            pid,
            name: Arc::clone(&name),
            ram_bytes,
            virtual_bytes,
            ram_pct,
            process_count: 1,
            is_hog: ram_pct >= 15.0,
            delta_bytes_per_sec: 0,
            exe_path: None,
            creation_time: None,
            pid_text: Arc::from(pid.to_string()),
            ram_text: Arc::from(format_bytes(ram_bytes)),
            virt_text: Arc::from(format_bytes(virtual_bytes)),
            pct_text: Arc::from(format!("{ram_pct:.1}%")),
            delta_text: Arc::from("0 B/s"),
            clone_text: Arc::from(""),
        }
    }

    /// Updates pre-formatted fields after values change.
    pub fn update_formatted(&mut self, total_ram: u64) {
        self.ram_pct = if total_ram > 0 {
            (self.ram_bytes as f64 / total_ram as f64 * 100.0) as f32
        } else {
            0.0
        };
        self.is_hog = self.ram_pct >= 15.0;
        self.pid_text = Arc::from(self.pid.to_string());
        self.ram_text = Arc::from(format_bytes(self.ram_bytes));
        self.virt_text = Arc::from(format_bytes(self.virtual_bytes));
        self.pct_text = Arc::from(format!("{:.1}%", self.ram_pct));
        self.delta_text = Arc::from(format_delta(self.delta_bytes_per_sec));
        self.clone_text = if self.process_count > 1 {
            Arc::from(format!("[{}]x", self.process_count))
        } else {
            Arc::from("")
        };
    }
}

/// Builds a display item from one raw process row, keeping its executable path.
pub fn item_from_raw(
    row: &crate::modules::memory::model::RawProcessRow,
    total_ram: u64,
) -> ProcessMemoryItem {
    let mut item = ProcessMemoryItem::new(
        row.pid,
        Arc::clone(&row.name),
        row.ram_bytes,
        row.virtual_bytes,
        total_ram,
    );
    item.delta_bytes_per_sec = row.delta_bytes_per_sec;
    item.exe_path = row.exe_path.clone();
    item.creation_time = row.creation_time;
    item.update_formatted(total_ram);
    item
}
