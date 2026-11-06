//! Storage telemetry model: volumes, I/O spikes, and history.
//! History itself lives in `history.rs` for the 280-line guard.

use std::sync::Arc;

pub const GIB: u64 = 1024 * 1024 * 1024;

#[allow(dead_code)]
pub const CRITICAL_AVAILABLE_BYTES: u64 = 15 * GIB;

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DiskMediaKind {
    Ssd,
    Hdd,
    Removable,
    #[default]
    Unknown,
}

impl DiskMediaKind {
    #[allow(dead_code)]
    pub fn badge(self) -> &'static str {
        match self {
            DiskMediaKind::Ssd => "SSD",
            DiskMediaKind::Hdd => "HDD",
            DiskMediaKind::Removable => "USB",
            DiskMediaKind::Unknown => "DISK",
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DiskVolumeInfo {
    pub mount_point: Arc<str>,
    pub name: Arc<str>,
    pub fs_type: Arc<str>,
    pub kind: DiskMediaKind,
    pub total_bytes: u64,
    pub available_bytes: u64,
    pub used_pct: f32,
    pub is_critical_space: bool,
    pub physical_disk: Option<u32>,
    pub partition_number: Option<u32>,
}

impl DiskVolumeInfo {
    #[allow(dead_code)]
    pub fn new(
        mount_point: Arc<str>,
        name: Arc<str>,
        fs_type: Arc<str>,
        kind: DiskMediaKind,
        total_bytes: u64,
        available_bytes: u64,
    ) -> Self {
        let used_pct = used_pct_for(total_bytes, available_bytes);
        let is_critical_space = is_critical_space(used_pct, total_bytes, available_bytes);
        Self {
            mount_point,
            name,
            fs_type,
            kind,
            total_bytes,
            available_bytes,
            used_pct,
            is_critical_space,
            physical_disk: None,
            partition_number: None,
        }
    }
}

#[allow(dead_code)]
pub fn used_pct_for(total_bytes: u64, available_bytes: u64) -> f32 {
    if total_bytes == 0 {
        return 0.0;
    }
    let used = total_bytes.saturating_sub(available_bytes) as f64 / total_bytes as f64 * 100.0;
    used.clamp(0.0, 100.0) as f32
}

#[allow(dead_code)]
pub fn is_critical_space(used_pct: f32, total_bytes: u64, available_bytes: u64) -> bool {
    used_pct >= 90.0 || (total_bytes >= 64 * GIB && available_bytes < 10 * GIB)
}

#[allow(dead_code)]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct IoSpikeEvent {
    pub pid: u32,
    pub process_name: Arc<str>,
    pub peak_mb_s: f32,
    pub read_mb_s: f32,
    pub write_mb_s: f32,
    pub timestamp_hms: Arc<str>,
    pub ticks_ago: u32,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct StorageSnapshot {
    pub volumes: Arc<[DiskVolumeInfo]>,
    pub read_mb_s: f32,
    pub write_mb_s: f32,
    pub top_io_process: Option<(u32, Arc<str>, f32)>,
    pub last_io_spike: Option<IoSpikeEvent>,
}

pub use crate::modules::storage::history::StorageHistory;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn badge_mapping() {
        assert_eq!(DiskMediaKind::Ssd.badge(), "SSD");
        assert_eq!(DiskMediaKind::Hdd.badge(), "HDD");
        assert_eq!(DiskMediaKind::Removable.badge(), "USB");
        assert_eq!(DiskMediaKind::Unknown.badge(), "DISK");
    }
    #[test]
    fn volume_critical_detection() {
        let gb = GIB;
        let mk = |m: &str, kind: DiskMediaKind, total: u64, avail: u64| {
            DiskVolumeInfo::new(Arc::from(m), Arc::from(m), Arc::from("NTFS"), kind, total, avail)
        };
        let full = mk("C:", DiskMediaKind::Ssd, 100 * gb, 5 * gb);
        assert!((full.used_pct - 95.0).abs() < 0.5);
        assert!(full.is_critical_space);
        assert!(mk("D:", DiskMediaKind::Hdd, 500 * gb, 10 * gb).is_critical_space);
        assert!(!mk("E:", DiskMediaKind::Ssd, 500 * gb, 250 * gb).is_critical_space);
        assert!(!mk("F:", DiskMediaKind::Removable, 8 * gb, 7 * gb).is_critical_space);
        assert!(is_critical_space(used_pct_for(80 * gb, 9 * gb), 80 * gb, 9 * gb));
    }
}
