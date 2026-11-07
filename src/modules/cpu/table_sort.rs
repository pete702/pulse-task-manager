//! Sorting, grouping, and row construction for the CPU process table.

use std::collections::HashMap;
use std::sync::Arc;

use crate::modules::cpu::collector::pid_contains;
use crate::modules::cpu::model::{ProcessCpuItem, RawProcessCpuRow};
use crate::modules::cpu::table_state::CpuSortCol;

/// Case-insensitive substring check.
#[inline]
pub fn contains_ci(haystack: &str, needle: &str) -> bool {
    needle.is_empty()
        || haystack
            .as_bytes()
            .windows(needle.len())
            .any(|w| w.eq_ignore_ascii_case(needle.as_bytes()))
}

/// Builds one display item from a raw row plus its RAM delta.
pub fn item_from_raw_cpu(row: &RawProcessCpuRow, delta: i64) -> ProcessCpuItem {
    let mut item = ProcessCpuItem::new(
        row.pid,
        Arc::clone(&row.name),
        row.cpu_usage.clamp(0.0, 100.0),
        row.ram_bytes,
        row.creation_time,
    );
    item.start_time = row.start_time;
    item.drive_letter = row.drive_letter;
    item.exe_path = row.exe_path.clone().unwrap_or_default();
    item.disk_read_mb_s = row.disk_read_mb_s;
    item.disk_write_mb_s = row.disk_write_mb_s;
    item.ram_delta_10s = delta;
    item.update_formatted();
    item
}

/// Filters rows, merges clone groups, then sorts the result.
/// `query_lower` is pre-lowercased once by the caller.
pub fn filter_group_and_sort(
    processes: &[RawProcessCpuRow],
    query_lower: &str,
    quick_filter: crate::modules::cpu::table_state::ProcessQuickFilter,
    group_clones: bool,
    sort_col: CpuSortCol,
    sort_asc: bool,
    deltas: &HashMap<(u32, u64), i64>,
) -> Vec<ProcessCpuItem> {
    let mut items: Vec<ProcessCpuItem> = processes
        .iter()
        .filter(|row| {
            let io = row.disk_read_mb_s + row.disk_write_mb_s;
            let quick_ok = quick_filter.matches(row.cpu_usage, row.ram_bytes, io, row.eco_qos);
            if !quick_ok {
                return false;
            }
            query_lower.is_empty()
                || contains_ci(&row.name, query_lower)
                || pid_contains(row.pid, query_lower)
        })
        .map(|row| {
            let delta = deltas.get(&(row.pid, row.start_time)).copied().unwrap_or(0);
            item_from_raw_cpu(row, delta)
        })
        .collect();
    if group_clones {
        let mut grouped: HashMap<Arc<str>, ProcessCpuItem> = HashMap::new();
        for item in items {
            let key: Arc<str> = Arc::clone(&item.name).to_ascii_lowercase().into();
            grouped
                .entry(key)
                .and_modify(|ex| {
                    ex.cpu_pct += item.cpu_pct;
                    ex.ram_bytes = ex.ram_bytes.saturating_add(item.ram_bytes);
                    ex.disk_read_mb_s += item.disk_read_mb_s;
                    ex.disk_write_mb_s += item.disk_write_mb_s;
                    ex.ram_delta_10s = ex.ram_delta_10s.saturating_add(item.ram_delta_10s);
                    ex.clone_count += 1;
                    ex.update_formatted();
                })
                .or_insert(item);
        }
        items = grouped.into_values().collect();
        items.sort_by(|a, b| {
            a.name
                .to_ascii_lowercase()
                .cmp(&b.name.to_ascii_lowercase())
                .then_with(|| a.pid.cmp(&b.pid))
        });
    }
    sort_slice(&mut items, sort_col, sort_asc);
    items
}

/// Sorts display rows by the selected column. CPU uses quantized tenths
/// so sub-0.1% jitter never shuffles; ties break by name then pid.
pub fn sort_slice(items: &mut [ProcessCpuItem], col: CpuSortCol, asc: bool) {
    items.sort_by(|a, b| {
        let ord = match col {
            CpuSortCol::Name => a.name.cmp(&b.name),
            CpuSortCol::Pid => a.pid.cmp(&b.pid),
            CpuSortCol::RamBytes => a.ram_bytes.cmp(&b.ram_bytes),
            CpuSortCol::CloneCount => a.clone_count.cmp(&b.clone_count),
            CpuSortCol::Delta => a.ram_delta_10s.cmp(&b.ram_delta_10s),
            CpuSortCol::CpuPct => {
                let aq = (a.cpu_pct * 10.0).round() as i64;
                let bq = (b.cpu_pct * 10.0).round() as i64;
                aq.cmp(&bq)
            }
            CpuSortCol::DiskIo => (a.disk_read_mb_s + a.disk_write_mb_s)
                .total_cmp(&(b.disk_read_mb_s + b.disk_write_mb_s)),
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
        let grouped = filter_group_and_sort(
            &rows,
            "",
            crate::modules::cpu::table_state::ProcessQuickFilter::All,
            true,
            CpuSortCol::DiskIo,
            false,
            &deltas,
        );
        assert_eq!(grouped.len(), 2);
        assert_eq!(grouped[0].name.as_ref(), "b.exe");
        assert_eq!(grouped[1].disk_read_mb_s + grouped[1].disk_write_mb_s, 18.0);
        let flat = filter_group_and_sort(
            &rows,
            "",
            crate::modules::cpu::table_state::ProcessQuickFilter::All,
            false,
            CpuSortCol::DiskIo,
            true,
            &deltas,
        );
        assert_eq!(flat[0].pid, 2);
    }

    #[test]
    fn contains_ci_and_pid_zero_alloc_match() {
        assert!(contains_ci("Chrome.exe", "chrome"));
        assert!(contains_ci("chrome.exe", "CHROM"));
        assert!(!contains_ci("chrome.exe", "edge"));
        assert!(contains_ci("a", ""));
        assert!(pid_contains(12345, "234"));
        assert!(!pid_contains(12345, "999"));
        let rows = [raw(12345, "Chrome.exe", 0.0, 0.0), raw(1, "notepad.exe", 0.0, 0.0)];
        let deltas = HashMap::new();
        let hit = filter_group_and_sort(
            &rows,
            "chrome",
            crate::modules::cpu::table_state::ProcessQuickFilter::All,
            false,
            CpuSortCol::Name,
            true,
            &deltas,
        );
        assert_eq!(hit.len(), 1);
        let hit_pid = filter_group_and_sort(
            &rows,
            "234",
            crate::modules::cpu::table_state::ProcessQuickFilter::All,
            false,
            CpuSortCol::Name,
            true,
            &deltas,
        );
        assert_eq!(hit_pid.len(), 1);
    }
}
