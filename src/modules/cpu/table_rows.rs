//! CPU table row comparison helpers.

use crate::modules::cpu::model::ProcessCpuItem;

pub fn cmp_cpu_pct(a: &ProcessCpuItem, b: &ProcessCpuItem) -> std::cmp::Ordering {
    let aa = if a.cpu_pct.is_finite() { a.cpu_pct } else { 0.0 };
    let bb = if b.cpu_pct.is_finite() { b.cpu_pct } else { 0.0 };
    aa.total_cmp(&bb)
}

pub fn cmp_disk_io(a: &ProcessCpuItem, b: &ProcessCpuItem) -> std::cmp::Ordering {
    let aa = if a.disk_read_mb_s.is_finite() && a.disk_write_mb_s.is_finite() {
        a.disk_read_mb_s + a.disk_write_mb_s
    } else {
        0.0
    };
    let bb = if b.disk_read_mb_s.is_finite() && b.disk_write_mb_s.is_finite() {
        b.disk_read_mb_s + b.disk_write_mb_s
    } else {
        0.0
    };
    aa.total_cmp(&bb)
}
