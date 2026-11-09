//! CPU collector tests.

use super::{first_sample_disk_bytes, guarded_disk_bytes, CpuCollector};
#[test]
fn first_sample_zeroes_lifetime_even_when_young() {
    assert_eq!(
        first_sample_disk_bytes(5_000_000_000, 3_000_000_000, 5_000_000_000, 3_000_000_000),
        (0, 0)
    );
    assert_eq!(first_sample_disk_bytes(1_000, 2_000, 5_000_000_000, 3_000_000_000), (1_000, 2_000));
    assert_eq!(first_sample_disk_bytes(0, 0, 0, 0), (0, 0));
}
#[test]
fn tick2_first_io_on_seen_pid_is_not_zeroed() {
    assert_eq!(guarded_disk_bytes(100, 100, 100, 100, false), (100, 100));
    assert_eq!(guarded_disk_bytes(100, 100, 100, 100, true), (0, 0));
}
#[test]
fn pid_reuse_with_new_start_time_is_new() {
    let mut c = CpuCollector::new();
    c.seen_pids.insert((100, 1));
    assert!(!c.is_new(100, 1));
    assert!(c.is_new(100, 2));
    assert!(c.is_new(101, 1));
}
#[test]
fn disk_rate_uses_production_helper() {
    assert_eq!(CpuCollector::disk_rate(200, 100, 1.0), 100);
    assert_eq!(CpuCollector::disk_rate(100, 200, 1.0), 0);
    assert_eq!(CpuCollector::disk_rate(200, 100, 0.0), 0);
    assert_eq!(CpuCollector::disk_rate(1_000_000, 0, 2.0), 500_000);
}
