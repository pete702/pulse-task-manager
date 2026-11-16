//! Process memory delta tracker for PULSE.
//! Tracks per-process working set changes across collection ticks.
//! Single HashMap keyed by (pid, start_time). creation_time cached once on Vacant entry.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::modules::memory::model::{ProcessSample, RawProcessRow};

use crate::platform::handle::query_creation_time_by_pid;

/// Minimum elapsed seconds used when normalising deltas (divide-by-zero guard).
const MIN_DT_SECS: f64 = 0.1;
/// Bound for a single-pass memory delta rate to avoid unrealistic spikes.
const MAX_DELTA_PER_SEC: i64 = 1_000_000_000_000;

/// Per-process delta tracking state.
struct DeltaState {
    prev_ram: u64,
    name: Arc<str>,
    exe_path: Option<Arc<str>>,
    /// Cached Win32 FILETIME creation_time (100ns since 1601).
    /// Populated once on first sighting of the process (Vacant entry).
    creation_time: Option<u64>,
    last_gen: u32,
    /// Number of failed creation-time resolve attempts (probe failure cooldown).
    fail_attempts: u8,
    /// Timestamp of the last resolve attempt (cooldown tracking).
    last_attempt: Option<Instant>,
}

/// Internal state for tracking process memory deltas across ticks.
/// Single HashMap keyed by (pid, start_time) where start_time = sysinfo Process::start_time().
/// On first sighting (Vacant entry), we call query_creation_time_by_pid() ONCE and cache it.
/// Subsequent sightings (Occupied entry) reuse the cached creation_time -- zero syscalls steady-state.
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
        self.track_with_resolver(processes, now, query_creation_time_by_pid)
    }

    /// Deterministic variant with an injectable creation-time resolver.
    /// Tests pass `|_| None` so results never depend on live OS processes.
    pub fn track_with_resolver<F>(
        &mut self,
        processes: &[ProcessSample],
        now: Instant,
        resolve: F,
    ) -> Vec<RawProcessRow>
    where
        F: Fn(u32) -> Option<u64>,
    {
        let dt_raw = self
            .last_sample
            .map(|prev| now.saturating_duration_since(prev).as_secs_f64())
            .unwrap_or(0.0);
        // A very short (or first-pass) interval must not produce an unrealistic spike rate.
        let short_interval = dt_raw > 0.0 && dt_raw < 1.0;
        let dt = dt_raw.max(MIN_DT_SECS);
        self.current_gen = self.current_gen.wrapping_add(1);
        let mut result = Vec::with_capacity(processes.len());
        // Syscall budget shared by new PIDs and deferred retries.
        let mut budget: u32 = 64;

        for (pid, name, ram, virt, exe_path, start_time) in processes {
            let key = (*pid, *start_time);
            let seen_pid = *pid;

            // Deferred marker is None; failed queries stay uncached for retry.
            let state = self.states.entry(key).or_insert_with(|| DeltaState {
                prev_ram: *ram,
                name: Arc::clone(name),
                exe_path: exe_path.clone(),
                creation_time: None,
                last_gen: 0,
                fail_attempts: 0,
                last_attempt: None,
            });
            // Probe-failure cooldown: unreadable/protected PIDs must not burn the
            // per-tick Win32 query budget on every single scan.
            let cooldown = match state.fail_attempts {
                0 | 1 => Duration::ZERO,
                2 => Duration::from_secs(2),
                3 => Duration::from_secs(4),
                _ => Duration::from_secs(8),
            };
            let attempt_allowed = state.creation_time.is_none()
                && budget > 0
                && state.last_attempt.is_none_or(|t| t.elapsed() >= cooldown);
            if attempt_allowed {
                budget -= 1;
                state.last_attempt = Some(now);
                if let Some(queried) = resolve(seen_pid) {
                    state.creation_time = Some(queried);
                    state.fail_attempts = 0;
                } else {
                    state.fail_attempts = state.fail_attempts.saturating_add(1);
                }
            }

            // Use the cached creation_time (FILETIME) for PID-reuse safety in actions (Open Location / Terminate).
            // No fallback to Unix start_time -- they are incompatible time bases.
            let identity = state.creation_time;

            // Treat very short / first-pass intervals like the stale-window rule: no delta.
            let is_new = state.last_gen == 0 || dt > 15.0 || short_interval;
            let prev = if is_new { *ram } else { state.prev_ram };
            let delta_bytes = *ram as i64 - prev as i64;
            let delta_per_sec = (delta_bytes as f64 / dt).round() as i64;
            let delta_per_sec = delta_per_sec.clamp(-MAX_DELTA_PER_SEC, MAX_DELTA_PER_SEC);

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
                creation_time: identity,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_key_retains_cached_creation_time() {
        let mut tracker = ProcessDeltaTracker::new();
        let pid = u32::MAX - 1000;
        let start_time = 5678u64;
        let sample1 = vec![(pid, Arc::from("test.exe"), 1000u64, 2000u64, None, start_time)];
        let result1 = tracker.track_with_resolver(&sample1, Instant::now(), |_| None);
        assert_eq!(result1.len(), 1);
        assert_eq!(result1[0].creation_time, None);
        let sample2 = vec![(pid, Arc::from("test.exe"), 1500u64, 2000u64, None, start_time)];
        let result2 = tracker.track_with_resolver(&sample2, Instant::now(), |_| None);
        assert_eq!(result2.len(), 1);
        assert_eq!(tracker.states.len(), 1);
        let state = tracker.states.get(&(pid, start_time)).unwrap();
        assert_eq!(state.last_gen, tracker.current_gen);
        assert_eq!(state.creation_time, None);
    }

    #[test]
    fn test_new_pid_gets_new_entry() {
        let mut tracker = ProcessDeltaTracker::new();
        let sample = vec![
            (u32::MAX - 2, Arc::from("a.exe"), 100u64, 200u64, None, 1000u64),
            (u32::MAX - 1, Arc::from("b.exe"), 200u64, 300u64, None, 2000u64),
        ];
        tracker.track_with_resolver(&sample, Instant::now(), |_| None);
        assert_eq!(tracker.states.len(), 2);
    }
    #[test]
    fn twelve_new_pids_defer_then_resolve() {
        let mut tracker = ProcessDeltaTracker::new();
        let sample: Vec<_> = (0..12u32)
            .map(|i| {
                (
                    u32::MAX - 20 + i,
                    Arc::from(format!("p{i}.exe")),
                    100u64,
                    200u64,
                    None,
                    5000u64 + i as u64,
                )
            })
            .collect();
        let r1 = tracker.track_with_resolver(&sample, Instant::now(), |_| None);
        assert_eq!(r1.len(), 12);
        assert_eq!(r1.iter().filter(|r| r.creation_time.is_none()).count(), 12);
        let r2 = tracker
            .track_with_resolver(&sample, Instant::now(), |pid| Some(9000 + pid as u64 % 100));
        assert_eq!(r2.iter().filter(|r| r.creation_time.is_none()).count(), 0);
        assert_eq!(r2.iter().filter(|r| r.creation_time.is_some()).count(), 12);
    }
}
