//! Process memory delta tracker for PULSE.
//! Tracks per-process working set changes across collection ticks.

use std::collections::{HashMap, HashSet};
use std::time::Instant;

use crate::modules::memory::model::{ProcessSample, RawProcessRow};
use crate::modules::memory::process_model::MemorySpikeEvent;
use crate::modules::memory::safe_handle::query_creation_time_by_pid;

/// Result of a process memory collection pass.
pub type ProcessMemoryScan = (Vec<RawProcessRow>, Option<MemorySpikeEvent>);

/// Minimum elapsed seconds used when normalising deltas (divide-by-zero guard).
const MIN_DT_SECS: f64 = 0.1;

/// Internal state for tracking process memory deltas across ticks.
pub struct ProcessDeltaTracker {
    /// Keyed by `(pid, creation_time_or_start_time)` so PID reuse cannot inherit deltas.
    prev_ram_bytes: HashMap<(u32, u64), u64>,
    /// Creation times cached per `(pid, sysinfo_start_time)`, queried at most once.
    creation_cache: HashMap<(u32, u64), Option<u64>>,
    active_keys: HashSet<(u32, u64)>,
    active_starts: HashSet<(u32, u64)>,
    last_sample: Option<Instant>,
}

impl ProcessDeltaTracker {
    pub fn new() -> Self {
        Self {
            prev_ram_bytes: HashMap::new(),
            creation_cache: HashMap::new(),
            active_keys: HashSet::new(),
            active_starts: HashSet::new(),
            last_sample: None,
        }
    }

    /// Calculates deltas using the real elapsed time since the previous sample.
    pub fn track(&mut self, processes: &[ProcessSample]) -> Vec<RawProcessRow> {
        self.track_at(processes, Instant::now())
    }

    /// Testable variant of [`Self::track`] with an explicit sample timestamp.
    pub fn track_at(&mut self, processes: &[ProcessSample], now: Instant) -> Vec<RawProcessRow> {
        let dt = self
            .last_sample
            .map(|prev| now.saturating_duration_since(prev).as_secs_f64())
            .unwrap_or(0.0)
            .max(MIN_DT_SECS);
        let mut result = Vec::with_capacity(processes.len());
        self.active_keys.clear();
        self.active_starts.clear();
        for (pid, name, ram, virt, exe_path, start_time) in processes {
            let start_key = (*pid, *start_time);
            let creation_time = *self
                .creation_cache
                .entry(start_key)
                .or_insert_with(|| query_creation_time_by_pid(*pid));
            let identity = creation_time.unwrap_or(*start_time);
            let key = (*pid, identity);
            let prev = self.prev_ram_bytes.get(&key).copied().unwrap_or(*ram);
            let delta_bytes = *ram as i64 - prev as i64;
            let delta_per_sec = (delta_bytes as f64 / dt).round() as i64;
            self.prev_ram_bytes.insert(key, *ram);
            self.active_keys.insert(key);
            self.active_starts.insert(start_key);
            result.push(RawProcessRow {
                pid: *pid,
                name: name.clone(),
                ram_bytes: *ram,
                virtual_bytes: *virt,
                delta_bytes_per_sec: delta_per_sec,
                exe_path: exe_path.clone(),
                creation_time,
            });
        }
        self.prev_ram_bytes.retain(|key, _| self.active_keys.contains(key));
        self.creation_cache.retain(|key, _| self.active_starts.contains(key));
        self.last_sample = Some(now);
        result
    }
}

impl Default for ProcessDeltaTracker {
    fn default() -> Self {
        Self::new()
    }
}
