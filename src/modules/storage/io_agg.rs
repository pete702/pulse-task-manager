//! Storage I/O helpers split from collector for the 280-line guard.

use std::cmp::Ordering;
use std::collections::HashMap;
use std::sync::Arc;

use sysinfo::{DiskKind, Disks};

use crate::modules::cpu::model::RawProcessCpuRow;
use crate::modules::storage::model::{DiskMediaKind, DiskVolumeInfo};

pub const TOP_IO_MIN_MB_S: f32 = 0.05;

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

pub fn clamp_rate(v: f32) -> f32 {
    if v.is_finite() {
        v.clamp(0.0, 100_000.0)
    } else {
        0.0
    }
}

/// Aggregates per-process disk I/O. Top process tie-break: higher total wins;
/// on a tie, the lexicographically smaller (case-insensitive) name wins, then
/// the smaller PID. Returns `None` top when all processes are idle (neutral).
#[allow(clippy::type_complexity)]
pub fn aggregate_io(raw: &[RawProcessCpuRow]) -> (f32, f32, Option<(u32, Arc<str>, f32)>) {
    let mut read_total = 0.0f32;
    let mut write_total = 0.0f32;
    let mut top: Option<(u32, Arc<str>, f32)> = None;
    for row in raw {
        let rr = clamp_rate(row.disk_read_mb_s);
        let ww = clamp_rate(row.disk_write_mb_s);
        read_total += rr;
        write_total += ww;
        let sum = rr + ww;
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
/// Known letters of unmapped volumes never fall through to Disk 0.
#[allow(clippy::type_complexity)]
pub fn aggregate_io_by_disk(
    raw: &[RawProcessCpuRow],
    volumes: &[DiskVolumeInfo],
) -> HashMap<u32, (f64, f64)> {
    let mut drive_to_disk: HashMap<char, u32> = HashMap::new();
    let mut unmapped: Vec<char> = Vec::new();
    for vol in volumes {
        let letter = crate::modules::storage::physical_disk::drive_letter(vol.mount_point.as_ref());
        if let Some(disk) = vol.physical_disk {
            if let Some(letter) = letter {
                drive_to_disk.entry(letter).or_insert(disk);
            }
        } else if let Some(letter) = letter {
            unmapped.push(letter);
        }
    }
    let mut out: HashMap<u32, (f64, f64)> = HashMap::new();
    for row in raw {
        let r = f64::from(clamp_rate(row.disk_read_mb_s));
        let w = f64::from(clamp_rate(row.disk_write_mb_s));
        if r + w <= 0.0 {
            continue;
        }
        let letter = row.drive_letter.map(|b| (b as char).to_ascii_uppercase());
        if let Some(l) = letter {
            if unmapped.contains(&l) {
                continue;
            }
        }
        let Some(disk) = letter.and_then(|l| drive_to_disk.get(&l).copied()) else {
            continue;
        };
        let entry = out.entry(disk).or_insert((0.0, 0.0));
        entry.0 += r;
        entry.1 += w;
    }
    out
}

pub(super) fn aggregate_io_by_letter(raw: &[RawProcessCpuRow]) -> HashMap<char, (f64, f64)> {
    let mut out: HashMap<char, (f64, f64)> = HashMap::new();
    for row in raw {
        let Some(b) = row.drive_letter else { continue };
        let letter = (b as char).to_ascii_uppercase();
        if !letter.is_ascii_alphabetic() {
            continue;
        }
        let r = f64::from(clamp_rate(row.disk_read_mb_s));
        let w = f64::from(clamp_rate(row.disk_write_mb_s));
        if r + w <= 0.0 {
            continue;
        }
        let e = out.entry(letter).or_insert((0.0, 0.0));
        e.0 += r;
        e.1 += w;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::cpu::model::RawProcessCpuRow;
    use crate::modules::storage::model::DiskVolumeInfo;
    use std::sync::Arc;

    #[test]
    fn unmapped_or_unc_does_not_hit_disk_zero() {
        let mut vol = DiskVolumeInfo::new(
            Arc::from("C:\\"),
            Arc::from("C"),
            Arc::from("NTFS"),
            crate::modules::storage::model::DiskMediaKind::Ssd,
            100,
            50,
        );
        vol.physical_disk = Some(1);
        let raw = RawProcessCpuRow {
            pid: 1,
            name: Arc::from("app.exe"),
            cpu_usage: 0.0,
            raw_cpu_usage: 0.0,
            ram_bytes: 0,
            disk_read_mb_s: 10.0,
            disk_write_mb_s: 0.0,
            eco_qos: None,
            exe_path: None,
            drive_letter: None,
            creation_time: None,
            start_time: 1,
            parent_pid: None,
            command_line: None,
            user: None,
        };
        let out = aggregate_io_by_disk(&[raw], &[vol]);
        assert!(!out.contains_key(&0));
    }
}
