//! OS CPU collector for PULSE: gathers real CPU metrics via sysinfo
//! and computes per-process CPU percentages in a single pass.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

use sysinfo::{ProcessRefreshKind, System};

pub use crate::modules::cpu::collector_cpu::{collect_cpu, warmup_cpu};
use crate::modules::cpu::model::RawProcessCpuRow;
use crate::modules::cpu::qos_action::is_process_eco_qos;

/// Memo: last process scan time for disk I/O rate divisor.
static LAST_SCAN: OnceLock<Mutex<Option<Instant>>> = OnceLock::new();
/// Pending EcoQoS overrides from UI actions to worker cache.
#[allow(clippy::type_complexity)]
static PENDING_ECO: OnceLock<Mutex<HashMap<(u32, u64), Option<bool>>>> = OnceLock::new();

/// Stateful collector holding `seen_pids` for first-sample disk guard.
#[allow(dead_code)]
pub struct CpuCollector {
    pub seen_pids: HashSet<(u32, u64)>,
    eco_cache: HashMap<(u32, u64), (Option<bool>, Instant)>,
    prev_disk: HashMap<(u32, u64), (u64, u64)>,
}

impl CpuCollector {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self { seen_pids: HashSet::new(), eco_cache: HashMap::new(), prev_disk: HashMap::new() }
    }
    /// Marks a PID as EcoQoS after UI enable action.
    #[allow(dead_code)]
    pub fn set_eco_cached(&mut self, pid: u32, start_time: u64, on: bool) {
        self.eco_cache.insert((pid, start_time), (Some(on), Instant::now()));
    }
    /// Drops a stale EcoQoS entry so the next scan re-queries.
    #[allow(dead_code)]
    pub fn invalidate_eco(&mut self, pid: u32, start_time: u64) {
        self.eco_cache.remove(&(pid, start_time));
    }
    #[allow(dead_code)]
    pub fn is_new(&self, pid: u32, start_time: u64) -> bool {
        !self.seen_pids.contains(&(pid, start_time))
    }
    #[allow(dead_code)]
    pub fn collect_process_cpu(
        &mut self,
        sys: &mut System,
        logical_cores: usize,
        refresh_processes: bool,
    ) -> Vec<RawProcessCpuRow> {
        if !refresh_processes {
            return Vec::new();
        }
        sys.refresh_processes_specifics(ProcessRefreshKind::everything());
        let elapsed_secs = elapsed_since_last_scan();
        let mut live: HashSet<(u32, u64)> = HashSet::with_capacity(sys.processes().len());
        let rate = |b: u64| {
            let v = b as f32 / (elapsed_secs * 1_048_576.0);
            if v.is_finite() {
                v.clamp(0.0, f32::MAX)
            } else {
                0.0
            }
        };
        let mut raw_rows: Vec<RawProcessCpuRow> = Vec::with_capacity(sys.processes().len());
        for (pid, process) in sys.processes().iter() {
            let pid_u32 = pid.as_u32();
            let start_time = process.start_time();
            live.insert((pid_u32, start_time));
            let cpu_usage = (process.cpu_usage() / logical_cores.max(1) as f32).clamp(0.0, 100.0);
            let ram = process.memory();
            let du = process.disk_usage();
            let exe = process.exe();
            let drive = exe.and_then(|p| {
                let eb = p.as_os_str().as_encoded_bytes();
                if eb.len() >= 2 && eb[1] == b':' && eb[0].is_ascii_alphabetic() {
                    Some(eb[0].to_ascii_uppercase())
                } else {
                    None
                }
            });
            let key = (pid_u32, start_time);
            let tot_r = du.total_read_bytes;
            let tot_w = du.total_written_bytes;
            let (r_mb, w_mb) = match self.prev_disk.get(&key) {
                Some((pr, pw)) => {
                    let dr = tot_r.saturating_sub(*pr);
                    let dw = tot_w.saturating_sub(*pw);
                    (rate(dr), rate(dw))
                }
                None => (0.0, 0.0),
            };
            self.prev_disk.insert(key, (tot_r, tot_w));
            raw_rows.push(RawProcessCpuRow {
                pid: pid_u32,
                name: Arc::from(process.name()),
                cpu_usage,
                ram_bytes: ram,
                disk_read_mb_s: r_mb,
                disk_write_mb_s: w_mb,
                eco_qos: None,
                exe_path: None,
                drive_letter: drive,
                creation_time: None,
                start_time,
            });
        }
        self.fill_eco_qos(&mut raw_rows);
        self.seen_pids = live.clone();
        self.eco_cache.retain(|k, _| live.contains(k));
        self.prev_disk.retain(|k, _| live.contains(k));
        raw_rows.sort_by(|a, b| b.cpu_usage.total_cmp(&a.cpu_usage));
        raw_rows
    }

    fn fill_eco_qos(&mut self, rows: &mut [RawProcessCpuRow]) {
        let mut budget = 12;
        let pending: HashMap<(u32, u64), Option<bool>> = PENDING_ECO
            .get()
            .map(|slot| std::mem::take(&mut *slot.lock().unwrap_or_else(|e| e.into_inner())))
            .unwrap_or_default();
        for row in rows.iter_mut() {
            if row.pid == 0 || row.pid == 4 {
                row.eco_qos = Some(false);
                continue;
            }
            let key = (row.pid, row.start_time);
            if let Some(primed) = pending.get(&key).copied() {
                self.eco_cache.insert(key, (primed, Instant::now()));
                row.eco_qos = primed;
                continue;
            }
            let fresh = self
                .eco_cache
                .get(&key)
                .is_some_and(|(_, at)| at.elapsed() < Duration::from_secs(30));
            if fresh {
                row.eco_qos = self.eco_cache.get(&key).map(|(v, _)| *v).unwrap_or(None);
            } else if budget > 0 {
                let v = is_process_eco_qos(row.pid);
                self.eco_cache.insert(key, (v, Instant::now()));
                row.eco_qos = v;
                budget -= 1;
            } else {
                row.eco_qos = None;
            }
        }
    }
}

