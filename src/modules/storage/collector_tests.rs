//! Unit tests for storage collector: spikes, dedup, and volume health.
use super::*;
use std::sync::Arc;

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
        drive_letter: None,
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
