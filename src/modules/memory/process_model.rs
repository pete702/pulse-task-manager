//! Pure mathematical model for process memory telemetry — zero dependencies on egui/sysinfo.
//!
//! Provides process memory aggregation, grouping, sorting, and spike detection.

use std::collections::HashMap;

/// A single process memory entry for display and analysis.
#[derive(Clone, Debug, Default, PartialEq)]
#[allow(dead_code)]
pub struct ProcessMemoryItem {
    /// Process ID.
    pub pid: u32,
    /// Process name (executable name).
    pub name: String,
    /// Physical memory (RAM) usage in bytes.
    pub ram_bytes: u64,
    /// Virtual memory usage in bytes.
    pub virtual_bytes: u64,
    /// RAM usage as percentage of total system RAM.
    pub ram_pct: f32,
    /// Number of processes grouped under this entry (for clone grouping).
    pub process_count: usize,
    /// True if this process is a memory hog (RAM% >= 15%).
    pub is_hog: bool,
}

/// Column for sorting process list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[allow(dead_code)]
pub enum MemorySortCol {
    /// Sort by process name.
    #[default]
    Name,
    /// Sort by PID.
    Pid,
    /// Sort by RAM usage (bytes).
    RamBytes,
    /// Sort by virtual memory usage (bytes).
    VirtualBytes,
    /// Sort by RAM percentage.
    RamPct,
}

/// Memory spike event for "black box" logging.
#[derive(Clone, Debug, Default, PartialEq)]
#[allow(dead_code)]
pub struct MemorySpikeEvent {
    /// Timestamp of the spike (ISO 8601 format).
    pub timestamp: String,
    /// Peak RAM usage percentage at spike time.
    pub peak_ram_pct: f32,
    /// Name of the process causing the spike.
    pub culprit_name: String,
    /// PID of the process causing the spike.
    pub culprit_pid: u32,
    /// RAM bytes of the culprit process at spike time.
    pub culprit_bytes: u64,
}

impl ProcessMemoryItem {
    /// Creates a new process memory item.
    #[allow(dead_code)]
    pub fn new(pid: u32, name: String, ram_bytes: u64, virtual_bytes: u64, total_ram: u64) -> Self {
        let ram_pct = if total_ram > 0 {
            (ram_bytes as f64 / total_ram as f64 * 100.0) as f32
        } else {
            0.0
        };
        Self {
            pid,
            name,
            ram_bytes,
            virtual_bytes,
            ram_pct,
            process_count: 1,
            is_hog: ram_pct >= 15.0,
        }
    }
}

/// Filters, groups (by name), and sorts process memory items.
///
/// - `search`: case-insensitive substring match on name or exact PID match
/// - `group_clones`: if true, aggregates processes with same name (sums RAM, counts instances)
/// - `sort_col`: column to sort by
/// - `sort_asc`: ascending (true) or descending (false) order
#[allow(dead_code)]
pub fn filter_group_and_sort(
    processes: &[(u32, String, u64, u64)],
    total_ram: u64,
    search: &str,
    group_clones: bool,
    sort_col: MemorySortCol,
    sort_asc: bool,
) -> Vec<ProcessMemoryItem> {
    let search_lower = search.to_lowercase();

    // Step 1: Filter and create initial items
    let mut items: Vec<ProcessMemoryItem> = processes
        .iter()
        .filter(|(pid, name, _, _)| {
            search.is_empty()
                || search_lower == pid.to_string()
                || name.to_lowercase().contains(&search_lower)
        })
        .map(|(pid, name, ram_bytes, virtual_bytes)| {
            ProcessMemoryItem::new(*pid, name.clone(), *ram_bytes, *virtual_bytes, total_ram)
        })
        .collect();

    // Step 2: Group clones (same process name) if enabled
    if group_clones {
        let mut grouped: HashMap<String, ProcessMemoryItem> = HashMap::new();

        for item in items {
            grouped
                .entry(item.name.clone())
                .and_modify(|existing| {
                    existing.ram_bytes = existing.ram_bytes.saturating_add(item.ram_bytes);
                    existing.virtual_bytes =
                        existing.virtual_bytes.saturating_add(item.virtual_bytes);
                    existing.process_count += 1;
                    // Recalculate percentage
                    existing.ram_pct = if total_ram > 0 {
                        (existing.ram_bytes as f64 / total_ram as f64 * 100.0) as f32
                    } else {
                        0.0
                    };
                    existing.is_hog = existing.ram_pct >= 15.0;
                    // Keep the lowest PID as representative
                    if item.pid < existing.pid {
                        existing.pid = item.pid;
                    }
                })
                .or_insert(item);
        }

        items = grouped.into_values().collect();
    }

    // Step 3: Sort
    items.sort_by(|a, b| {
        let ord = match sort_col {
            MemorySortCol::Name => a.name.cmp(&b.name),
            MemorySortCol::Pid => a.pid.cmp(&b.pid),
            MemorySortCol::RamBytes => a.ram_bytes.cmp(&b.ram_bytes),
            MemorySortCol::VirtualBytes => a.virtual_bytes.cmp(&b.virtual_bytes),
            MemorySortCol::RamPct => a
                .ram_pct
                .partial_cmp(&b.ram_pct)
                .unwrap_or(std::cmp::Ordering::Equal),
        };
        if sort_asc {
            ord
        } else {
            ord.reverse()
        }
    });

    items
}

