//! Unit tests for the non-blocking memory action bar.

#[cfg(test)]
mod tests {
    use crate::modules::memory::action_bar::{
        format_trim_status, MemoryActionBarState, CONFIRM_TTL, STATUS_TTL,
    };
    use crate::modules::memory::actions::ReclaimReport;
    use std::time::{Duration, Instant};

    #[test]
    fn test_confirm_kill_expires_after_four_seconds() {
        let mut state = MemoryActionBarState::default();
        let t0 = Instant::now();
        state.begin_confirm_kill("chrome.exe", t0);
        assert!(state.confirm_kill_active("chrome.exe", t0 + Duration::from_secs(3)));
        assert!(!state.confirm_kill_active("other.exe", t0));
        assert!(!state.confirm_kill_active("chrome.exe", t0 + CONFIRM_TTL));
    }

    #[test]
    fn test_cancel_confirm_disarms_kill() {
        let mut state = MemoryActionBarState::default();
        let t0 = Instant::now();
        state.begin_confirm_kill("game.exe", t0);
        state.cancel_confirm();
        assert!(!state.confirm_kill_active("game.exe", t0));
    }

    #[test]
    fn test_status_fades_after_eight_seconds() {
        let mut state = MemoryActionBarState::default();
        let t0 = Instant::now();
        state.set_status(t0, "Working-set trim: 12.5 MB across 3 procs (1 skipped)".to_owned());
        assert!(state.status_text(t0 + Duration::from_secs(7)).is_some());
        assert!(state.status_text(t0 + STATUS_TTL).is_none());
    }

    #[test]
    fn test_action_bar_moves_from_empty_to_protected_selection() {
        use crate::modules::memory::action_bar_view::render_action_bar;
        use crate::modules::memory::model::MemorySnapshot;
        use crate::modules::memory::table_view::MemoryTableState;

        let mem = MemorySnapshot {
            ram_total_bytes: 16_000_000_000,
            processes: vec![(900, "svchost.exe".to_owned(), 1_000_000, 2_000_000, 0, None)],
            ..Default::default()
        };
        let mut ts = MemoryTableState::default();
        let mut st = MemoryActionBarState::default();
        egui::__run_test_ui(|ui| render_action_bar(ui, &mem, &ts, &mut st));
        ts.selected_group = Some("svchost.exe".to_owned());
        egui::__run_test_ui(|ui| render_action_bar(ui, &mem, &ts, &mut st));
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
        assert_eq!(text, "Working-set trim: 5.0 MB across 4 procs (3 skipped)");
        assert!(!text.contains("Freed"));
    }
}