impl Default for CpuCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Primes worker EcoQoS cache after UI enable so filter updates next tick.
#[allow(dead_code)]
pub fn set_eco_cached(pid: u32, start_time: u64, on: bool) {
    let slot = PENDING_ECO.get_or_init(|| Mutex::new(HashMap::new()));
    slot.lock().unwrap_or_else(|e| e.into_inner()).insert((pid, start_time), Some(on));
}

/// Drops pending EcoQoS prime so next scan re-queries Win32.
#[allow(dead_code)]
pub fn invalidate_eco_cached(pid: u32, start_time: u64) {
    let slot = PENDING_ECO.get_or_init(|| Mutex::new(HashMap::new()));
    slot.lock().unwrap_or_else(|e| e.into_inner()).insert((pid, start_time), None);
}

/// First-sample guard: only zero when `is_first_seen` is true.
/// An already-seen PID doing its first I/O on tick 2 keeps real bytes.
pub(crate) fn guarded_disk_bytes(
    read: u64,
    written: u64,
    total_read: u64,
    total_written: u64,
    is_first_seen: bool,
) -> (u64, u64) {
    if !is_first_seen {
        return (read, written);
    }
    let r = if total_read > 0 && read == total_read { 0 } else { read };
    let w = if total_written > 0 && written == total_written { 0 } else { written };
    (r, w)
}

/// Back-compat wrapper: treat as first-seen (old stateless behavior).
#[allow(dead_code)]
pub(crate) fn first_sample_disk_bytes(
    read: u64,
    written: u64,
    total_read: u64,
    total_written: u64,
) -> (u64, u64) {
    guarded_disk_bytes(read, written, total_read, total_written, true)
}

/// Elapsed seconds since previous process scan (default 1.0, clamped >= 0.2).
fn elapsed_since_last_scan() -> f32 {
    let slot = LAST_SCAN.get_or_init(|| Mutex::new(None));
    let now = std::time::Instant::now();
    let mut guard = slot.lock().unwrap_or_else(|e| e.into_inner());
    let elapsed = match *guard {
        Some(prev) => now.duration_since(prev).as_secs_f32().max(0.2),
        None => 1.0,
    };
    *guard = Some(now);
    elapsed
}

/// Back-compat free function using a static collector instance.
#[allow(dead_code)]
pub fn collect_process_cpu(
    sys: &mut System,
    logical_cores: usize,
    refresh_processes: bool,
) -> Vec<RawProcessCpuRow> {
    static STATIC: OnceLock<Mutex<CpuCollector>> = OnceLock::new();
    let slot = STATIC.get_or_init(|| Mutex::new(CpuCollector::new()));
    let mut guard = slot.lock().unwrap_or_else(|e| e.into_inner());
    guard.collect_process_cpu(sys, logical_cores, refresh_processes)
}

/// Zero-alloc PID substring check using a stack buffer.
pub(crate) fn pid_contains(pid: u32, needle: &str) -> bool {
    let mut buf = [0u8; 10];
    let mut pos = 10;
    let mut n = pid;
    loop {
        pos -= 1;
        buf[pos] = b'0' + (n % 10) as u8;
        n /= 10;
        if n == 0 {
            break;
        }
    }
    std::str::from_utf8(&buf[pos..]).unwrap_or("").contains(needle)
}

#[cfg(test)]
#[path = "collector_tests.rs"]
mod tests;
