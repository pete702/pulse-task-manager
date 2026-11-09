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
#[test]
fn item_clamps_nonfinite_cpu() {
    for v in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, -5.0] {
        let item = ProcessCpuItem::new(1, Arc::from("a.exe"), v, 1024, None);
        assert!(item.cpu_pct >= 0.0 && item.cpu_pct <= 100.0, "v={v}");
    }
    let nan = ProcessCpuItem::new(1, Arc::from("a.exe"), f32::NAN, 1024, None);
    assert_eq!(nan.cpu_pct, 0.0);
    assert_eq!(nan.cpu_text.as_ref(), "0.0%");
    let neg = ProcessCpuItem::new(1, Arc::from("a.exe"), -5.0, 1024, None);
    assert_eq!(neg.cpu_pct, 0.0);
    let raw = RawProcessCpuRow {
        pid: 2,
        name: Arc::from("b.exe"),
        cpu_usage: f32::INFINITY,
        ram_bytes: 2048,
        disk_read_mb_s: 0.0,
        disk_write_mb_s: 0.0,
        eco_qos: None,
        exe_path: None,
        drive_letter: None,
        creation_time: None,
        start_time: 0,
    };
    let item = ProcessCpuItem::from_raw(&raw);
    assert_eq!(item.cpu_pct, 0.0);
    assert_eq!(item.cpu_text.as_ref(), "0.0%");
    let raw2 = RawProcessCpuRow { cpu_usage: 150.0, ..raw.clone() };
    let item2 = ProcessCpuItem::from_raw(&raw2);
    assert_eq!(item2.cpu_pct, 100.0);
    assert_eq!(item2.cpu_text.as_ref(), "100.0%");
}
