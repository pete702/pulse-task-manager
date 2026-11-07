//! Storage collector: volume metadata plus aggregated process I/O.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use sysinfo::Disks;

use crate::modules::cpu::model::RawProcessCpuRow;
use crate::modules::storage::io_agg::{
    aggregate_io, aggregate_io_by_disk, clamp_rate, volumes_from_disks,
};
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
        // Sum every volume handle per physical disk: IOCTL on volume
        // counters is per-volume only, so D:/E: would be lost if skipped.
        let all: Vec<(String, Option<u32>)> = self
            .cached_volumes
            .iter()
            .map(|v| (v.mount_point.to_string(), v.physical_disk))
            .collect();
        let volumes_for_rates = Arc::clone(&self.cached_volumes);
        let (read_total, write_total, per_disk) =
            self.per_disk_rates(&all, raw_processes, &volumes_for_rates, agg_read, agg_write);
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
        all: &[(String, Option<u32>)],
        raw_processes: &[RawProcessCpuRow],
        volumes: &[DiskVolumeInfo],
        agg_read: f32,
        agg_write: f32,
    ) -> (f32, f32, HashMap<u32, (f32, f32)>) {
        let now = Instant::now();
        let mut per_disk: HashMap<u32, (f32, f32)> = HashMap::new();
        let mut sums: HashMap<u32, (i64, i64)> = HashMap::new();
        for (mount, disk) in all {
            let Some(disk) = disk else { continue };
            let Some((r, w)) = physical_disk::perf_counters_for_mount(mount) else { continue };
            let e = sums.entry(*disk).or_insert((0, 0));
            e.0 += r;
            e.1 += w;
        }
        let (mut sum_r, mut sum_w, mut any_rate) = (0.0f32, 0.0f32, false);
        for (disk, (new_r, new_w)) in sums {
            match self.prev_per_disk.get(&disk).copied() {
                Some((old_r, old_w, at)) => {
                    let dt = now.duration_since(at).as_secs_f64();
                    if dt > 0.0 {
                        if new_r >= old_r && new_w >= old_w {
                            let r = ((new_r - old_r) as f64 / dt / 1_048_576.0) as f32;
                            let w = ((new_w - old_w) as f64 / dt / 1_048_576.0) as f32;
                            per_disk.insert(disk, (r, w));
                            sum_r += r;
                            sum_w += w;
                            any_rate = true;
                        }
                        self.prev_per_disk.insert(disk, (new_r, new_w, now));
                    }
                }
                None => {
                    self.prev_per_disk.insert(disk, (new_r, new_w, now));
                }
            }
        }
        // Process-attributed fallback for missing/zero hardware disks.
        let fallback = aggregate_io_by_disk(raw_processes, volumes);
        for (disk, fr_fw) in fallback {
            let r = clamp_rate(fr_fw.0 as f32);
            let w = clamp_rate(fr_fw.1 as f32);
            if r + w <= 0.0 {
                continue;
            }
            let need = match per_disk.get(&disk) {
                None => true,
                Some((hr, hw)) => *hr == 0.0 && *hw == 0.0,
            };
            if need {
                per_disk.insert(disk, (r, w));
            }
        }
        if any_rate {
            let (mut sr, mut sw) = (0.0f32, 0.0f32);
            for (r, w) in per_disk.values() {
                let _ = (sr, sw);
                sr += r;
                sw += w;
            }
            let _ = (sum_r, sum_w);
            (clamp_rate(sr), clamp_rate(sw), per_disk)
        } else {
            // No valid hardware deltas yet: total stays aggregate, per-disk live.
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
#[allow(dead_code)]
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
#[path = "collector_tests.rs"]
mod tests;