/// Detects memory spike events (RAM usage >= 85%).
///
/// Returns a `MemorySpikeEvent` if a spike is detected, with the top memory consumer
/// identified as the culprit. Returns `None` if no spike.
#[allow(dead_code)]
pub fn detect_memory_spike(
    processes: &[(u32, String, u64, u64)],
    total_ram: u64,
    current_ram_pct: f32,
) -> Option<MemorySpikeEvent> {
    if current_ram_pct < 85.0 || total_ram == 0 {
        return None;
    }

    // Find the process using the most RAM
    let culprit = processes
        .iter()
        .max_by_key(|(_, _, ram_bytes, _)| *ram_bytes);

    if let Some((pid, name, ram_bytes, _)) = culprit {
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        Some(MemorySpikeEvent {
            timestamp,
            peak_ram_pct: current_ram_pct,
            culprit_name: name.clone(),
            culprit_pid: *pid,
            culprit_bytes: *ram_bytes,
        })
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_item_creation() {
        let item = ProcessMemoryItem::new(
            1234,
            "test.exe".to_string(),
            1_000_000_000,
            2_000_000_000,
            16_000_000_000,
        );
        assert_eq!(item.pid, 1234);
        assert_eq!(item.name, "test.exe");
        assert_eq!(item.ram_bytes, 1_000_000_000);
        assert_eq!(item.virtual_bytes, 2_000_000_000);
        assert!((item.ram_pct - 6.25).abs() < 0.01); // 1GB / 16GB = 6.25%
        assert_eq!(item.process_count, 1);
        assert!(!item.is_hog); // < 15%
    }

    #[test]
    fn test_process_item_is_hog() {
        let item = ProcessMemoryItem::new(
            1234,
            "hog.exe".to_string(),
            3_000_000_000,
            4_000_000_000,
            16_000_000_000,
        );
        assert!(item.is_hog); // 3GB / 16GB = 18.75% >= 15%
    }

    #[test]
    fn test_group_clones() {
        let processes = vec![
            (100, "chrome.exe".to_string(), 500_000_000, 1_000_000_000),
            (101, "chrome.exe".to_string(), 300_000_000, 800_000_000),
            (102, "firefox.exe".to_string(), 800_000_000, 1_500_000_000),
        ];
        let total_ram = 16_000_000_000;

        let result = filter_group_and_sort(
            &processes,
            total_ram,
            "",
            true,
            MemorySortCol::RamBytes,
            false,
        );

        // Should have 2 groups: chrome (2 processes) and firefox (1 process)
        assert_eq!(result.len(), 2);

        // Chrome group should have combined RAM
        let chrome = result.iter().find(|p| p.name == "chrome.exe").unwrap();
        assert_eq!(chrome.process_count, 2);
        assert_eq!(chrome.ram_bytes, 800_000_000);
        assert!((chrome.ram_pct - 5.0).abs() < 0.01); // 800MB / 16GB = 5%

        // Firefox should be unchanged
        let firefox = result.iter().find(|p| p.name == "firefox.exe").unwrap();
        assert_eq!(firefox.process_count, 1);
        assert_eq!(firefox.ram_bytes, 800_000_000);
    }

    #[test]
    fn test_search_filter_by_name() {
        let processes = vec![
            (100, "chrome.exe".to_string(), 500_000_000, 1_000_000_000),
            (101, "firefox.exe".to_string(), 800_000_000, 1_500_000_000),
            (102, "notepad.exe".to_string(), 50_000_000, 100_000_000),
        ];
        let total_ram = 16_000_000_000;

        // Search for "chrome"
        let result = filter_group_and_sort(
            &processes,
            total_ram,
            "chrome",
            false,
            MemorySortCol::RamBytes,
            false,
        );
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "chrome.exe");

        // Search for "fox" (substring)
        let result = filter_group_and_sort(
            &processes,
            total_ram,
            "fox",
            false,
            MemorySortCol::RamBytes,
            false,
        );
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "firefox.exe");
    }

    #[test]
    fn test_search_filter_by_pid() {
        let processes = vec![
            (100, "chrome.exe".to_string(), 500_000_000, 1_000_000_000),
            (101, "firefox.exe".to_string(), 800_000_000, 1_500_000_000),
        ];
        let total_ram = 16_000_000_000;

        // Search by exact PID
        let result = filter_group_and_sort(
            &processes,
            total_ram,
            "101",
            false,
            MemorySortCol::RamBytes,
            false,
        );
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].pid, 101);
        assert_eq!(result[0].name, "firefox.exe");
    }

    #[test]
    fn test_sort_by_ram_pct_desc() {
        let processes = vec![
            (100, "small.exe".to_string(), 100_000_000, 200_000_000),
            (101, "large.exe".to_string(), 1_000_000_000, 2_000_000_000),
            (102, "medium.exe".to_string(), 500_000_000, 1_000_000_000),
        ];
        let total_ram = 16_000_000_000;

        let result = filter_group_and_sort(
            &processes,
            total_ram,
            "",
            false,
            MemorySortCol::RamPct,
            false,
        );

        // Should be sorted by RAM% descending: large, medium, small
        assert_eq!(result[0].name, "large.exe");
        assert_eq!(result[1].name, "medium.exe");
        assert_eq!(result[2].name, "small.exe");
    }

    #[test]
    fn test_detect_memory_spike_no_spike() {
        let processes = vec![(100, "chrome.exe".to_string(), 500_000_000, 1_000_000_000)];
        let total_ram = 16_000_000_000;

        // 50% RAM - no spike
        let result = detect_memory_spike(&processes, total_ram, 50.0);
        assert!(result.is_none());
    }

    #[test]
    fn test_detect_memory_spike_detected() {
        let processes = vec![
            (100, "chrome.exe".to_string(), 500_000_000, 1_000_000_000),
            (101, "heavy.exe".to_string(), 12_000_000_000, 14_000_000_000),
        ];
        let total_ram = 16_000_000_000;

        // 85% RAM - spike detected
        let result = detect_memory_spike(&processes, total_ram, 85.0);
        assert!(result.is_some());
        let spike = result.unwrap();
        assert_eq!(spike.culprit_name, "heavy.exe");
        assert_eq!(spike.culprit_pid, 101);
        assert_eq!(spike.culprit_bytes, 12_000_000_000);
        assert!((spike.peak_ram_pct - 85.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_sort_by_name_asc() {
        let processes = vec![
            (100, "zebra.exe".to_string(), 100_000_000, 200_000_000),
            (101, "alpha.exe".to_string(), 500_000_000, 1_000_000_000),
        ];
        let total_ram = 16_000_000_000;

        let result =
            filter_group_and_sort(&processes, total_ram, "", false, MemorySortCol::Name, true);

        assert_eq!(result[0].name, "alpha.exe");
        assert_eq!(result[1].name, "zebra.exe");
    }
}
