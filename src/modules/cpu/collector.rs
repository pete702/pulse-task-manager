//! OS CPU collector for PULSE: gathers real CPU metrics via sysinfo
//! and computes per-process CPU percentages in a single pass.
pub use crate::modules::cpu::collector_cpu::{collect_cpu, warmup_cpu};
use crate::modules::cpu::model::RawProcessCpuRow;
use crate::modules::cpu::qos_action::probe_eco_qos_budgeted;
#[allow(unused_imports)]
pub use crate::modules::cpu::qos_action::{mark_pending_eco, set_eco_watch};
use std::collections::{HashMap, HashSet};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};
use sysinfo::{ProcessRefreshKind, System, UpdateKind};
static LAST_SCAN: OnceLock<Mutex<Option<Instant>>> = OnceLock::new();
#[allow(dead_code)]
pub struct CpuCollector {
    pub seen_pids: HashSet<(u32, u64)>,
    eco_cache: HashMap<(u32, u64), (Option<bool>, Instant)>,
    prev_disk: HashMap<(u32, u64), (u64, u64, Option<u64>)>,
    creation_cache: HashMap<(u32, u64), Option<u64>>,
}
impl CpuCollector {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            seen_pids: HashSet::new(),
            eco_cache: HashMap::new(),
            prev_disk: HashMap::new(),
            creation_cache: HashMap::new(),
        }
    }
    #[allow(dead_code)]
    pub fn set_eco_cached(&mut self, pid: u32, start_time: u64, on: bool) {
        if self.eco_cache.len() > 512 {
            self.eco_cache.clear();
        }
        self.eco_cache.insert((pid, start_time), (Some(on), Instant::now()));
    }
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
        sys.refresh_processes_specifics(
            ProcessRefreshKind::new()
                .with_memory()
                .with_cpu()
                .with_disk_usage()
                .with_exe(UpdateKind::OnlyIfNotSet),
        );
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
        let cores = logical_cores.max(1) as f32;
        let mut raw_rows: Vec<RawProcessCpuRow> = Vec::with_capacity(sys.processes().len());
        for (pid, process) in sys.processes().iter() {
            let pid_u32 = pid.as_u32();
            let start_time = process.start_time();
            let key = (pid_u32, start_time);
            live.insert(key);
            let raw_cpu = process.cpu_usage() / cores;
            let cpu_usage = if raw_cpu.is_finite() { raw_cpu.clamp(0.0, 100.0) } else { 0.0 };
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
            let tot_r = du.total_read_bytes;
            let tot_w = du.total_written_bytes;
            let prev = self.prev_disk.get(&key).copied();
            let creation_time = if let Some(ct) = self.creation_cache.get(&key).copied() {
                ct
            } else if let Some((_, _, ct)) = prev {
                self.creation_cache.insert(key, ct);
                ct
            } else {
                let ct = crate::modules::memory::query_creation_time_by_pid(pid_u32);
                if self.creation_cache.len() > 512 {
                    self.creation_cache.clear();
                }
                self.creation_cache.insert(key, ct);
                ct
            };
            let (r_mb, w_mb) = match prev {
                Some((pr, pw, _)) => {
                    (rate(tot_r.saturating_sub(pr)), rate(tot_w.saturating_sub(pw)))
                }
                None => (0.0, 0.0),
            };
            self.prev_disk.insert(key, (tot_r, tot_w, creation_time));
            raw_rows.push(RawProcessCpuRow {
                pid: pid_u32,
                name: crate::util::intern_name(process.name()),
                cpu_usage,
                ram_bytes: ram,
                disk_read_mb_s: r_mb,
                disk_write_mb_s: w_mb,
                eco_qos: None,
                exe_path: None,
                drive_letter: drive,
                creation_time,
                start_time,
            });
        }
        self.fill_eco_qos(&mut raw_rows);
        self.seen_pids = live.clone();
        self.eco_cache.retain(|k, _| live.contains(k));
        self.prev_disk.retain(|k, _| live.contains(k));
        self.creation_cache.retain(|k, _| live.contains(k));
        raw_rows.sort_by(|a, b| b.cpu_usage.total_cmp(&a.cpu_usage));
        raw_rows
    }
    fn fill_eco_qos(&mut self, rows: &mut [RawProcessCpuRow]) {
        let mut budget: u8 = 12;
        let pending: HashMap<(u32, u64), Option<bool>> =
            crate::modules::cpu::qos_action::PENDING_ECO
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
            } else if let Some(v) = probe_eco_qos_budgeted(row.pid, &mut budget) {
                self.eco_cache.insert(key, (v, Instant::now()));
                row.eco_qos = v;
            } else {
                row.eco_qos = None;
            }
        }
        if self.eco_cache.len() > 512 {
            self.eco_cache.clear();
        }
    }
}
impl Default for CpuCollector {
    fn default() -> Self {
        Self::new()
    }
}
#[cfg(test)]
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
#[allow(dead_code)]
#[cfg(test)]
pub(crate) fn first_sample_disk_bytes(
    read: u64,
    written: u64,
    total_read: u64,
    total_written: u64,
) -> (u64, u64) {
    guarded_disk_bytes(read, written, total_read, total_written, true)
}
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
pub(crate) fn pid_contains(pid: u32, needle: &str) -> bool {
    pid.to_string().contains(needle)
}
#[cfg(test)]
#[path = "collector_tests.rs"]
mod tests;
