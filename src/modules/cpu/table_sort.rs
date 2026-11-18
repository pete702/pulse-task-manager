//! Sorting, grouping, and row construction for the CPU process table.

use std::collections::HashMap;

use crate::modules::cpu::collector::pid_contains;
use crate::modules::cpu::model::{ProcessCpuItem, RawProcessCpuRow};
use crate::modules::cpu::table_rows;
use crate::modules::cpu::table_state::CpuSortCol;

pub use crate::util::contains_ci;

/// Builds one display item from a raw row plus its RAM delta.
pub fn item_from_raw_cpu(row: &RawProcessCpuRow, delta: i64) -> ProcessCpuItem {
    let mut item = ProcessCpuItem::from_raw(row);
    item.ram_delta_10s = delta;
    item.update_formatted();
    item
}

/// Quantized CPU percentage comparison to tenths.
pub fn cmp_cpu_pct(a: f32, b: f32) -> std::cmp::Ordering {
    let _ = crate::modules::cpu::table_rows::cmp_cpu_pct;
    let aa = if a.is_finite() { a } else { 0.0 };
    let bb = if b.is_finite() { b } else { 0.0 };
    ((aa * 10.0).round() as i32).cmp(&((bb * 10.0).round() as i32))
}

/// Filters rows, merges clone groups, then sorts the result.
/// `query_lower` is pre-lowercased once by the caller.
#[allow(clippy::too_many_arguments)]
pub fn filter_group_and_sort(
    processes: &[RawProcessCpuRow],
    deltas: &HashMap<(u32, u64), i64>,
    query_lower: &str,
    quick_filter: crate::modules::cpu::table_state::ProcessQuickFilter,
    group_clones: bool,
    sort_col: CpuSortCol,
    sort_asc: bool,
    eco_overrides: &HashMap<(u32, u64), bool>,
) -> Vec<ProcessCpuItem> {
    let mut items: Vec<ProcessCpuItem> = processes
        .iter()
        .filter(|row| {
            let effective_eco =
                eco_overrides.get(&(row.pid, row.start_time)).copied().or(row.eco_qos);
            let io = row.disk_read_mb_s + row.disk_write_mb_s;
            let quick_ok = quick_filter.matches(row.cpu_usage, row.ram_bytes, io, effective_eco);
            if !quick_ok {
                return false;
            }
            query_lower.is_empty()
                || contains_ci(&row.name, query_lower)
                || pid_contains(row.pid, query_lower)
        })
        .map(|row| {
            let delta = deltas.get(&(row.pid, row.start_time)).copied().unwrap_or(0);
            let mut item = item_from_raw_cpu(row, delta);
            let effective_eco =
                eco_overrides.get(&(row.pid, row.start_time)).copied().or(row.eco_qos);
            item.eco_qos = effective_eco;
            item
        })
        .collect();
    if group_clones {
        items = crate::modules::cpu::table_groups::aggregate_grouped_items(items);
    }
    sort_slice(&mut items, sort_col, sort_asc);
    items
}

/// Sorts display rows by the selected column. CPU uses quantized tenths
/// so sub-0.1% jitter never shuffles; ties break by name then pid.
pub fn sort_slice(items: &mut [ProcessCpuItem], col: CpuSortCol, asc: bool) {
    items.sort_by(|a, b| {
        let ord = match col {
            CpuSortCol::Name => a
                .name
                .as_bytes()
                .iter()
                .map(u8::to_ascii_lowercase)
                .cmp(b.name.as_bytes().iter().map(u8::to_ascii_lowercase)),
            CpuSortCol::Pid => a.pid.cmp(&b.pid),
            CpuSortCol::RamBytes => a.ram_bytes.cmp(&b.ram_bytes),
            CpuSortCol::CloneCount => a.clone_count.cmp(&b.clone_count),
            CpuSortCol::Delta => a.ram_delta_10s.cmp(&b.ram_delta_10s),
            CpuSortCol::CpuPct => cmp_cpu_pct(a.cpu_pct, b.cpu_pct),
            CpuSortCol::DiskIo => table_rows::cmp_disk_io(a, b),
        };
        let directed = if asc { ord } else { ord.reverse() };
        if directed != std::cmp::Ordering::Equal {
            return directed;
        }
        a.name
            .to_ascii_lowercase()
            .cmp(&b.name.to_ascii_lowercase())
            .then_with(|| a.pid.cmp(&b.pid))
    });
}

#[cfg(test)]
mod table_sort_tests {
    use super::*;
    use std::sync::Arc;

    fn raw(pid: u32, name: &str, read: f32, write: f32) -> RawProcessCpuRow {
        RawProcessCpuRow {
            pid,
            name: Arc::from(name),
            cpu_usage: 0.0,
            ram_bytes: 10,
            disk_read_mb_s: read,
            disk_write_mb_s: write,
            eco_qos: None,
            exe_path: None,
            drive_letter: None,
            creation_time: None,
            start_time: 1,
        }
    }

    #[test]
    fn disk_io_sort_sum_and_group_sums_io() {
        let rows =
            [raw(1, "a.exe", 10.0, 5.0), raw(2, "a.exe", 1.0, 2.0), raw(3, "b.exe", 10.0, 10.0)];
        let deltas = HashMap::new();
        let eco: HashMap<(u32, u64), bool> = HashMap::new();
        let grouped = filter_group_and_sort(
            &rows,
            &deltas,
            "",
            crate::modules::cpu::table_state::ProcessQuickFilter::All,
            true,
            CpuSortCol::DiskIo,
            false,
            &eco,
        );
        assert_eq!(grouped.len(), 2);
        assert_eq!(grouped[0].name.as_ref(), "b.exe");
        assert_eq!(grouped[1].disk_read_mb_s + grouped[1].disk_write_mb_s, 18.0);
        let flat = filter_group_and_sort(
            &rows,
            &deltas,
            "",
            crate::modules::cpu::table_state::ProcessQuickFilter::All,
            false,
            CpuSortCol::DiskIo,
            true,
            &eco,
        );
        assert_eq!(flat[0].pid, 2);
    }

    #[test]
    fn cmp_cpu_pct_quantizes_tenths() {
        assert_eq!(cmp_cpu_pct(12.31, 12.34), std::cmp::Ordering::Equal);
        assert_eq!(cmp_cpu_pct(12.3, 12.4), std::cmp::Ordering::Less);
    }

    #[test]
    fn contains_ci_and_pid_zero_alloc_match() {
        assert!(crate::util::contains_ci("Chrome.exe", "chrome"));
        assert!(crate::util::contains_ci("chrome.exe", "CHROM"));
        assert!(!crate::util::contains_ci("chrome.exe", "edge"));
        assert!(crate::util::contains_ci("a", ""));
        assert!(pid_contains(12345, "234"));
        assert!(!pid_contains(12345, "999"));
        let rows = [raw(12345, "Chrome.exe", 0.0, 0.0), raw(1, "notepad.exe", 0.0, 0.0)];
        let deltas = HashMap::new();
        let eco: HashMap<(u32, u64), bool> = HashMap::new();
        let hit = filter_group_and_sort(
            &rows,
            &deltas,
            "chrome",
            crate::modules::cpu::table_state::ProcessQuickFilter::All,
            false,
            CpuSortCol::Name,
            true,
            &eco,
        );
        assert_eq!(hit.len(), 1);
        let hit_pid = filter_group_and_sort(
            &rows,
            &deltas,
            "234",
            crate::modules::cpu::table_state::ProcessQuickFilter::All,
            false,
            CpuSortCol::Name,
            true,
            &eco,
        );
        assert_eq!(hit_pid.len(), 1);
    }
}
