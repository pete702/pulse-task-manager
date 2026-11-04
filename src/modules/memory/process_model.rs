//! Pure mathematical model for process memory telemetry — zero dependencies on egui/sysinfo.
//! Provides process memory aggregation, grouping, sorting, and spike detection.

use std::collections::HashMap;
use std::sync::Arc;

use crate::modules::memory::model::RawProcessRow;
use crate::modules::memory::process_item::ProcessMemoryItem;

/// Column for sorting process list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[allow(dead_code)]
pub enum MemorySortCol {
    #[default]
    Name,
    Pid,
    RamBytes,
    VirtualBytes,
    RamPct,
}

/// Filters, groups (by name), and sorts process memory items.
#[allow(dead_code)]
pub fn filter_group_and_sort(
    processes: &[RawProcessRow],
    total_ram: u64,
    search: &str,
    group_clones: bool,
    sort_col: MemorySortCol,
    sort_asc: bool,
) -> Vec<ProcessMemoryItem> {
    let search_lower = search.to_lowercase();
    let mut items: Vec<ProcessMemoryItem> = processes
        .iter()
        .filter(|row| {
            search.is_empty()
                || search_lower == row.pid.to_string()
                || row.name.to_lowercase().contains(&search_lower)
        })
        .map(|row| item_from_raw(row, total_ram))
        .collect();
    if group_clones {
        let mut grouped: HashMap<Arc<str>, ProcessMemoryItem> = HashMap::new();
        for item in items {
            grouped
                .entry(Arc::clone(&item.name))
                .and_modify(|ex| {
                    ex.ram_bytes = ex.ram_bytes.saturating_add(item.ram_bytes);
                    ex.virtual_bytes = ex.virtual_bytes.saturating_add(item.virtual_bytes);
                    ex.delta_bytes_per_sec =
                        ex.delta_bytes_per_sec.saturating_add(item.delta_bytes_per_sec);
                    ex.process_count += 1;
                    ex.update_formatted(total_ram);
                    if ex.exe_path.is_none() {
                        ex.exe_path = item.exe_path.clone();
                    }
                    if item.pid < ex.pid {
                        ex.pid = item.pid;
                    }
                })
                .or_insert(item);
        }
        items = grouped.into_values().collect();
    }
    items.sort_by(|a, b| {
        let ord = match sort_col {
            MemorySortCol::Name => a.name.cmp(&b.name),
            MemorySortCol::Pid => a.pid.cmp(&b.pid),
            MemorySortCol::RamBytes => a.ram_bytes.cmp(&b.ram_bytes),
            MemorySortCol::VirtualBytes => a.virtual_bytes.cmp(&b.virtual_bytes),
            MemorySortCol::RamPct => {
                a.ram_pct.partial_cmp(&b.ram_pct).unwrap_or(std::cmp::Ordering::Equal)
            }
        };
        if sort_asc {
            ord
        } else {
            ord.reverse()
        }
    });
    items
}

/// Re-export for backward compatibility
pub use crate::modules::memory::process_item::item_from_raw;
pub use crate::modules::memory::process_spike::detect_memory_spike;
// ProcessMemoryItem and MemorySpikeEvent are already public via their modules
// Use `crate::modules::memory::process_item::ProcessMemoryItem` and `crate::modules::memory::process_spike::MemorySpikeEvent` directly
