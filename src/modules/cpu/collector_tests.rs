//! CPU collector tests.

use super::{first_sample_disk_bytes, guarded_disk_bytes, pid_contains, CpuCollector};
use crate::modules::cpu::table_sort::contains_ci;
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
#[test]
fn contains_ci_ascii_and_unicode() {
    assert!(contains_ci("Chrome.exe", "chrome"));
    assert!(contains_ci("CHROME.EXE", "chrome"));
    assert!(!contains_ci("chrome.exe", "edge"));
    assert!(contains_ci("", ""));
    let upper = "\u{041f}\u{0440}\u{043e}\u{0446}\u{0435}\u{0441}";
    let lower = "\u{043f}\u{0440}\u{043e}\u{0446}\u{0435}\u{0441}";
    assert!(contains_ci(upper, lower));
    assert!(contains_ci(lower, upper));
    assert!(!contains_ci(lower, "other"));
}
#[test]
fn pid_contains_edges() {
    assert!(pid_contains(0, "0"));
    assert!(pid_contains(99999, "999"));
    assert!(!pid_contains(12, "123"));
    assert!(!pid_contains(100, "9"));
    assert!(!pid_contains(99999, ""));
    assert!(!pid_contains(1234, "12a"));
}
