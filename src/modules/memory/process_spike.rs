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
    let culprit = processes.iter().max_by_key(|row| row.ram_bytes);
    culprit.map(|row| MemorySpikeEvent {
        timestamp: Arc::from(format_timestamp_now()),
        peak_ram_pct: current_ram_pct,
        culprit_name: Arc::clone(&row.name),
        culprit_pid: row.pid,
        culprit_bytes: row.ram_bytes,
    })
}
