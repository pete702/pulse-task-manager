//! Unit tests for process memory model.

#[cfg(test)]
mod tests {
    use crate::modules::memory::process_model::{
        filter_group_and_sort, MemorySortCol, ProcessMemoryItem,
    };

    #[test]
    fn test_process_item_creation() {
        let item = ProcessMemoryItem::new(
            1234,
            "test.exe".into(),
            1_000_000_000,
            2_000_000_000,
            16_000_000_000,
        );
        assert_eq!(item.pid, 1234);
        assert_eq!(item.name, "test.exe");
        assert_eq!(item.ram_bytes, 1_000_000_000);
        assert_eq!(item.virtual_bytes, 2_000_000_000);
        assert!((item.ram_pct - 6.25).abs() < 0.01);
        assert_eq!(item.process_count, 1);
        assert!(!item.is_hog);
    }

    #[test]
    fn test_process_item_is_hog() {
        let item = ProcessMemoryItem::new(
            1234,
            "hog.exe".into(),
            3_000_000_000,
            4_000_000_000,
            16_000_000_000,
        );
        assert!(item.is_hog);
    }

    #[test]
    fn test_group_clones() {
        let procs = vec![
            (100, "chrome.exe".into(), 500_000_000, 1_000_000_000),
            (101, "chrome.exe".into(), 300_000_000, 800_000_000),
            (102, "firefox.exe".into(), 800_000_000, 1_500_000_000),
        ];
        let res = filter_group_and_sort(
            &procs,
            16_000_000_000,
            "",
            true,
            MemorySortCol::RamBytes,
            false,
        );
        assert_eq!(res.len(), 2);
        let c = res.iter().find(|p| p.name == "chrome.exe").unwrap();
        assert_eq!(c.process_count, 2);
        assert_eq!(c.ram_bytes, 800_000_000);
        assert!((c.ram_pct - 5.0).abs() < 0.01);
        let f = res.iter().find(|p| p.name == "firefox.exe").unwrap();
        assert_eq!(f.process_count, 1);
        assert_eq!(f.ram_bytes, 800_000_000);
    }

    #[test]
    fn test_search_filter_by_name() {
        let procs = vec![
            (100, "chrome.exe".into(), 500_000_000, 1_000_000_000),
            (101, "firefox.exe".into(), 800_000_000, 1_500_000_000),
            (102, "notepad.exe".into(), 50_000_000, 100_000_000),
        ];
        let ram = 16_000_000_000;
        let r = filter_group_and_sort(&procs, ram, "chrome", false, MemorySortCol::RamBytes, false);
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].name, "chrome.exe");
        let r = filter_group_and_sort(&procs, ram, "fox", false, MemorySortCol::RamBytes, false);
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].name, "firefox.exe");
    }

    #[test]
    fn test_search_filter_by_pid() {
        let procs = vec![
            (100, "chrome.exe".into(), 500_000_000, 1_000_000_000),
            (101, "firefox.exe".into(), 800_000_000, 1_500_000_000),
        ];
        let r = filter_group_and_sort(
            &procs,
            16_000_000_000,
            "101",
            false,
            MemorySortCol::RamBytes,
            false,
        );
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].pid, 101);
        assert_eq!(r[0].name, "firefox.exe");
    }

    #[test]
    fn test_sort_by_ram_pct_desc() {
        let procs = vec![
            (100, "small.exe".into(), 100_000_000, 200_000_000),
            (101, "large.exe".into(), 1_000_000_000, 2_000_000_000),
            (102, "medium.exe".into(), 500_000_000, 1_000_000_000),
        ];
        let r = filter_group_and_sort(
            &procs,
            16_000_000_000,
            "",
            false,
            MemorySortCol::RamPct,
            false,
        );
        assert_eq!(r[0].name, "large.exe");
        assert_eq!(r[1].name, "medium.exe");
        assert_eq!(r[2].name, "small.exe");
    }

    #[test]
    fn test_detect_memory_spike_no_spike() {
        let procs = vec![(100, "chrome.exe".into(), 500_000_000, 1_000_000_000)];
        let r = crate::modules::memory::process_model::detect_memory_spike(
            &procs,
            16_000_000_000,
            50.0,
        );
        assert!(r.is_none());
    }

    #[test]
    fn test_detect_memory_spike_detected() {
        let procs = vec![
            (100, "chrome.exe".into(), 500_000_000, 1_000_000_000),
            (101, "heavy.exe".into(), 12_000_000_000, 14_000_000_000),
        ];
        let r = crate::modules::memory::process_model::detect_memory_spike(
            &procs,
            16_000_000_000,
            85.0,
        );
        assert!(r.is_some());
        let s = r.unwrap();
        assert_eq!(s.culprit_name, "heavy.exe");
        assert_eq!(s.culprit_pid, 101);
        assert_eq!(s.culprit_bytes, 12_000_000_000);
        assert!((s.peak_ram_pct - 85.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_sort_by_name_asc() {
        let procs = vec![
            (100, "zebra.exe".into(), 100_000_000, 200_000_000),
            (101, "alpha.exe".into(), 500_000_000, 1_000_000_000),
        ];
        let r = filter_group_and_sort(&procs, 16_000_000_000, "", false, MemorySortCol::Name, true);
        assert_eq!(r[0].name, "alpha.exe");
        assert_eq!(r[1].name, "zebra.exe");
    }
}
