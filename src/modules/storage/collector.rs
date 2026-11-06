//! Storage collector: volume metadata plus aggregated process I/O.

use std::sync::Arc;
use std::time::Instant;

use sysinfo::Disks;

use crate::modules::cpu::model::RawProcessCpuRow;
use crate::modules::storage::io_agg::{aggregate_io, clamp_rate, volumes_from_disks};
use crate::modules::storage::model::{DiskVolumeInfo, IoSpikeEvent, StorageSnapshot};
use crate::modules::storage::physical_disk;

#[allow(dead_code)]
pub const IO_SPIKE_THRESHOLD_MB_S: f32 = 80.0;

#[allow(dead_code)]
fn local_hms() -> String {
    crate::util::format_timestamp_now()
}

#[allow(dead_code)]
pub struct StorageCollector {
    disks: Disks,
    tick_counter: u32,
    cached_volumes: Arc<[DiskVolumeInfo]>,
    last_io_spike: Option<IoSpikeEvent>,
    last_scan_id: u64,
    last_hw: Option<(i64, i64)>,
    last_hw_at: Option<Instant>,
}

impl StorageCollector {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            disks: Disks::new(),
            tick_counter: 0,
            cached_volumes: Arc::from(Vec::new()),
            last_io_spike: None,
            last_scan_id: 0,
            last_hw: None,
            last_hw_at: None,
        }
    }
    #[allow(dead_code)]
    pub fn collect(&mut self, raw_processes: &[RawProcessCpuRow], scan_id: u64) -> StorageSnapshot {
        if self.tick_counter.is_multiple_of(10) {
            self.disks.refresh_list();
            self.cached_volumes = Arc::from(volumes_from_disks(&self.disks));
        }
        let (agg_read, agg_write, top) = aggregate_io(raw_processes);
        let unique_owned: Vec<String> = {
            let entries: Vec<(&str, Option<u32>)> = self
                .cached_volumes
                .iter()
                .map(|v| (v.mount_point.as_ref(), v.physical_disk))
                .collect();
            let unique = dedupe_disk_mounts(&entries);
            unique.iter().map(|s| (*s).to_string()).collect()
        };
        let mounts: Vec<&str> = unique_owned.iter().map(|s| s.as_str()).collect();
        let (read_total, write_total) = self.hw_rates(&mounts).unwrap_or((agg_read, agg_write));
        let read_total = clamp_rate(read_total);
        let write_total = clamp_rate(write_total);
        let is_fresh_scan = !raw_processes.is_empty() && scan_id != self.last_scan_id;
        self.last_scan_id = scan_id;
        let total = read_total + write_total;
        let top_sum = top.as_ref().map(|t| t.2).unwrap_or(0.0);
        let spike_now = total >= IO_SPIKE_THRESHOLD_MB_S || top_sum >= IO_SPIKE_THRESHOLD_MB_S;
        if is_fresh_scan && spike_now {
            if let Some((pid, name, peak)) = top.clone() {
                let (r, w) = top_rw(raw_processes, pid);
                self.last_io_spike = Some(IoSpikeEvent {
                    pid,
                    process_name: name,
                    peak_mb_s: peak,
                    read_mb_s: r,
                    write_mb_s: w,
                    timestamp_hms: Arc::from(local_hms()),
                    ticks_ago: 0,
                });
            } else if let Some(spike) = self.last_io_spike.as_mut() {
                let _ = spike;
            } else {
                self.last_io_spike = None;
            }
        } else {
            let expired = match self.last_io_spike.as_mut() {
                Some(spike) => {
                    spike.ticks_ago = spike.ticks_ago.saturating_add(1);
                    spike.ticks_ago > 120
                }
                None => false,
            };
            if expired {
                self.last_io_spike = None;
            }
        }
        let snapshot = StorageSnapshot {
            volumes: Arc::clone(&self.cached_volumes),
            read_mb_s: read_total,
            write_mb_s: write_total,
            top_io_process: top,
            last_io_spike: self.last_io_spike.clone(),
        };
        self.tick_counter = self.tick_counter.wrapping_add(1);
        snapshot
    }
    #[allow(dead_code)]
    pub fn last_io_spike(&self) -> Option<&IoSpikeEvent> {
        self.last_io_spike.as_ref()
    }
    #[allow(dead_code)]
    pub fn cached_volumes(&self) -> &[DiskVolumeInfo] {
        &self.cached_volumes
    }

    fn hw_rates(&mut self, mounts: &[&str]) -> Option<(f32, f32)> {
        let now = Instant::now();
        let sums = physical_disk::sum_perf_counters(mounts);
        let prev = self.last_hw;
        let prev_at = self.last_hw_at;
        self.last_hw = sums;
        self.last_hw_at = Some(now);
        let (old_r, old_w) = prev?;
        let (new_r, new_w) = sums?;
        let dt = prev_at?.elapsed().as_secs_f64();
        if dt < 0.5 || dt > 10.0 {
            return None;
        }
        if new_r < old_r || new_w < old_w {
            return None;
        }
        let read = (new_r - old_r) as f64 / dt / 1_048_576.0;
        let write = (new_w - old_w) as f64 / dt / 1_048_576.0;
        Some((read as f32, write as f32))
    }
}

