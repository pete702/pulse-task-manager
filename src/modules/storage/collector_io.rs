//! Pure per-disk I/O rate computation extracted from the storage collector.
use crate::modules::cpu::model::RawProcessCpuRow;
use crate::modules::storage::io_agg::{aggregate_io_by_disk, aggregate_io_by_letter, clamp_rate};
use crate::modules::storage::model::DiskVolumeInfo;
use crate::modules::storage::physical_disk;
use std::collections::HashMap;
use std::time::Instant;

/// Computes clamped per-disk rates plus fallback totals from process I/O.
pub(crate) fn per_disk_rates(
    all: &[(String, Option<u32>)],
    raw_processes: &[RawProcessCpuRow],
    volumes: &[DiskVolumeInfo],
    agg_read: f32,
    agg_write: f32,
    prev_per_disk: &mut HashMap<u32, (i64, i64, Instant)>,
    last_per_disk_rates: &mut HashMap<u32, (f32, f32)>,
) -> (f32, f32, HashMap<u32, (f32, f32)>) {
    let now = Instant::now();
    let mut per_disk: HashMap<u32, (f32, f32)> = HashMap::new();
    let mut sums: HashMap<u32, (i64, i64)> = HashMap::new();
    let mut visited: Vec<u32> = Vec::new();
    for (mount, disk) in all {
        let key = match disk {
            Some(d) => Some(*d),
            None => physical_disk::drive_letter(mount).map(|ch| 0x8000_0000 | (ch as u32)),
        };
        let Some(disk) = key else { continue };
        if visited.contains(&disk) {
            continue;
        }
        visited.push(disk);
        let Some((r, w)) = physical_disk::perf_counters_for_mount(mount) else { continue };
        let e = sums.entry(disk).or_insert((0, 0));
        e.0 += r;
        e.1 += w;
    }
    let mut any_rate = false;
    for (disk, (new_r, new_w)) in sums {
        match prev_per_disk.get(&disk).copied() {
            Some((old_r, old_w, at)) => {
                let dt = now.duration_since(at).as_secs_f64();
                if dt > 0.0 {
                    // Rollback guard: counter reset -> emit zero and re-seed baseline.
                    if new_r < old_r || new_w < old_w {
                        per_disk.insert(disk, (0.0, 0.0));
                        last_per_disk_rates.insert(disk, (0.0, 0.0));
                        prev_per_disk.insert(disk, (new_r, new_w, now));
                        any_rate = true;
                        continue;
                    }
                    if dt < 0.35 {
                        if let Some(prev) = last_per_disk_rates.get(&disk).copied() {
                            per_disk.insert(disk, prev);
                            any_rate = true;
                        }
                        continue;
                    }
                    if new_r >= old_r && new_w >= old_w {
                        let r = ((new_r - old_r) as f64 / dt / 1_048_576.0) as f32;
                        let w = ((new_w - old_w) as f64 / dt / 1_048_576.0) as f32;
                        per_disk.insert(disk, (r, w));
                        last_per_disk_rates.insert(disk, (r, w));
                        any_rate = true;
                    }
                    prev_per_disk.insert(disk, (new_r, new_w, now));
                }
            }
            None => {
                prev_per_disk.insert(disk, (new_r, new_w, now));
            }
        }
    }
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
    let by_letter = aggregate_io_by_letter(raw_processes);
    for vol in volumes {
        if vol.physical_disk.is_some() {
            continue;
        }
        if let Some(l) = physical_disk::drive_letter(vol.mount_point.as_ref()) {
            let key = 0x8000_0000 | (l as u32);
            let need = match per_disk.get(&key) {
                None => true,
                Some((hr, hw)) => *hr == 0.0 && *hw == 0.0,
            };
            if need {
                if let Some((r, w)) = by_letter.get(&l) {
                    per_disk.insert(key, (clamp_rate(*r as f32), clamp_rate(*w as f32)));
                }
            }
        }
    }
    if any_rate {
        let (mut sr, mut sw) = (0.0f32, 0.0f32);
        for (r, w) in per_disk.values() {
            sr += r;
            sw += w;
        }
        (clamp_rate(sr), clamp_rate(sw), per_disk)
    } else {
        (agg_read, agg_write, per_disk)
    }
}
