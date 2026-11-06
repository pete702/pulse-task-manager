//! Storage collector: volume metadata plus aggregated process I/O.

use std::collections::HashMap;
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
pub struct StorageCollector {
    disks: Disks,
    tick_counter: u32,
    cached_volumes: Arc<[DiskVolumeInfo]>,
    last_io_spike: Option<IoSpikeEvent>,
    last_scan_id: u64,
    prev_per_disk: HashMap<u32, (i64, i64, Instant)>,
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
            prev_per_disk: HashMap::new(),
        }
    }
    #[allow(dead_code)]
    pub fn collect(&mut self, raw_processes: &[RawProcessCpuRow], scan_id: u64) -> StorageSnapshot {
        if self.tick_counter.is_multiple_of(10) {
            self.disks.refresh_list();
            self.cached_volumes = Arc::from(volumes_from_disks(&self.disks));
        }
        let (agg_read, agg_write, top) = aggregate_io(raw_processes);
        // One representative mount per disk: DISK_PERFORMANCE counters are
        // per-disk, so C: and D: on Disk 0 must not both be summed.
        let entries: Vec<(&str, Option<u32>)> =
            self.cached_volumes.iter().map(|v| (v.mount_point.as_ref(), v.physical_disk)).collect();
        let unique_mounts = dedupe_disk_mounts(&entries);
        let unique: Vec<(String, Option<u32>)> = unique_mounts
            .iter()
            .map(|m| {
                let disk = self
                    .cached_volumes
                    .iter()
                    .find(|v| v.mount_point.as_ref() == *m)
                    .and_then(|v| v.physical_disk);
                (m.to_string(), disk)
            })
            .collect();
        let (read_total, write_total, per_disk) = self.per_disk_rates(&unique, agg_read, agg_write);
        let read_total = clamp_rate(read_total);
        let write_total = clamp_rate(write_total);
        // Stamp every volume of a disk with its live per-disk rates.
        let volumes: Arc<[DiskVolumeInfo]> = Arc::from(
            self.cached_volumes
                .iter()
                .map(|v| {
                    let mut info = v.clone();
                    if let Some(d) = v.physical_disk {
                        if let Some((r, w)) = per_disk.get(&d) {
                            info.read_mb_s = *r;
                            info.write_mb_s = *w;
                        }
                    }
                    info
                })
                .collect::<Vec<_>>(),
        );
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
                    timestamp_hms: Arc::from(crate::util::format_timestamp_now()),
                    ticks_ago: 0,
                });
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
            volumes: Arc::clone(&volumes),
            read_mb_s: read_total,
            write_mb_s: write_total,
            top_io_process: top,
            last_io_spike: self.last_io_spike.clone(),
        };
        self.tick_counter = self.tick_counter.wrapping_add(1);
        snapshot
    }
    fn per_disk_rates(
        &mut self,
        unique: &[(String, Option<u32>)],
        agg_read: f32,
        agg_write: f32,
    ) -> (f32, f32, HashMap<u32, (f32, f32)>) {
        let now = Instant::now();
        let mut per_disk: HashMap<u32, (f32, f32)> = HashMap::new();
        let (mut sum_r, mut sum_w, mut any_rate) = (0.0f32, 0.0f32, false);
        for (mount, disk) in unique {
            let Some(disk) = disk else { continue };
            let Some(counters) = physical_disk::perf_counters_for_mount(mount) else { continue };
            let (new_r, new_w) = counters;
            let prev = self.prev_per_disk.insert(*disk, (new_r, new_w, now));
            if let Some((old_r, old_w, at)) = prev {
                let dt = now.duration_since(at).as_secs_f64();
                if (0.5..=10.0).contains(&dt) && new_r >= old_r && new_w >= old_w {
                    let r = ((new_r - old_r) as f64 / dt / 1_048_576.0) as f32;
                    let w = ((new_w - old_w) as f64 / dt / 1_048_576.0) as f32;
                    per_disk.insert(*disk, (r, w));
                    sum_r += r;
                    sum_w += w;
                    any_rate = true;
                }
            }
        }
        if any_rate {
            (clamp_rate(sum_r), clamp_rate(sum_w), per_disk)
        } else {
            // No valid deltas yet (first sample or no IOCTL data) - use agg.
            (agg_read, agg_write, per_disk)
        }
    }
}

fn top_rw(raw: &[RawProcessCpuRow], pid: u32) -> (f32, f32) {
    raw.iter()
        .find(|r| r.pid == pid)
        .map(|r| (r.disk_read_mb_s, r.disk_write_mb_s))
        .unwrap_or((0.0, 0.0))
}

/// Keeps one mount per unique physical disk; None entries stay distinct.
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
        let entries2 = [("C:\\", None), ("D:\\", None), ("H:\\", Some(1))];
        let unique2 = dedupe_disk_mounts(&entries2);
        assert_eq!(unique2.len(), 3);
        assert!(unique2.contains(&"C:\\"));
        assert!(unique2.contains(&"D:\\"));
        assert!(unique2.contains(&"H:\\"));
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
