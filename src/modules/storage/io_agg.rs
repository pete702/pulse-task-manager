//! Storage I/O helpers split from collector for the 280-line guard.

use std::cmp::Ordering;
use std::collections::HashMap;
use std::sync::Arc;

use sysinfo::{DiskKind, Disks};

use crate::modules::cpu::model::RawProcessCpuRow;
use crate::modules::storage::model::{DiskMediaKind, DiskVolumeInfo};

#[allow(dead_code)]
pub const TOP_IO_MIN_MB_S: f32 = 0.05;

#[allow(dead_code)]
pub fn media_kind(disk: &sysinfo::Disk) -> DiskMediaKind {
    if disk.is_removable() {
        DiskMediaKind::Removable
    } else {
        match disk.kind() {
            DiskKind::SSD => DiskMediaKind::Ssd,
            DiskKind::HDD => DiskMediaKind::Hdd,
            DiskKind::Unknown(_) => DiskMediaKind::Unknown,
        }
    }
}

#[allow(dead_code)]
pub fn volumes_from_disks(disks: &Disks) -> Vec<DiskVolumeInfo> {
    let mut volumes: Vec<DiskVolumeInfo> = disks
        .list()
        .iter()
        .map(|d| {
            let mount = d.mount_point().to_string_lossy().into_owned();
            let mut info = DiskVolumeInfo::new(
                Arc::from(mount.clone()),
                Arc::from(d.name().to_string_lossy().into_owned()),
                Arc::from(d.file_system().to_string_lossy().into_owned()),
                media_kind(d),
                d.total_space(),
                d.available_space(),
            );
            if let Some((disk, part)) =
                crate::modules::storage::physical_disk::physical_disk_for_mount(&mount)
            {
                info.physical_disk = Some(disk);
                info.partition_number = Some(part);
            }
            info
        })
        .collect();
    volumes.sort_by(|a, b| {
        a.physical_disk
            .unwrap_or(99)
            .cmp(&b.physical_disk.unwrap_or(99))
            .then_with(|| a.mount_point.as_ref().cmp(b.mount_point.as_ref()))
    });
    volumes
}

#[allow(dead_code)]
pub fn clamp_rate(v: f32) -> f32 {
    if v.is_finite() {
        v.clamp(0.0, 100_000.0)
    } else {
        0.0
    }
}

#[allow(dead_code)]
#[allow(clippy::type_complexity)]
pub fn aggregate_io(raw: &[RawProcessCpuRow]) -> (f32, f32, Option<(u32, Arc<str>, f32)>) {
    let mut read_total = 0.0f32;
    let mut write_total = 0.0f32;
    let mut top: Option<(u32, Arc<str>, f32)> = None;
    for row in raw {
        read_total += row.disk_read_mb_s;
        write_total += row.disk_write_mb_s;
        let sum = row.disk_read_mb_s + row.disk_write_mb_s;
        if sum > TOP_IO_MIN_MB_S {
            let is_better = top.as_ref().is_some_and(|(best_pid, best_name, best)| {
                if sum != *best {
                    sum > *best
                } else {
                    row.name
                        .to_ascii_lowercase()
                        .cmp(&best_name.to_ascii_lowercase())
                        .then_with(|| row.pid.cmp(best_pid))
                        == Ordering::Less
                }
            });
            if top.is_none() || is_better {
                top = Some((row.pid, Arc::clone(&row.name), sum));
            }
        }
    }
    (read_total, write_total, top)
}

/// Per-physical-disk process I/O fallback when IOCTL counters are missing.
/// Unresolved drives attribute to the lowest physical disk so IO is never lost.
#[allow(dead_code)]
#[allow(clippy::type_complexity)]
pub fn aggregate_io_by_disk(
    raw: &[RawProcessCpuRow],
    volumes: &[DiskVolumeInfo],
) -> HashMap<u32, (f64, f64)> {
    let mut drive_to_disk: HashMap<char, u32> = HashMap::new();
    let mut first_disk: Option<u32> = None;
    for vol in volumes {
        if let Some(disk) = vol.physical_disk {
            if first_disk.is_none_or(|f| disk < f) {
                first_disk = Some(disk);
            }
            if let Some(letter) =
                crate::modules::storage::physical_disk::drive_letter(vol.mount_point.as_ref())
            {
                drive_to_disk.entry(letter).or_insert(disk);
            }
        }
    }
    let default_disk = first_disk.unwrap_or(0);
    let mut out: HashMap<u32, (f64, f64)> = HashMap::new();
    for row in raw {
        let r = f64::from(row.disk_read_mb_s);
        let w = f64::from(row.disk_write_mb_s);
        if r + w <= 0.0 {
            continue;
        }
        let letter = row.drive_letter.map(|b| (b as char).to_ascii_uppercase());
        let disk = letter.and_then(|l| drive_to_disk.get(&l).copied()).unwrap_or(default_disk);
        let entry = out.entry(disk).or_insert((0.0, 0.0));
        entry.0 += r;
        entry.1 += w;
    }
    out
}
