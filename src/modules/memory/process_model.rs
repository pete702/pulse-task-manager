//! Pure mathematical model for process memory telemetry — zero dependencies on egui/sysinfo.
//! Provides process memory aggregation, grouping, sorting, and spike detection.

use std::collections::HashMap;

use crate::modules::memory::model::RawProcessRow;
use crate::util::format_timestamp_now;

/// A single process memory entry for display and analysis.
#[derive(Clone, Debug, Default, PartialEq)]
#[allow(dead_code)]
pub struct ProcessMemoryItem {
    pub pid: u32,
    pub name: String,
    pub ram_bytes: u64,
    pub virtual_bytes: u64,
    pub ram_pct: f32,
    pub process_count: usize,
    pub is_hog: bool,
    /// Per-second working set delta (bytes/sec). Positive = growing (leak), negative = shrinking (reclaim).
    pub delta_bytes_per_sec: i64,
    pub exe_path: Option<String>,
    /// Stable process creation time used for PID-reuse-safe actions.
    pub creation_time: Option<u64>,
}

/// Column for sorting process list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[allow(dead_code)]
pub enum MemorySortCol {
    #[default]
    Name,
    Pid,
    RamBytes,
    VirtualBytes,
    RamPct,
}

/// Memory spike event for "black box" logging.
#[derive(Clone, Debug, Default, PartialEq)]
#[allow(dead_code)]
pub struct MemorySpikeEvent {
    pub timestamp: String,
    pub peak_ram_pct: f32,
    pub culprit_name: String,
    pub culprit_pid: u32,
    pub culprit_bytes: u64,
}

impl ProcessMemoryItem {
    #[allow(dead_code)]
    pub fn new(pid: u32, name: String, ram_bytes: u64, virtual_bytes: u64, total_ram: u64) -> Self {
        let ram_pct =
            if total_ram > 0 { (ram_bytes as f64 / total_ram as f64 * 100.0) as f32 } else { 0.0 };
        Self {
            pid,
            name,
            ram_bytes,
            virtual_bytes,
            ram_pct,
            process_count: 1,
            is_hog: ram_pct >= 15.0,
            delta_bytes_per_sec: 0,
            exe_path: None,
            creation_time: None,
        }
    }
}

/// Builds a display item from one raw process row, keeping its executable path.
pub fn item_from_raw(row: &RawProcessRow, total_ram: u64) -> ProcessMemoryItem {
    let mut item = ProcessMemoryItem::new(
        row.pid,
        row.name.clone(),
        row.ram_bytes,
        row.virtual_bytes,
        total_ram,
    );
    item.delta_bytes_per_sec = row.delta_bytes_per_sec;
    item.exe_path.clone_from(&row.exe_path);
    item.creation_time = row.creation_time;
    item
}

/// Filters, groups (by name), and sorts process memory items.
#[allow(dead_code)]
pub fn filter_group_and_sort(
    processes: &[RawProcessRow],
    total_ram: u64,
    search: &str,
    group_clones: bool,
    sort_col: MemorySortCol,
    sort_asc: bool,
) -> Vec<ProcessMemoryItem> {
    let search_lower = search.to_lowercase();
    let mut items: Vec<ProcessMemoryItem> = processes
        .iter()
        .filter(|row| {
            search.is_empty()
                || search_lower == row.pid.to_string()
                || row.name.to_lowercase().contains(&search_lower)
        })
        .map(|row| item_from_raw(row, total_ram))
        .collect();
    if group_clones {
        let mut grouped: HashMap<String, ProcessMemoryItem> = HashMap::new();
        for item in items {
            grouped
                .entry(item.name.clone())
                .and_modify(|ex| {
                    ex.ram_bytes = ex.ram_bytes.saturating_add(item.ram_bytes);
                    ex.virtual_bytes = ex.virtual_bytes.saturating_add(item.virtual_bytes);
                    ex.delta_bytes_per_sec =
                        ex.delta_bytes_per_sec.saturating_add(item.delta_bytes_per_sec);
                    ex.process_count += 1;
                    ex.ram_pct = if total_ram > 0 {
                        (ex.ram_bytes as f64 / total_ram as f64 * 100.0) as f32
                    } else {
                        0.0
                    };
                    ex.is_hog = ex.ram_pct >= 15.0;
                    if ex.exe_path.is_none() {
                        ex.exe_path.clone_from(&item.exe_path);
                    }
                    if item.pid < ex.pid {
                        ex.pid = item.pid;
                    }
                })
                .or_insert(item);
        }
        items = grouped.into_values().collect();
    }
    items.sort_by(|a, b| {
        let ord = match sort_col {
            MemorySortCol::Name => a.name.cmp(&b.name),
            MemorySortCol::Pid => a.pid.cmp(&b.pid),
            MemorySortCol::RamBytes => a.ram_bytes.cmp(&b.ram_bytes),
            MemorySortCol::VirtualBytes => a.virtual_bytes.cmp(&b.virtual_bytes),
            MemorySortCol::RamPct => {
                a.ram_pct.partial_cmp(&b.ram_pct).unwrap_or(std::cmp::Ordering::Equal)
            }
        };
        if sort_asc {
            ord
        } else {
            ord.reverse()
        }
    });
    items
}

/// Detects memory spike events (RAM usage >= 85%).
#[allow(dead_code)]
pub fn detect_memory_spike(
    processes: &[RawProcessRow],
    total_ram: u64,
    current_ram_pct: f32,
) -> Option<MemorySpikeEvent> {
    if current_ram_pct < 85.0 || total_ram == 0 {
        return None;
    }
    let culprit = processes.iter().max_by_key(|row| row.ram_bytes);
    culprit.map(|row| MemorySpikeEvent {
        timestamp: format_timestamp_now(),
        peak_ram_pct: current_ram_pct,
        culprit_name: row.name.clone(),
        culprit_pid: row.pid,
        culprit_bytes: row.ram_bytes,
    })
}
