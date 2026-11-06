//! Memory spike event for "black box" logging.

use std::sync::Arc;

use crate::util::format_timestamp_now;

/// Memory spike event for "black box" logging.
#[derive(Clone, Debug, Default, PartialEq)]
#[allow(dead_code)]
pub struct MemorySpikeEvent {
    pub timestamp: Arc<str>,
    pub peak_ram_pct: f32,
    pub culprit_name: Arc<str>,
    pub culprit_pid: u32,
    pub culprit_bytes: u64,
}

/// Detects memory spike events (RAM usage >= 85%).
#[allow(dead_code)]
pub fn detect_memory_spike(
    processes: &[crate::modules::memory::model::RawProcessRow],
    total_ram: u64,
    current_ram_pct: f32,
) -> Option<MemorySpikeEvent> {
    if current_ram_pct < 85.0 || total_ram == 0 {
        return None;
    }
    let culprit = processes
        .iter()
        .filter(|r| r.delta_bytes_per_sec > 0)
        .max_by_key(|r| r.delta_bytes_per_sec)
        .or_else(|| processes.iter().max_by_key(|r| r.ram_bytes));
    culprit.map(|row| MemorySpikeEvent {
        timestamp: Arc::from(format_timestamp_now()),
        peak_ram_pct: current_ram_pct,
        culprit_name: Arc::clone(&row.name),
        culprit_pid: row.pid,
        culprit_bytes: row.ram_bytes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::memory::model::RawProcessRow;

    fn row(pid: u32, name: &str, ram_bytes: u64, delta: i64) -> RawProcessRow {
        RawProcessRow {
            pid,
            name: Arc::from(name),
            ram_bytes,
            virtual_bytes: 0,
            delta_bytes_per_sec: delta,
            exe_path: None,
            creation_time: None,
        }
    }

    #[test]
    fn growing_small_process_wins_over_static_large() {
        let rows = [
            row(100, "large_static.exe", 2 * 1024 * 1024 * 1024, 0),
            row(200, "small_growing.exe", 500 * 1024 * 1024, 100 * 1024 * 1024),
        ];
        let event = detect_memory_spike(&rows, 16 * 1024 * 1024 * 1024, 90.0);
        let event = event.unwrap_or_default();
        assert_eq!(event.culprit_pid, 200);
        assert_eq!(event.culprit_name.as_ref(), "small_growing.exe");
    }

    #[test]
    fn falls_back_to_largest_when_no_growth() {
        let rows = [
            row(100, "large_static.exe", 2 * 1024 * 1024 * 1024, 0),
            row(200, "small_shrinking.exe", 500 * 1024 * 1024, -1024),
        ];
        let event = detect_memory_spike(&rows, 16 * 1024 * 1024 * 1024, 90.0);
        let event = event.unwrap_or_default();
        assert_eq!(event.culprit_pid, 100);
    }

    #[test]
    fn below_threshold_returns_none() {
        let rows = [row(100, "large_static.exe", 2 * 1024 * 1024 * 1024, 0)];
        let event = detect_memory_spike(&rows, 16 * 1024 * 1024 * 1024, 84.9);
        assert!(event.is_none());
    }

    #[test]
    fn empty_slice_returns_none() {
        let rows: Vec<RawProcessRow> = Vec::new();
        let event = detect_memory_spike(&rows, 16 * 1024 * 1024 * 1024, 90.0);
        assert!(event.is_none());
    }
}
