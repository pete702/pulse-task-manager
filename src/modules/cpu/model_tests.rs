//! Tests for CPU model.
use super::*;
#[test]
fn test_hottest_core_detection() {
    let mut snap = CpuSnapshot {
        per_core_pct: Arc::from(vec![10.0, 95.0, 45.0, 30.0].into_boxed_slice()),
        ..Default::default()
    };
    let (idx, val) =
        snap.per_core_pct.iter().enumerate().max_by(|a, b| a.1.total_cmp(b.1)).unwrap_or((0, &0.0));
    assert_eq!(idx, 1);
    assert_eq!(*val, 95.0);
    snap.hottest_core_idx = idx;
    snap.hottest_core_pct = *val;
    assert_eq!(snap.hottest_core_idx, 1);
    assert_eq!(snap.hottest_core_pct, 95.0);
}
#[test]
fn test_process_cpu_item_formatting() {
    let mut item = ProcessCpuItem::new(1234, Arc::from("test.exe"), 15.0, 1024 * 1024 * 100, None);
    assert_eq!(item.cpu_text.as_ref(), "15.0%");
    assert_eq!(item.ram_text.as_ref(), "100.00 MiB");
    assert_eq!(item.pid_text.as_ref(), "1234");
    assert_eq!(item.clone_text.as_ref(), "1x");
    assert!(!item.is_cpu_hog);
    item.update_formatted();
    assert_eq!(item.cpu_text.as_ref(), "15.0%");
    item.cpu_pct = 50.0;
    item.update_formatted();
    assert_eq!(item.cpu_text.as_ref(), "50.0%");
    assert!(item.is_cpu_hog);
    item.ram_bytes = 1024 * 1024 * 200;
    item.update_formatted();
    assert_eq!(item.ram_text.as_ref(), "200.00 MiB");
    item.clone_count = 5;
    item.update_formatted();
    assert_eq!(item.clone_text.as_ref(), "[5]x");
}