fn top_rw(raw: &[RawProcessCpuRow], pid: u32) -> (f32, f32) {
    raw.iter()
        .find(|r| r.pid == pid)
        .map(|r| (r.disk_read_mb_s, r.disk_write_mb_s))
        .unwrap_or((0.0, 0.0))
}

/// Keeps one mount per unique physical disk; None entries stay distinct.
/// DISK_PERFORMANCE counters are per-disk, so C: and D: on Disk 0 must
/// not both be queried (double-count + duplicate handles).
pub(crate) fn dedupe_disk_mounts<'a>(entries: &[(&'a str, Option<u32>)]) -> Vec<&'a str> {
    let mut seen: Vec<u32> = Vec::new();
    let mut out: Vec<&'a str> = Vec::new();
    for (mount, disk) in entries {
        if let Some(d) = disk {
            if seen.contains(d) {
                continue;
            }
            seen.push(*d);
        }
        out.push(mount);
    }
    out
}

impl Default for StorageCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn raw(pid: u32, name: &str, read: f32, write: f32) -> RawProcessCpuRow {
        RawProcessCpuRow {
            pid,
            name: Arc::from(name),
            cpu_usage: 0.0,
            ram_bytes: 0,
            disk_read_mb_s: read,
            disk_write_mb_s: write,
            eco_qos: None,
            exe_path: None,
            creation_time: None,
            start_time: 0,
        }
    }
    #[test]
    fn captures_io_spike_at_threshold() {
        let mut c = StorageCollector::new();
        let rows = [raw(7, "writer.exe", 60.0, 30.0), raw(8, "idle.exe", 0.0, 0.0)];
        let snap = c.collect(&rows, 1);
        assert!(snap.read_mb_s + snap.write_mb_s >= IO_SPIKE_THRESHOLD_MB_S);
        let spike = snap.last_io_spike.clone().unwrap_or_default();
        assert_eq!(spike.pid, 7);
        assert!((spike.peak_mb_s - 90.0).abs() < 0.5);
    }
    #[test]
    fn spike_stores_rw_breakdown() {
        let mut c = StorageCollector::new();
        let rows = [raw(7, "writer.exe", 60.0, 30.0)];
        let snap = c.collect(&rows, 1);
        let spike = snap.last_io_spike.unwrap_or_default();
        assert!((spike.read_mb_s - 60.0).abs() < 0.5);
        assert!((spike.write_mb_s - 30.0).abs() < 0.5);
    }
    #[test]
    fn spike_ages_without_new_spike() {
        let mut c = StorageCollector::new();
        let hot = [raw(7, "writer.exe", 90.0, 0.0)];
        let _ = c.collect(&hot, 1);
        let calm = [raw(7, "writer.exe", 0.1, 0.0)];
        let snap = c.collect(&calm, 1);
        let spike = snap.last_io_spike.clone().unwrap_or_default();
        assert_eq!(spike.ticks_ago, 1);
    }
    #[test]
    fn top_io_ignores_quiet_processes() {
        let mut c = StorageCollector::new();
        let rows = [raw(1, "quiet.exe", 0.02, 0.02)];
        let snap = c.collect(&rows, 1);
        assert!(snap.top_io_process.is_none());
        assert!(snap.last_io_spike.is_none());
    }
    #[test]
    fn spike_expires_after_120_ticks() {
        let mut c = StorageCollector::new();
        let hot = [raw(7, "writer.exe", 90.0, 0.0)];
        let _ = c.collect(&hot, 1);
        let calm = [raw(7, "writer.exe", 0.1, 0.0)];
        let mut snap = c.collect(&calm, 1);
        for _ in 0..121 {
            snap = c.collect(&calm, 1);
        }
        assert!(snap.last_io_spike.is_none());
    }
    #[test]
    fn dedup_selects_one_per_disk() {
        let entries = [("C:\\", Some(0)), ("D:\\", Some(0)), ("H:\\", Some(1))];
        let unique = dedupe_disk_mounts(&entries);
        assert_eq!(unique.len(), 2);
        assert!(unique.contains(&"C:\\"));
        assert!(unique.contains(&"H:\\"));
        assert!(!unique.contains(&"D:\\"));
    }
    #[test]
    fn dedup_keeps_none_distinct() {
        let entries = [("C:\\", None), ("D:\\", None), ("H:\\", Some(1))];
        let unique = dedupe_disk_mounts(&entries);
        assert_eq!(unique.len(), 3);
        assert!(unique.contains(&"C:\\"));
        assert!(unique.contains(&"D:\\"));
        assert!(unique.contains(&"H:\\"));
    }
    #[test]
    fn volume_low_space_detection() {
        use crate::modules::storage::model::{is_critical_space, used_pct_for};
        let gb = 1024 * 1024 * 1024u64;
        assert!(is_critical_space(used_pct_for(100 * gb, 5 * gb), 100 * gb, 5 * gb));
        assert!(is_critical_space(95.0, 500 * gb, 25 * gb));
        assert!(!is_critical_space(used_pct_for(500 * gb, 250 * gb), 500 * gb, 250 * gb));
        assert_eq!(used_pct_for(0, 0), 0.0);
        assert!(!is_critical_space(used_pct_for(8 * gb, 7 * gb), 8 * gb, 7 * gb));
    }
}
