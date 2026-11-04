//! Unit tests for the non-blocking memory action bar.

#[cfg(test)]
mod tests {
    use crate::modules::memory::action_bar::{
        format_trim_status, MemoryActionBarState, CONFIRM_TTL, STATUS_TTL,
    };
    use crate::modules::memory::actions::ReclaimReport;
    use crate::modules::memory::process_target::ProcessTarget;
    use std::time::{Duration, Instant};

    fn make_target(pid: u32, name: &str) -> ProcessTarget {
        ProcessTarget {
            pid,
            creation_time: Some(pid as u64),
            name: name.to_owned(),
            exe_path: None,
        }
    }

    #[test]
    fn test_confirm_kill_expires_after_three_seconds() {
        let mut state = MemoryActionBarState::default();
        let t0 = Instant::now();
        state.begin_confirm_kill("chrome.exe", &[make_target(10, "chrome.exe")], t0);
        assert!(state.confirm_kill_active("chrome.exe", t0 + Duration::from_secs(2)));
        assert!(!state.confirm_kill_active("other.exe", t0));
        assert!(!state.confirm_kill_active("chrome.exe", t0 + CONFIRM_TTL));
    }

    #[test]
    fn test_cancel_confirm_disarms_kill() {
        let mut state = MemoryActionBarState::default();
        let t0 = Instant::now();
        state.begin_confirm_kill("game.exe", &[make_target(10, "game.exe")], t0);
        state.cancel_confirm();
        assert!(!state.confirm_kill_active("game.exe", t0));
    }

    #[test]
    fn test_status_fades_after_eight_seconds() {
        let mut state = MemoryActionBarState::default();
        let t0 = Instant::now();
        state.set_status(
            t0,
            "Observed WS delta: 12.5 MB (3 processes trimmed, 1 skipped)".to_owned(),
        );
        assert!(state.status_text(t0 + Duration::from_secs(7)).is_some());
        assert!(state.status_text(t0 + STATUS_TTL).is_none());
    }

    #[test]
    fn test_action_bar_moves_from_empty_to_protected_selection() {
        use crate::modules::memory::action_bar_view::render_action_bar;
        use crate::modules::memory::model::{MemorySnapshot, RawProcessRow};
        use crate::modules::memory::table_state::MemoryTableState;
        use std::sync::Arc;

        let mem = MemorySnapshot {
            ram_total_bytes: 16_000_000_000,
            processes: Arc::new(vec![RawProcessRow {
                pid: 900,
                name: Arc::from("svchost.exe"),
                ram_bytes: 1_000_000,
                virtual_bytes: 2_000_000,
                delta_bytes_per_sec: 0,
                exe_path: None,
                creation_time: None,
            }]),
            ..Default::default()
        };
        let mut ts = MemoryTableState::default();
        let mut st = MemoryActionBarState::default();
        egui::__run_test_ui(|ui| render_action_bar(ui, &mem, &mut ts, &mut st, &mut false));
        ts.selected_group = Some("svchost.exe".to_owned());
        egui::__run_test_ui(|ui| render_action_bar(ui, &mem, &mut ts, &mut st, &mut false));
    }

    #[test]
    fn test_honest_trim_status_format() {
        let report = ReclaimReport {
            trimmed_count: 4,
            skipped_protected: 1,
            skipped_access_denied: 2,
            reclaimed_bytes: 5 * 1_048_576,
        };
        let text = format_trim_status(&report);
        assert_eq!(text, "Observed WS delta: 5.0 MB (4 processes trimmed, 3 skipped)");
        assert!(!text.contains("Reclaimed"));
        assert!(!text.contains("Freed"));
    }

    #[test]
    fn test_frozen_kill_targets_are_captured_on_first_click() {
        let mut state = MemoryActionBarState::default();
        let t0 = Instant::now();
        let targets = vec![make_target(10, "chrome.exe"), make_target(11, "chrome.exe")];

        // First click: arm confirmation with frozen targets
        state.begin_confirm_kill("chrome.exe", &targets, t0);
        let frozen = state.confirmed_targets("chrome.exe", t0);
        assert!(frozen.is_some());
        let frozen = frozen.unwrap();
        assert_eq!(frozen.len(), 2);
        assert_eq!(frozen[0].pid, 10);
        assert_eq!(frozen[1].pid, 11);

        // Simulate a new process with same name spawning (PID 12)
        // The frozen targets should NOT include PID 12
        let new_targets = vec![
            make_target(10, "chrome.exe"),
            make_target(11, "chrome.exe"),
            make_target(12, "chrome.exe"),
        ];
        state.begin_confirm_kill("chrome.exe", &new_targets, t0 + Duration::from_secs(1));
        let still_frozen = state.confirmed_targets("chrome.exe", t0 + Duration::from_secs(1));
        assert!(still_frozen.is_some());
        // Should still be the original frozen targets (2 items), not the new 3
        assert_eq!(still_frozen.unwrap().len(), 2);
    }

    #[test]
    fn test_confirm_kill_clears_when_group_changes() {
        let mut state = MemoryActionBarState::default();
        let t0 = Instant::now();

        state.begin_confirm_kill("chrome.exe", &[make_target(10, "chrome.exe")], t0);
        assert!(state.confirm_kill_active("chrome.exe", t0));

        // Different group key - should not be active
        assert!(!state.confirm_kill_active("game.exe", t0));

        // Clear by calling cancel_confirm (simulating selection change)
        state.cancel_confirm();
        assert!(!state.confirm_kill_active("chrome.exe", t0));
    }
}
