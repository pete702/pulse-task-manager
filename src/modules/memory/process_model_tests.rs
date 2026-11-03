//! Unit tests for process memory model.

#[cfg(test)]
mod tests {
    use crate::modules::memory::process_model::{
        detect_memory_spike, filter_group_and_sort, MemorySortCol, ProcessMemoryItem,
    };

    #[test]
    fn test_filter_group_and_sort_delta_aggregation() {
        let total_ram = 16_000_000_000u64;
        let processes = vec![
            (
                100,
                "chrome.exe".to_owned(),
                1_000_000_000,
                2_000_000_000,
                5_000_000,
            ), // +5 MB/s
            (
                101,
                "chrome.exe".to_owned(),
                2_000_000_000,
                3_000_000_000,
                -3_000_000,
            ), // -3 MB/s
            (
                200,
                "game.exe".to_owned(),
                3_000_000_000,
                4_000_000_000,
                10_000_000,
            ), // +10 MB/s
        ];

        let result = filter_group_and_sort(
            &processes,
            total_ram,
            "",
            true,
            MemorySortCol::RamBytes,
            false,
        );

        // chrome.exe should be grouped with delta = +5 - 3 = +2 MB/s
        let chrome = result.iter().find(|p| p.name == "chrome.exe").unwrap();
        assert_eq!(chrome.process_count, 2);
        assert_eq!(chrome.delta_bytes_per_sec, 2_000_000);

        // game.exe should have delta = +10 MB/s
        let game = result.iter().find(|p| p.name == "game.exe").unwrap();
        assert_eq!(game.process_count, 1);
        assert_eq!(game.delta_bytes_per_sec, 10_000_000);
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
        assert!(!item.is_hog); // 1GB / 16GB = 6.25% < 15%
    }

    #[test]
    fn test_detect_memory_spike_above_threshold() {
        let total_ram = 16_000_000_000u64;
        let processes = vec![
            (
                100,
                "chrome.exe".to_owned(),
                1_000_000_000,
                2_000_000_000,
                0,
            ),
            (
                200,
                "game.exe".to_owned(),
                14_000_000_000,
                16_000_000_000,
                0,
            ), // 87.5% > 85%
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
            (
                100,
                "chrome.exe".to_owned(),
                1_000_000_000,
                2_000_000_000,
                0,
            ),
            (
                200,
                "game.exe".to_owned(),
                10_000_000_000,
                12_000_000_000,
                0,
            ), // 62.5% < 85%
        ];

        let spike = detect_memory_spike(&processes, total_ram, 62.5);
        assert!(spike.is_none());
    }
}
