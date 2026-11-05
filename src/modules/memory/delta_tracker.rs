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
    /// Cached Win32 FILETIME creation_time (100ns since 1601).
    /// Populated once on first sighting of the process.
    creation_time: Option<u64>,
    last_gen: u32,
}

/// Internal state for tracking process memory deltas across ticks.
/// Single HashMap keyed by (pid, start_time) where start_time = sysinfo Process::start_time().
/// On first sighting (Vacant entry), we call query_creation_time_by_pid() ONCE and cache it.
/// Subsequent sightings (Occupied entry) reuse the cached creation_time — zero syscalls steady-state.
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
    /// Single pass: resolve creation_time and track deltas together.
    pub fn track_at(&mut self, processes: &[ProcessSample], now: Instant) -> Vec<RawProcessRow> {
        let dt = self
            .last_sample
            .map(|prev| now.saturating_duration_since(prev).as_secs_f64())
            .unwrap_or(0.0)
            .max(MIN_DT_SECS);
        self.current_gen = self.current_gen.wrapping_add(1);
        let mut result = Vec::with_capacity(processes.len());

        for (pid, name, ram, virt, exe_path, start_time) in processes {
            let start_key = (*pid, *start_time);

            // Single lookup: get or create the DeltaState, caching creation_time only on Vacant entry.
            let state = self.states.entry(start_key).or_insert_with(|| {
                let ct = query_creation_time_by_pid(*pid);
                DeltaState {
                    prev_ram: *ram,
                    name: Arc::clone(name),
                    exe_path: exe_path.clone(),
                    creation_time: ct,
                    last_gen: 0,
                }
            });

            // Identity for delta tracking: (pid, creation_time) — falls back to start_time if Win32 query failed.
            let identity = state.creation_time.unwrap_or(*start_time);
            let key = (*pid, identity);

            // If we're tracking by a different identity key than the start_key,
            // we need to ensure the delta state exists under the identity key too.
            // This handles the rare case where creation_time differs from start_time.
            if key != start_key {
                // Migrate or create the delta-tracking entry under the identity key.
                // This is a no-op if already present.
                self.states.entry(key).or_insert_with(|| DeltaState {
                    prev_ram: *ram,
                    name: Arc::clone(name),
                    exe_path: exe_path.clone(),
                    creation_time: Some(identity),
                    last_gen: 0,
                });
            }

            // Now use the identity-keyed state for delta calculation.
            let delta_state = self.states.get_mut(&key).expect("identity key must exist");
            let is_new = delta_state.last_gen == 0;
            let prev = if is_new { *ram } else { delta_state.prev_ram };
            let delta_bytes = *ram as i64 - prev as i64;
            let delta_per_sec = (delta_bytes as f64 / dt).round() as i64;

            delta_state.prev_ram = *ram;
            delta_state.last_gen = self.current_gen;

            // Update name and exe_path if they changed
            if !Arc::ptr_eq(&delta_state.name, name) {
                delta_state.name = Arc::clone(name);
            }
            if delta_state.exe_path.as_ref() != exe_path.as_ref() {
                delta_state.exe_path = exe_path.clone();
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
