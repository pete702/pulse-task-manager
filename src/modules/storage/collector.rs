//! Storage collector: volume metadata plus aggregated process I/O.
use crate::modules::cpu::model::RawProcessCpuRow;
use crate::modules::storage::io_agg::{
    aggregate_io, aggregate_io_by_letter, clamp_rate, volumes_from_disks,
};
use crate::modules::storage::model::{DiskVolumeInfo, IoSpikeEvent, StorageSnapshot};
use crate::modules::storage::physical_disk;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;
use sysinfo::Disks;
pub const IO_SPIKE_THRESHOLD_MB_S: f64 = 350.0;
pub const DISK_LIST_REFRESH_TICKS: u64 = 5;
static FORCE_STORAGE_REFRESH: AtomicBool = AtomicBool::new(false);
pub fn request_storage_refresh() {
    FORCE_STORAGE_REFRESH.store(true, Ordering::Release);
}
pub fn should_refresh_disk_list(tick_count: u64, is_empty: bool, force: bool) -> bool {
    force || is_empty || tick_count.is_multiple_of(DISK_LIST_REFRESH_TICKS)
}
pub struct StorageCollector {
    disks: Disks,
    tick_counter: u32,
    cached_volumes: Arc<[DiskVolumeInfo]>,
    last_io_spike: Option<IoSpikeEvent>,
    last_scan_id: u64,
    prev_per_disk: HashMap<u32, (i64, i64, Instant)>,
    last_per_disk_rates: HashMap<u32, (f32, f32)>,
    force_disk_refresh: bool,
}
impl StorageCollector {
    pub fn new() -> Self {
        Self {
            disks: Disks::new(),
            tick_counter: 0,
            cached_volumes: Arc::from(Vec::new()),
            last_io_spike: None,
            last_scan_id: 0,
            prev_per_disk: HashMap::new(),
            last_per_disk_rates: HashMap::new(),
            force_disk_refresh: false,
        }
    }
    pub fn collect(&mut self, raw_processes: &[RawProcessCpuRow], scan_id: u64) -> StorageSnapshot {
        prune_per_disk_maps(&mut self.prev_per_disk, &mut self.last_per_disk_rates);
        let force = self.force_disk_refresh || FORCE_STORAGE_REFRESH.swap(false, Ordering::AcqRel);
        if should_refresh_disk_list(self.tick_counter as u64, self.cached_volumes.is_empty(), force)
        {
            self.disks.refresh_list();
            self.cached_volumes = Arc::from(volumes_from_disks(&self.disks));
            self.force_disk_refresh = false;
        }
        let (agg_read, agg_write, top) = aggregate_io(raw_processes);
        let all: Vec<(String, Option<u32>)> = self
            .cached_volumes
            .iter()
            .map(|v| (v.mount_point.to_string(), v.physical_disk))
            .collect();
        let vr = Arc::clone(&self.cached_volumes);
        let (read_total, write_total, per_disk) =
            crate::modules::storage::collector_io::per_disk_rates(
                &all,
                raw_processes,
                &vr,
                agg_read,
                agg_write,
                &mut self.prev_per_disk,
                &mut self.last_per_disk_rates,
            );
        let read_total = clamp_rate(read_total);
        let write_total = clamp_rate(write_total);
        let by_letter = aggregate_io_by_letter(raw_processes);
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
                    } else if let Some(letter) = physical_disk::drive_letter(v.mount_point.as_ref())
                    {
                        let key = 0x8000_0000 | (letter as u32);
                        if let Some((r, w)) = per_disk.get(&key) {
                            info.read_mb_s = *r;
                            info.write_mb_s = *w;
                        } else if let Some((r, w)) = by_letter.get(&letter) {
                            info.read_mb_s = clamp_rate(*r as f32);
                            info.write_mb_s = clamp_rate(*w as f32);
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
        let spike_now = (total as f64) >= IO_SPIKE_THRESHOLD_MB_S
            || (top_sum as f64) >= IO_SPIKE_THRESHOLD_MB_S;
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
}
fn prune_per_disk_maps(
    prev: &mut HashMap<u32, (i64, i64, Instant)>,
    rates: &mut HashMap<u32, (f32, f32)>,
) {
    if prev.len() > 32 {
        prev.clear();
        rates.clear();
    }
}
fn top_rw(raw: &[RawProcessCpuRow], pid: u32) -> (f32, f32) {
    raw.iter()
        .find(|r| r.pid == pid)
        .map(|r| (r.disk_read_mb_s, r.disk_write_mb_s))
        .unwrap_or((0.0, 0.0))
}

impl Drop for StorageCollector {
    fn drop(&mut self) {
        crate::modules::storage::physical_disk::clear_handle_cache();
    }
}

impl Default for StorageCollector {
    fn default() -> Self {
        Self::new()
    }
}
#[cfg(test)]
mod refresh_tests {
    use super::*;
    #[test]
    fn refresh_at_five_ticks_and_empty() {
        assert!(should_refresh_disk_list(0, true, false));
        assert!(should_refresh_disk_list(5, false, false));
        assert!(should_refresh_disk_list(10, false, false));
        assert!(!should_refresh_disk_list(1, false, false));
        assert!(!should_refresh_disk_list(2, false, false));
        assert!(should_refresh_disk_list(3, false, true));
    }

    #[test]
    fn prune_rates_with_prev() {
        let mut prev = HashMap::new();
        let mut rates = HashMap::new();
        for i in 0..33 {
            prev.insert(i, (0, 0, Instant::now()));
            rates.insert(i, (1.0, 2.0));
        }
        prune_per_disk_maps(&mut prev, &mut rates);
        assert!(prev.is_empty());
        assert!(rates.is_empty());
    }
}

#[cfg(test)]
#[path = "collector_tests.rs"]
mod tests;
