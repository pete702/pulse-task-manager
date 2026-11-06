//! Storage I/O helpers split from collector for the 280-line guard.

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
            let is_better = top.as_ref().is_some_and(|(_, _, best)| sum > *best);
            if top.is_none() || is_better {
                top = Some((row.pid, Arc::clone(&row.name), sum));
            }
        }
    }
    (read_total, write_total, top)
}
