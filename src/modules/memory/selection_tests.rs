//! Unit tests for raw-process selection resolution and protected-process gating.

#[cfg(test)]
mod tests {
    use crate::modules::memory::model::{MemorySnapshot, RawProcessRow};
    use crate::modules::memory::process_model::ProcessMemoryItem;
    use crate::modules::memory::selection::{base_group_key, raw_targets, resolve_selection};
    use crate::modules::memory::table_state::MemoryTableState;
    use std::sync::Arc;

    fn raw(pid: u32, name: &str, path: Option<&str>) -> RawProcessRow {
        RawProcessRow {
            pid,
            name: name.to_owned(),
            ram_bytes: 1_000_000,
            virtual_bytes: 2_000_000,
            delta_bytes_per_sec: 0,
            exe_path: path.map(str::to_owned),
            creation_time: Some(pid as u64),
        }
    }

    fn snapshot(rows: Vec<RawProcessRow>) -> MemorySnapshot {
        MemorySnapshot {
            ram_total_bytes: 16_000_000_000,
            processes: Arc::new(rows),
            ..Default::default()
        }
    }

    fn item(pid: u32, name: &str) -> ProcessMemoryItem {
        ProcessMemoryItem { pid, name: name.to_owned(), process_count: 1, ..Default::default() }
    }

    #[test]
    fn test_base_group_key_strips_clone_markers() {
        assert_eq!(base_group_key("chrome.exe [60x]"), "chrome.exe");
        assert_eq!(base_group_key("chrome.exe (3x)"), "chrome.exe");
        assert_eq!(base_group_key(" chrome.exe "), "chrome.exe");
        assert_eq!(base_group_key("chrome.exe"), "chrome.exe");
        assert_eq!(base_group_key("Tag[v2]"), "Tag[v2]");
    }

    #[test]
    fn test_selection_targets_all_raw_group_pids() {
        let mem = snapshot(vec![
            raw(10, "chrome.exe", Some(r"C:\chrome.exe")),
            raw(11, "chrome.exe", None),
            raw(12, "chrome.exe", None),
            raw(20, "game.exe", None),
        ]);
        let view = resolve_selection(&mem, Some("chrome.exe [60x]")).expect("selected");
        assert_eq!(view.name, "chrome.exe");
        assert_eq!(view.pids, vec![10, 11, 12]);
        assert_eq!(view.targets.len(), 3);
        assert_eq!(view.path.as_deref(), Some(r"C:\chrome.exe"));
        assert!(!view.blocked);
        assert!(view.can_modify());
    }

    #[test]
    fn test_raw_targets_ignore_table_search_and_grouping() {
        let mem = snapshot(vec![
            raw(1, "chrome.exe", None),
            raw(2, "notepad.exe", None),
            raw(3, "game.exe", None),
        ]);
        assert_eq!(raw_targets(&mem).len(), 3);
        let view = resolve_selection(&mem, Some("game.exe")).expect("selected");
        assert_eq!(view.pids, vec![3]);
    }

    #[test]
    fn test_protected_process_is_gated() {
        let mem = snapshot(vec![raw(4, "System", None), raw(900, "svchost.exe", None)]);
        let view = resolve_selection(&mem, Some("svchost.exe")).expect("selected");
        assert!(view.blocked);
        assert!(!view.can_modify());
    }

    #[test]
    fn test_missing_or_empty_selection_returns_none() {
        let mem = snapshot(vec![raw(10, "chrome.exe", None)]);
        assert!(resolve_selection(&mem, Some("absent.exe")).is_none());
        assert!(resolve_selection(&mem, Some("   ")).is_none());
        assert!(resolve_selection(&mem, None).is_none());
    }

    #[test]
    fn test_toggle_selection_keeps_clean_name() {
        let mut state = MemoryTableState::default();
        let chrome = item(42, "chrome.exe");
        state.toggle_selection(&chrome);
        assert_eq!(state.selected_group.as_deref(), Some("chrome.exe"));
        assert_eq!(state.selected_pid, Some(42));
        state.toggle_selection(&chrome);
        assert!(state.selected_group.is_none());
        assert!(state.selected_pid.is_none());
    }

    #[test]
    fn test_frozen_kill_confirm_excludes_newly_spawned_process() {
        // Initial snapshot with chrome.exe at PID 10
        let mem1 = snapshot(vec![raw(10, "chrome.exe", None)]);
        let view1 = resolve_selection(&mem1, Some("chrome.exe")).expect("selected");

        // Freeze the targets (simulating first [End Task] click)
        let frozen_targets = view1.targets.clone();
        assert_eq!(frozen_targets.len(), 1);
        assert_eq!(frozen_targets[0].pid, 10);

        // New snapshot: chrome.exe at PID 10 AND a new chrome.exe at PID 11 spawns
        let mem2 = snapshot(vec![raw(10, "chrome.exe", None), raw(11, "chrome.exe", None)]);
        let view2 = resolve_selection(&mem2, Some("chrome.exe")).expect("selected");

        // Current resolution sees 2 PIDs
        assert_eq!(view2.pids, vec![10, 11]);
        assert_eq!(view2.targets.len(), 2);

        // But frozen targets from first click still only has PID 10
        assert_eq!(frozen_targets.len(), 1);
        assert_eq!(frozen_targets[0].pid, 10);
        assert!(!frozen_targets.iter().any(|t| t.pid == 11));
    }
}
