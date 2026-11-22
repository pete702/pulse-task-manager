//! Unit tests for storage collector: spikes, dedup, and volume health.
use super::*;
use std::sync::Arc;

fn raw(pid: u32, name: &str, read: f32, write: f32) -> RawProcessCpuRow {
    RawProcessCpuRow {
        pid,
        name: Arc::from(name),
        cpu_usage: 0.0,
        raw_cpu_usage: 0.0,
        ram_bytes: 0,
        disk_read_mb_s: read,
        disk_write_mb_s: write,
        eco_qos: None,
        exe_path: None,
        drive_letter: None,
        creation_time: None,
        start_time: 0,
        parent_pid: None,
        command_line: None,
        user: None,
    }
}
#[test]
fn captures_io_spike_at_threshold() {
    let mut c = StorageCollector::new();
    let rows = [raw(7, "writer.exe", 200.0, 200.0), raw(8, "idle.exe", 0.0, 0.0)];
    let snap = c.collect(&rows, 1);
    assert!(snap.read_mb_s + snap.write_mb_s >= IO_SPIKE_THRESHOLD_MB_S as f32);
    let spike = snap.last_io_spike.clone().unwrap_or_default();
    assert_eq!(spike.pid, 7);
    assert!((spike.peak_mb_s - 400.0).abs() < 0.5);
}
#[test]
fn spike_stores_rw_breakdown() {
    let mut c = StorageCollector::new();
    let rows = [raw(7, "writer.exe", 200.0, 200.0)];
    let snap = c.collect(&rows, 1);
    let spike = snap.last_io_spike.unwrap_or_default();
    assert!((spike.read_mb_s - 200.0).abs() < 0.5);
    assert!((spike.write_mb_s - 200.0).abs() < 0.5);
}
#[test]
fn spike_ages_without_new_spike() {
    let mut c = StorageCollector::new();
    let hot = [raw(7, "writer.exe", 400.0, 0.0)];
    let _ = c.collect(&hot, 1);
    let calm = [raw(7, "writer.exe", 0.1, 0.0)];
    let snap = c.collect(&calm, 1);
    let spike = snap.last_io_spike.clone().unwrap_or_default();
    assert_eq!(spike.ticks_ago, 1);
}
#[test]
fn io_spike_threshold_is_350_mb_s() {
    assert_eq!(IO_SPIKE_THRESHOLD_MB_S, 350.0);
    let mut c = StorageCollector::new();
    let hot = [raw(7, "writer.exe", 175.0, 175.0)];
    let snap = c.collect(&hot, 1);
    assert!(snap.last_io_spike.is_some());
    let mut c2 = StorageCollector::new();
    let warm = [raw(8, "writer.exe", 174.0, 174.0)];
    let snap2 = c2.collect(&warm, 1);
    assert!(snap2.last_io_spike.is_none());
}

#[test]
fn disk_list_refresh_ticks_is_five() {
    assert_eq!(DISK_LIST_REFRESH_TICKS, 5);
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
    let hot = [raw(7, "writer.exe", 400.0, 0.0)];
    let _ = c.collect(&hot, 1);
    let calm = [raw(7, "writer.exe", 0.1, 0.0)];
    let mut snap = c.collect(&calm, 1);
    for _ in 0..121 {
        snap = c.collect(&calm, 1);
    }
    assert!(snap.last_io_spike.is_none());
}

#[test]
fn unmapped_volume_gets_per_letter_rates_without_disk0_inflation() {
    use crate::modules::storage::io_agg::{aggregate_io_by_disk, aggregate_io_by_letter};
    use crate::modules::storage::model::{DiskMediaKind, DiskVolumeInfo};
    let gb = 1024 * 1024 * 1024u64;
    let mk = |m: &str| {
        DiskVolumeInfo::new(
            Arc::from(m),
            Arc::from(m),
            Arc::from("NTFS"),
            DiskMediaKind::Ssd,
            100 * gb,
            50 * gb,
        )
    };
    let mut c = mk("C:\\");
    c.physical_disk = Some(0);
    let mut l = mk("L:\\");
    l.physical_disk = None;
    let volumes = vec![c.clone(), l.clone()];
    let mut row = raw(9, "game.exe", 5.0, 2.0);
    row.drive_letter = Some(b'L');
    let by_letter = aggregate_io_by_letter(&[row.clone()]);
    assert_eq!(by_letter.get(&'L'), Some(&(5.0, 2.0)));
    let by_disk = aggregate_io_by_disk(&[row], &volumes);
    let disk0_read = by_disk.get(&0).map(|v| v.0).unwrap_or(0.0);
    assert!(disk0_read < 1.0);
    let mut collector = StorageCollector::new();
    collector.tick_counter = 1;
    collector.cached_volumes = Arc::from(vec![c, l]);
    let mut hot = raw(9, "game.exe", 5.0, 2.0);
    hot.drive_letter = Some(b'L');
    let snap = collector.collect(&[hot], 99);
    let l_vol = snap.volumes.iter().find(|v| v.mount_point.as_ref() == "L:\\").unwrap();
    assert!((l_vol.read_mb_s - 5.0).abs() < 0.5);
    assert!((l_vol.write_mb_s - 2.0).abs() < 0.5);
    let c_vol = snap.volumes.iter().find(|v| v.mount_point.as_ref() == "C:\\").unwrap();
    assert!(c_vol.read_mb_s < 1.0);
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
