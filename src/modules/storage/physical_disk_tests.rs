//! Physical disk helper tests.

use super::*;

#[test]
fn drive_letter_extraction() {
    assert_eq!(drive_letter("C:\\"), Some('C'));
    assert_eq!(drive_letter("D:"), Some('D'));
    assert_eq!(drive_letter("h:\\"), Some('H'));
    assert_eq!(drive_letter("X:"), Some('X'));
    assert_eq!(drive_letter(""), None);
    assert_eq!(drive_letter("\\\\Host\\Share"), None);
    assert_eq!(drive_letter("1:\\"), None);
}

#[test]
fn fallback_handling_empty() {
    assert_eq!(physical_disk_for_mount(""), None);
    assert_eq!(perf_counters_for_mount(""), None);
    assert_eq!(perf_counters_for_mount("invalid"), None);
    assert_eq!(sum_perf_counters(&[]), None);
}
