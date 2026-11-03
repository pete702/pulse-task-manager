//! Process memory delta tracker for PULSE.
//! Tracks per-process working set changes across collection ticks.

use std::collections::{HashMap, HashSet};
use std::time::Instant;

use crate::modules::memory::model::{ProcessSample, RawProcessRow};
use crate::modules::memory::process_model::MemorySpikeEvent;

/// Result of a process memory collection pass.
pub type ProcessMemoryScan = (Vec<RawProcessRow>, Option<MemorySpikeEvent>);

/// Minimum elapsed seconds used when normalising deltas (divide-by-zero guard).
const MIN_DT_SECS: f64 = 0.1;

/// Internal state for tracking process memory deltas across ticks.
/// Reuses active_pids HashSet capacity across ticks to avoid reallocations.
pub struct ProcessDeltaTracker {
    prev_ram_bytes: HashMap<u32, u64>,
    active_pids: HashSet<u32>,
    last_sample: Option<Instant>,
}

impl ProcessDeltaTracker {
    pub fn new() -> Self {
        Self { prev_ram_bytes: HashMap::new(), active_pids: HashSet::new(), last_sample: None }
    }

    /// Calculates deltas using the real elapsed time since the previous sample.
    pub fn track(&mut self, processes: &[ProcessSample]) -> Vec<RawProcessRow> {
        self.track_at(processes, Instant::now())
    }

    /// Testable variant of [`Self::track`] with an explicit sample timestamp.
    /// Divides the byte delta by the actual elapsed seconds, clamped to 0.1 s.
    pub fn track_at(&mut self, processes: &[ProcessSample], now: Instant) -> Vec<RawProcessRow> {
        let dt = self
            .last_sample
            .map(|prev| now.saturating_duration_since(prev).as_secs_f64())
            .unwrap_or(0.0)
            .max(MIN_DT_SECS);
        let mut result = Vec::with_capacity(processes.len());
        for (pid, name, ram, virt, exe_path) in processes {
            let prev = self.prev_ram_bytes.get(pid).copied().unwrap_or(*ram);
            let delta_bytes = *ram as i64 - prev as i64;
            let delta_per_sec = (delta_bytes as f64 / dt).round() as i64;
            self.prev_ram_bytes.insert(*pid, *ram);
            result.push((*pid, name.clone(), *ram, *virt, delta_per_sec, exe_path.clone()));
        }
        // Reuse active_pids HashSet capacity across ticks
        self.active_pids.clear();
        self.active_pids.extend(processes.iter().map(|(pid, _, _, _, _)| *pid));
        self.prev_ram_bytes.retain(|pid, _| self.active_pids.contains(pid));
        self.last_sample = Some(now);
        result
    }
}

impl Default for ProcessDeltaTracker {
    fn default() -> Self {
        Self::new()
    }
}
