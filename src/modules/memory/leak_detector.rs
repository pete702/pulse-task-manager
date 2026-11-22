//! Lightweight per-process memory slope tracker for slow-leak detection.
use std::collections::{HashMap, HashSet};
use std::sync::{Mutex, OnceLock};
use std::time::Instant;

pub const MAX_PROCESSES: usize = 128;
pub const MAX_SAMPLES: usize = 20;
pub const SAMPLE_INTERVAL_SECS: u64 = 30;
pub const MIN_SAMPLES: usize = 10;
pub const MIN_SPAN_SECS: u64 = 300;
pub const MIN_GROWTH_BYTES: u64 = 32 * 1024 * 1024;
const MIN_SLOPE_MB_PER_MIN: f64 = 1.0;
const BYTES_PER_MB: f64 = 1024.0 * 1024.0;

type Checkpoint = (u64, u64);
/// Identity key: a reused PID must not inherit the previous process's history.
type ProcKey = (u32, u64);

static HISTORY: OnceLock<Mutex<HashMap<ProcKey, Vec<Checkpoint>>>> = OnceLock::new();
static START: OnceLock<Instant> = OnceLock::new();

fn history() -> &'static Mutex<HashMap<ProcKey, Vec<Checkpoint>>> {
    HISTORY.get_or_init(|| Mutex::new(HashMap::new()))
}

fn elapsed_secs() -> u64 {
    START.get_or_init(Instant::now).elapsed().as_secs()
}

/// Pure leak test: 5+ minutes, mostly non-decreasing, near peak, >= 1 MB/min.
pub fn evaluate_leak_slope(samples: &[Checkpoint]) -> Option<u32> {
    if samples.len() < MIN_SAMPLES {
        return None;
    }
    let first = samples.first()?;
    let last = samples.last()?;
    let span = last.0.saturating_sub(first.0);
    if span < MIN_SPAN_SECS {
        return None;
    }
    let max_bytes = samples.iter().map(|s| s.1).max()?;
    let intervals = samples.len() - 1;
    let non_decreasing = samples.windows(2).filter(|w| w[1].1 >= w[0].1).count();
    if non_decreasing * 5 < intervals * 4 {
        return None;
    }
    let growth = last.1.saturating_sub(first.1);
    if growth < MIN_GROWTH_BYTES || last.1 < max_bytes * 98 / 100 {
        return None;
    }
    // A single early burst followed by a flat plateau is not a slow leak.
    let max_step = samples.windows(2).map(|w| w[1].1.saturating_sub(w[0].1)).max().unwrap_or(0);
    if (max_step as f64) * 100.0 >= (growth as f64) * 60.0 {
        return None;
    }
    let mid = samples.len() / 2;
    if samples[mid].1 <= first.1 || last.1 <= samples[mid].1 {
        return None;
    }
    let slope = growth as f64 / BYTES_PER_MB / span as f64 * 60.0;
    if slope >= MIN_SLOPE_MB_PER_MIN {
        Some(slope.round() as u32)
    } else {
        None
    }
}

pub fn format_leak_badge(mb_per_min: u32) -> String {
    format!("[LEAK +{mb_per_min} MB/m]")
}

/// Records one sampling tick; only top memory processes are kept and stale identities pruned.
/// Each tuple is `(pid, start_time, ram_bytes)`.
pub fn record_memory_snapshot(processes: &[(u32, u64, u64)]) {
    record_at(processes, elapsed_secs());
}

fn record_at(processes: &[(u32, u64, u64)], now: u64) {
    let mut top: Vec<(u32, u64, u64)> = processes.to_vec();
    top.sort_unstable_by(|a, b| b.2.cmp(&a.2).then(a.0.cmp(&b.0)));
    top.truncate(MAX_PROCESSES);
    let keep: HashSet<ProcKey> = top.iter().map(|(pid, start, _)| (*pid, *start)).collect();

    let mut map = history().lock().unwrap_or_else(|e| e.into_inner());
    map.retain(|key, _| keep.contains(key));
    for (pid, start, bytes) in top {
        let samples = map.entry((pid, start)).or_default();
        if let Some(last) = samples.last() {
            if last.0 > now {
                samples.clear();
            } else if now - last.0 < SAMPLE_INTERVAL_SECS {
                continue;
            }
        }
        samples.push((now, bytes));
        if samples.len() > MAX_SAMPLES {
            samples.remove(0);
        }
    }
}

fn rate_for(samples: &[Checkpoint]) -> Option<u32> {
    evaluate_leak_slope(samples)
}

/// Current leak rate for one verified `(pid, start_time)` identity.
pub fn get_process_leak_mb_per_min(pid: u32, start_time: u64) -> Option<u32> {
    let map = history().lock().unwrap_or_else(|e| e.into_inner());
    map.get(&(pid, start_time)).and_then(|samples| rate_for(samples))
}

/// Strongest current leak as `(pid, MB/min)`, for the memory cockpit header.
pub fn get_top_leak() -> Option<(u32, u32)> {
    let map = history().lock().unwrap_or_else(|e| e.into_inner());
    map.iter()
        .filter_map(|((pid, _start), samples)| rate_for(samples).map(|rate| (*pid, rate)))
        .max_by_key(|(pid, rate)| (*rate, std::cmp::Reverse(*pid)))
}

#[cfg(test)]
#[path = "leak_detector_tests.rs"]
mod tests;
