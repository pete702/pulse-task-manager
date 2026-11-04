//! Unit tests for process memory model.

#[cfg(test)]
mod tests {
    use crate::modules::memory::model::RawProcessRow;
    use crate::modules::memory::process_model::{
        detect_memory_spike, filter_group_and_sort, MemorySortCol, ProcessMemoryItem,
    };

    fn raw(
        pid: u32,
        name: &str,
        ram: u64,
        virt: u64,
        delta: i64,
        exe: Option<&str>,
    ) -> RawProcessRow {
        RawProcessRow {
            pid,
            name: name.to_owned(),
            ram_bytes: ram,
            virtual_bytes: virt,
            delta_bytes_per_sec: delta,
            exe_path: exe.map(str::to_owned),
            creation_time: None,
        }
    }

    #[test]
    fn test_filter_group_and_sort_delta_aggregation() {
        let total_ram = 16_000_000_000u64;
        let processes = vec![
            raw(100, "chrome.exe", 1_000_000_000, 2_000_000_000, 5_000_000, None),
            raw(101, "chrome.exe", 2_000_000_000, 3_000_000_000, -3_000_000, None),
            raw(200, "game.exe", 3_000_000_000, 4_000_000_000, 10_000_000, None),
        ];

        let result =
            filter_group_and_sort(&processes, total_ram, "", true, MemorySortCol::RamBytes, false);

        let chrome = result.iter().find(|p| p.name == "chrome.exe").unwrap();
        assert_eq!(chrome.process_count, 2);
        assert_eq!(chrome.delta_bytes_per_sec, 2_000_000);

        let game = result.iter().find(|p| p.name == "game.exe").unwrap();
        assert_eq!(game.process_count, 1);
        assert_eq!(game.delta_bytes_per_sec, 10_000_000);
    }

    #[test]
    fn test_grouping_preserves_first_available_exe_path() {
        let processes = vec![
            raw(100, "chrome.exe", 1_000_000_000, 2_000_000_000, 0, None),
            raw(
                101,
                "chrome.exe",
                1_000_000_000,
                2_000_000_000,
                0,
                Some(r"C:\Program Files\Chrome\chrome.exe"),
            ),
        ];
        let items =
            filter_group_and_sort(&processes, 16_000_000_000, "", true, MemorySortCol::Name, true);
        let chrome = items.iter().find(|p| p.name == "chrome.exe").unwrap();
        assert_eq!(chrome.process_count, 2);
        assert_eq!(chrome.exe_path.as_deref(), Some(r"C:\Program Files\Chrome\chrome.exe"));
    }

    #[test]
    fn test_ungrouped_items_keep_their_exe_path() {
        let processes = vec![
            raw(100, "chrome.exe", 1_000_000_000, 2_000_000_000, 0, Some("a")),
            raw(200, "game.exe", 3_000_000_000, 4_000_000_000, 0, Some("b")),
        ];
        let items =
            filter_group_and_sort(&processes, 16_000_000_000, "", false, MemorySortCol::Name, true);
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].exe_path.as_deref(), Some("a"));
        assert_eq!(items[1].exe_path.as_deref(), Some("b"));
    }

    #[test]
    fn test_process_memory_item_new_with_delta() {
        let item = ProcessMemoryItem::new(
            100,
            "test.exe".to_owned(),
            1_000_000_000,
            2_000_000_000,
            16_000_000_000,
        );
        assert_eq!(item.pid, 100);
        assert_eq!(item.name, "test.exe");
        assert_eq!(item.ram_bytes, 1_000_000_000);
        assert_eq!(item.virtual_bytes, 2_000_000_000);
        assert_eq!(item.process_count, 1);
        assert_eq!(item.delta_bytes_per_sec, 0);
        assert!(!item.is_hog);
        assert!(item.creation_time.is_none());
    }

    #[test]
    fn test_detect_memory_spike_above_threshold() {
        let total_ram = 16_000_000_000u64;
        let processes = vec![
            raw(100, "chrome.exe", 1_000_000_000, 2_000_000_000, 0, None),
            raw(200, "game.exe", 14_000_000_000, 16_000_000_000, 0, None),
        ];

        let spike = detect_memory_spike(&processes, total_ram, 87.5);
        assert!(spike.is_some());
        let spike = spike.unwrap();
        assert_eq!(spike.culprit_name, "game.exe");
        assert_eq!(spike.culprit_pid, 200);
        assert_eq!(spike.culprit_bytes, 14_000_000_000);
        assert!(!spike.timestamp.is_empty());
    }

    #[test]
    fn test_detect_memory_spike_below_threshold() {
        let total_ram = 16_000_000_000u64;
        let processes = vec![
            raw(100, "chrome.exe", 1_000_000_000, 2_000_000_000, 0, None),
            raw(200, "game.exe", 10_000_000_000, 12_000_000_000, 0, None),
        ];

        let spike = detect_memory_spike(&processes, total_ram, 62.5);
        assert!(spike.is_none());
    }
}
