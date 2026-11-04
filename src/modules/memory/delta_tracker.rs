//! Process memory delta tracker for PULSE.
//! Tracks per-process working set changes across collection ticks.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use crate::modules::memory::model::{ProcessSample, RawProcessRow};
use crate::modules::memory::process_spike::MemorySpikeEvent;
use crate::modules::memory::safe_handle::query_creation_time_by_pid;

/// Result of a process memory collection pass.
pub type ProcessMemoryScan = (Vec<RawProcessRow>, Option<MemorySpikeEvent>);

/// Minimum elapsed seconds used when normalising deltas (divide-by-zero guard).
const MIN_DT_SECS: f64 = 0.1;

/// Per-process delta tracking state.
struct DeltaState {
    prev_ram: u64,
    name: Arc<str>,
    exe_path: Option<Arc<str>>,
    creation_time: Option<u64>,
    last_gen: u32,
}

/// Internal state for tracking process memory deltas across ticks.
/// Single HashMap keyed by (pid, identity) where identity = creation_time or start_time.
pub struct ProcessDeltaTracker {
    states: HashMap<(u32, u64), DeltaState>,
    current_gen: u32,
    last_sample: Option<Instant>,
}

impl ProcessDeltaTracker {
    pub fn new() -> Self {
        Self { states: HashMap::new(), current_gen: 0, last_sample: None }
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
        self.current_gen = self.current_gen.wrapping_add(1);
        let mut result = Vec::with_capacity(processes.len());

        // First pass: resolve creation_time for each process
        let mut creation_times = Vec::with_capacity(processes.len());
        for (pid, _name, _ram, _virt, _exe_path, start_time) in processes {
            let start_key = (*pid, *start_time);
            let ct = self
                .states
                .entry(start_key)
                .or_insert_with(|| DeltaState {
                    prev_ram: 0,
                    name: Arc::from(""),
                    exe_path: None,
                    creation_time: query_creation_time_by_pid(*pid),
                    last_gen: 0,
                })
                .creation_time
                .unwrap_or(*start_time);
            creation_times.push(ct);
        }

        // Second pass: track deltas using identity key
        for (i, (pid, name, ram, virt, exe_path, _start_time)) in processes.iter().enumerate() {
            let identity = creation_times[i];
            let key = (*pid, identity);

            // If the entry exists but has empty name, it was created in the first pass
            // for creation_time caching only - treat as new process.
            let is_new = self.states.get(&key).is_none_or(|s| s.name.is_empty());
            let state = self.states.entry(key).or_insert_with(|| DeltaState {
                prev_ram: *ram,
                name: Arc::clone(name),
                exe_path: exe_path.clone(),
                creation_time: Some(identity),
                last_gen: 0,
            });

            let prev = if is_new { *ram } else { state.prev_ram };
            let delta_bytes = *ram as i64 - prev as i64;
            let delta_per_sec = (delta_bytes as f64 / dt).round() as i64;

            state.prev_ram = *ram;
            state.last_gen = self.current_gen;

            // Update name and exe_path if they changed
            if !Arc::ptr_eq(&state.name, name) {
                state.name = Arc::clone(name);
            }
            if state.exe_path.as_ref() != exe_path.as_ref() {
                state.exe_path = exe_path.clone();
            }

            result.push(RawProcessRow {
                pid: *pid,
                name: Arc::clone(name),
                ram_bytes: *ram,
                virtual_bytes: *virt,
                delta_bytes_per_sec: delta_per_sec,
                exe_path: exe_path.clone(),
                creation_time: Some(identity),
            });
        }

        // Retain only entries seen in this generation
        self.states.retain(|_, state| state.last_gen == self.current_gen);
        self.last_sample = Some(now);
        result
    }
}

impl Default for ProcessDeltaTracker {
    fn default() -> Self {
        Self::new()
    }
}
