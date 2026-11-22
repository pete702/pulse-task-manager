//! OS CPU collector for PULSE: gathers real CPU metrics via sysinfo
//! and computes per-process CPU percentages in a single pass.
pub use crate::modules::cpu::collector_cpu::{collect_cpu, warmup_cpu};
use crate::modules::cpu::collector_eco;
use crate::modules::cpu::model::RawProcessCpuRow;
#[allow(unused_imports)]
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Instant;
use sysinfo::{ProcessRefreshKind, System, UpdateKind};
type PrevDiskEntry = (u64, u64, Option<u64>, f32, f32);

pub struct CpuCollector {
    pub seen_pids: HashSet<(u32, u64)>,
    eco_cache: HashMap<(u32, u64), (bool, Instant)>,
    eco_cursor: usize,
    prev_disk: HashMap<(u32, u64), PrevDiskEntry>,
    creation_cache: HashMap<(u32, u64), Option<u64>>,
    last_scan: Option<Instant>,
}
impl CpuCollector {
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
    #[cfg(test)]
    pub fn is_new(&self, pid: u32, start_time: u64) -> bool {
        !self.seen_pids.contains(&(pid, start_time))
    }
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
                .with_exe(UpdateKind::OnlyIfNotSet)
                .with_cmd(UpdateKind::OnlyIfNotSet)
                .with_user(UpdateKind::OnlyIfNotSet),
        );
        let elapsed_secs = self.elapsed_since_last_scan();
        let mut live: HashSet<(u32, u64)> = HashSet::with_capacity(sys.processes().len());
        let cores = logical_cores.max(1) as f32;
        let mut creation_probes: usize = 0;
        let probe_budget = creation_probe_budget(self.creation_cache.len());
        let mut raw_rows: Vec<RawProcessCpuRow> = Vec::with_capacity(sys.processes().len());
        for (pid, process) in sys.processes().iter() {
            let pid_u32 = pid.as_u32();
            let start_time = process.start_time();
            let key = (pid_u32, start_time);
            live.insert(key);
            let raw_cpu_usage = process.cpu_usage();
            let raw_cpu = raw_cpu_usage / cores;
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
                } else if pid_u32 != 0 && pid_u32 != 4 && creation_probes < probe_budget {
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
            let short_dt = elapsed_secs > 0.0 && elapsed_secs < 0.2;
            let (r_mb, w_mb) = if short_dt {
                prev.map(|(_, _, _, rr, ww)| (rr, ww)).unwrap_or((0.0, 0.0))
            } else {
                Self::disk_rates_from_prev(
                    prev.map(|(r, w, _, _, _)| (r, w)),
                    tot_r,
                    tot_w,
                    elapsed_secs.max(0.2),
                )
            };
            let exe_path = exe.and_then(|p| p.to_str()).filter(|s| !s.is_empty()).map(Arc::from);
            let meta = crate::modules::cpu::proc_meta::get_or_compute(pid_u32, start_time, || {
                let cmd = process.cmd();
                let command_line =
                    if cmd.is_empty() { Arc::from("") } else { Arc::from(cmd.join(" ")) };
                let user = process
                    .user_id()
                    .map(|u| Arc::from(format!("{}", **u)))
                    .unwrap_or_else(|| Arc::from(""));
                (command_line, user)
            });
            let command_line =
                (!meta.command_line.is_empty()).then(|| Arc::clone(&meta.command_line));
            let user = (!meta.user.is_empty()).then(|| Arc::clone(&meta.user));
            self.prev_disk.insert(key, (tot_r, tot_w, creation_time, r_mb, w_mb));
            raw_rows.push(RawProcessCpuRow {
                pid: pid_u32,
                name: crate::util::intern_name(process.name()),
                cpu_usage,
                raw_cpu_usage: if raw_cpu_usage.is_finite() {
                    raw_cpu_usage.clamp(0.0, 10000.0)
                } else {
                    0.0
                },
                ram_bytes: ram,
                disk_read_mb_s: r_mb,
                disk_write_mb_s: w_mb,
                eco_qos: None,
                exe_path,
                drive_letter: drive,
                creation_time,
                start_time,
                parent_pid: process.parent().map(|p| p.as_u32()),
                command_line,
                user,
            });
        }
        self.fill_eco_qos(&mut raw_rows);
        self.eco_cache.retain(|k, _| live.contains(k));
        self.prev_disk.retain(|k, _| live.contains(k));
        self.creation_cache.retain(|k, _| live.contains(k));
        self.seen_pids = live;
        raw_rows.sort_by(|a, b| b.cpu_usage.total_cmp(&a.cpu_usage));
        raw_rows
    }
    fn fill_eco_qos(&mut self, rows: &mut [RawProcessCpuRow]) {
        let pending: HashMap<(u32, u64), Option<bool>> = crate::platform::qos::PENDING_ECO
            .get()
            .map(|slot| {
                let mut g = slot.lock().unwrap_or_else(|e| e.into_inner());
                let map = std::mem::take(&mut g.0);
                g.1.clear();
                map.into_iter().collect()
            })
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
    pub(crate) fn disk_rates_from_prev(
        prev: Option<(u64, u64)>,
        read_bytes: u64,
        written_bytes: u64,
        elapsed_secs: f32,
    ) -> (f32, f32) {
        match prev {
            Some((pr, pw)) => (
                Self::disk_rate(read_bytes, pr, elapsed_secs as f64) as f32 / 1_048_576.0,
                Self::disk_rate(written_bytes, pw, elapsed_secs as f64) as f32 / 1_048_576.0,
            ),
            None => (0.0, 0.0),
        }
    }
    fn elapsed_since_last_scan(&mut self) -> f32 {
        let now = std::time::Instant::now();
        let elapsed = match self.last_scan {
            Some(prev) => now.duration_since(prev).as_secs_f32(),
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
pub(crate) fn creation_probe_budget(cached_len: usize) -> usize {
    if cached_len < 32 {
        256
    } else {
        128
    }
}

#[cfg(test)]
#[path = "collector_tests.rs"]
mod tests;
