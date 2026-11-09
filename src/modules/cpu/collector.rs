//! OS CPU collector for PULSE: gathers real CPU metrics via sysinfo
//! and computes per-process CPU percentages in a single pass.
pub use crate::modules::cpu::collector_cpu::{collect_cpu, warmup_cpu};
use crate::modules::cpu::collector_eco;
use crate::modules::cpu::model::RawProcessCpuRow;
#[allow(unused_imports)]
pub use crate::modules::cpu::qos_action::mark_pending_eco;
use std::collections::{HashMap, HashSet};
use std::time::Instant;
use sysinfo::{ProcessRefreshKind, System, UpdateKind};
#[allow(dead_code)]
pub struct CpuCollector {
    pub seen_pids: HashSet<(u32, u64)>,
    eco_cache: HashMap<(u32, u64), (bool, Instant)>,
    eco_cursor: usize,
    prev_disk: HashMap<(u32, u64), (u64, u64, Option<u64>)>,
    creation_cache: HashMap<(u32, u64), Option<u64>>,
    last_scan: Option<Instant>,
}
impl CpuCollector {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            seen_pids: HashSet::new(),
            eco_cache: HashMap::new(),
            eco_cursor: 0,
            prev_disk: HashMap::new(),
            creation_cache: HashMap::new(),
            last_scan: None,
        }
    }
    #[allow(dead_code)]
    pub fn set_eco_cached(&mut self, pid: u32, start_time: u64, on: bool) {
        if self.eco_cache.len() > 512 {
            self.eco_cache.clear();
        }
        self.eco_cache.insert((pid, start_time), (on, Instant::now()));
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
    ) -> Vec<RawProcessCpuRow> {
        sys.refresh_processes_specifics(
            ProcessRefreshKind::new()
                .with_memory()
                .with_cpu()
                .with_disk_usage()
                .with_exe(UpdateKind::OnlyIfNotSet),
        );
        let elapsed_secs = self.elapsed_since_last_scan();
        let mut live: HashSet<(u32, u64)> = HashSet::with_capacity(sys.processes().len());
        let cores = logical_cores.max(1) as f32;
        let mut creation_probes: usize = 0;
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
            let creation_time =
                if let Some(cached) = self.creation_cache.get(&key).copied().flatten() {
                    Some(cached)
                } else if pid_u32 != 0 && pid_u32 != 4 && creation_probes < 64 {
                    creation_probes += 1;
                    if let Some(ct) = crate::modules::memory::query_creation_time_by_pid(pid_u32) {
                        if self.creation_cache.len() > 512 {
                            self.creation_cache.clear();
                        }
                        self.creation_cache.insert(key, Some(ct));
                        Some(ct)
                    } else {
                        None
                    }
                } else {
                    None
                };
            let (r_mb, w_mb) = match prev {
                Some((pr, pw, _)) => (
                    Self::disk_rate(tot_r, pr, elapsed_secs as f64) as f32 / 1_048_576.0,
                    Self::disk_rate(tot_w, pw, elapsed_secs as f64) as f32 / 1_048_576.0,
                ),
                None => (0.0, 0.0),
            };
            let exe_path =
                exe.and_then(|p| p.to_str()).filter(|s| !s.is_empty()).map(std::sync::Arc::from);
            self.prev_disk.insert(key, (tot_r, tot_w, creation_time));
            raw_rows.push(RawProcessCpuRow {
                pid: pid_u32,
                name: crate::util::intern_name(process.name()),
                cpu_usage,
                ram_bytes: ram,
                disk_read_mb_s: r_mb,
                disk_write_mb_s: w_mb,
                eco_qos: None,
                exe_path,
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
            if let Some(primed) = pending.get(&(row.pid, row.start_time)).copied() {
                if let Some(v) = primed {
                    self.eco_cache.insert((row.pid, row.start_time), (v, Instant::now()));
                    row.eco_qos = primed;
                }
            }
        }
        collector_eco::fill_eco_qos(rows, &mut self.eco_cache, &mut self.eco_cursor);
    }
    pub(crate) fn disk_rate(cur: u64, prev: u64, dt: f64) -> u64 {
        if dt <= 1e-6 {
            0
        } else {
            (cur.saturating_sub(prev) as f64 / dt) as u64
        }
    }
    fn elapsed_since_last_scan(&mut self) -> f32 {
        let now = std::time::Instant::now();
        let elapsed = match self.last_scan {
            Some(prev) => now.duration_since(prev).as_secs_f32().max(0.2),
            None => 1.0,
        };
        self.last_scan = Some(now);
        elapsed
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
pub(crate) fn pid_contains(pid: u32, needle: &str) -> bool {
    if needle.is_empty() || !needle.bytes().all(|b| b.is_ascii_digit()) {
        return false;
    }
    let mut buf = [0u8; 10];
    let mut n = pid;
    let mut len = 0usize;
    if n == 0 {
        buf[0] = b'0';
        len = 1;
    } else {
        while n > 0 {
            buf[len] = b'0' + (n % 10) as u8;
            n /= 10;
            len += 1;
        }
        buf[..len].reverse();
    }
    let s = std::str::from_utf8(&buf[..len]).unwrap_or("");
    s.contains(needle)
}
#[cfg(test)]
#[path = "collector_tests.rs"]
mod tests;
