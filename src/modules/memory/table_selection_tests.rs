//! Unit tests for mode-aware table selection (single-PID vs grouped).

#[cfg(test)]
mod tests {
    use crate::modules::memory::process_model::ProcessMemoryItem;
    use crate::modules::memory::table_state::MemoryTableState;

    fn item(pid: u32, name: &str) -> ProcessMemoryItem {
        ProcessMemoryItem { pid, name: name.to_owned(), process_count: 1, ..Default::default() }
    }

    #[test]
    fn test_toggle_selection_keeps_clean_name() {
        let mut state = MemoryTableState::default();
        let chrome = item(42, "chrome.exe");
        // Grouped mode (default): clicking a row selects the whole name group.
        state.toggle_selection(&chrome);
        assert_eq!(state.selected_group.as_deref(), Some("chrome.exe"));
        assert!(state.selected_pid.is_none());
        assert!(state.is_row_selected(&item(99, "chrome.exe [4x]")));
        state.toggle_selection(&chrome);
        assert!(state.selected_group.is_none());
        assert!(state.selected_pid.is_none());
    }

    #[test]
    fn test_toggle_selection_single_pid_mode_tracks_pid() {
        let mut state = MemoryTableState::default();
        state.set_group_clones(false);
        state.toggle_selection(&item(42, "chrome.exe"));
        assert_eq!(state.selected_group.as_deref(), Some("chrome.exe"));
        assert_eq!(state.selected_pid, Some(42));
        // Clicking a different instance of the same name switches the PID instead of the group.
        state.toggle_selection(&item(77, "chrome.exe"));
        assert_eq!(state.selected_pid, Some(77));
        assert!(!state.is_row_selected(&item(42, "chrome.exe")), "other instance is not selected");
        assert!(state.is_row_selected(&item(77, "chrome.exe")));
        // Clicking the same instance again clears the selection.
        state.toggle_selection(&item(77, "chrome.exe"));
        assert!(state.selected_pid.is_none());
    }

    #[test]
    fn test_group_mode_switch_clears_selection_and_flags_toggle() {
        let mut state = MemoryTableState::default();
        state.toggle_selection(&item(42, "chrome.exe"));
        assert!(state.selected_group.is_some());
        assert!(!state.take_group_clones_toggled());
        state.set_group_clones(false);
        assert!(state.selected_group.is_none());
        assert!(state.selected_pid.is_none());
        assert!(state.take_group_clones_toggled(), "mode change must flag a toggle");
        assert!(!state.take_group_clones_toggled(), "flag is consumed once");
    }

    #[test]
    fn test_group_mode_noop_does_not_clear_selection() {
        let mut state = MemoryTableState::default();
        state.toggle_selection(&item(42, "chrome.exe"));
        state.set_group_clones(true);
        assert!(state.selected_group.is_some(), "unchanged mode keeps the selection");
        assert!(!state.take_group_clones_toggled());
    }
}
