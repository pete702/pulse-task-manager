//! Unit tests for dt-normalised process memory deltas.

#[cfg(test)]
mod tests {
    use crate::modules::memory::delta_tracker::ProcessDeltaTracker;
    use crate::modules::memory::model::ProcessSample;
    use std::time::{Duration, Instant};

    fn row(pid: u32, name: &str, ram: u64) -> ProcessSample {
        (pid, name.to_owned(), ram, ram * 2, None, 1_000 + pid as u64)
    }

    fn row_started(pid: u32, name: &str, ram: u64, start: u64) -> ProcessSample {
        (pid, name.to_owned(), ram, ram * 2, None, start)
    }

    #[test]
    fn test_first_tick_has_zero_delta() {
        let t0 = Instant::now();
        let mut tracker = ProcessDeltaTracker::new();
        let out = tracker.track_at(
            &[row(100, "chrome.exe", 1_000_000_000), row(200, "game.exe", 3_000_000_000)],
            t0,
        );
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].delta_bytes_per_sec, 0);
        assert_eq!(out[1].delta_bytes_per_sec, 0);
    }

    #[test]
    fn test_delta_normalised_by_elapsed_time() {
        let t0 = Instant::now();
        let mut tracker = ProcessDeltaTracker::new();
        tracker.track_at(&[row(100, "chrome.exe", 1_000_000_000)], t0);
        let out =
            tracker.track_at(&[row(100, "chrome.exe", 1_050_000_000)], t0 + Duration::from_secs(2));
        assert_eq!(out[0].delta_bytes_per_sec, 25_000_000);
    }

    #[test]
    fn test_delta_is_clamped_when_samples_are_too_close() {
        let t0 = Instant::now();
        let mut tracker = ProcessDeltaTracker::new();
        tracker.track_at(&[row(100, "chrome.exe", 1_000_000_000)], t0);
        let out = tracker
            .track_at(&[row(100, "chrome.exe", 1_050_000_000)], t0 + Duration::from_millis(1));
        assert_eq!(out[0].delta_bytes_per_sec, 500_000_000);
    }

    #[test]
    fn test_shrinking_process_has_negative_rate() {
        let t0 = Instant::now();
        let mut tracker = ProcessDeltaTracker::new();
        tracker.track_at(&[row(100, "chrome.exe", 1_050_000_000)], t0);
        let out =
            tracker.track_at(&[row(100, "chrome.exe", 1_020_000_000)], t0 + Duration::from_secs(2));
        assert_eq!(out[0].delta_bytes_per_sec, -15_000_000);
    }

    #[test]
    fn test_new_process_has_zero_delta() {
        let t0 = Instant::now();
        let mut tracker = ProcessDeltaTracker::new();
        tracker.track_at(&[row(100, "chrome.exe", 1_000_000_000)], t0);
        let out = tracker.track_at(
            &[row(100, "chrome.exe", 1_000_000_000), row(200, "game.exe", 3_000_000_000)],
            t0 + Duration::from_secs(1),
        );
        assert_eq!(out.iter().find(|p| p.pid == 200).unwrap().delta_bytes_per_sec, 0);
    }

    #[test]
    fn test_stale_pid_is_forgotten_and_reappears_with_zero_delta() {
        let t0 = Instant::now();
        let mut tracker = ProcessDeltaTracker::new();
        tracker.track_at(
            &[row(100, "chrome.exe", 1_000_000_000), row(200, "game.exe", 3_000_000_000)],
            t0,
        );
        tracker.track_at(&[row(100, "chrome.exe", 1_000_000_000)], t0 + Duration::from_secs(1));
        let out = tracker.track_at(
            &[row(100, "chrome.exe", 1_000_000_000), row(200, "game.exe", 3_000_000_000)],
            t0 + Duration::from_secs(2),
        );
        assert_eq!(out.iter().find(|p| p.pid == 200).unwrap().delta_bytes_per_sec, 0);
    }

    #[test]
    fn test_reused_pid_with_new_start_time_has_zero_delta() {
        let t0 = Instant::now();
        let mut tracker = ProcessDeltaTracker::new();
        tracker.track_at(&[row_started(100, "chrome.exe", 1_000_000_000, 5_000)], t0);
        let out = tracker.track_at(
            &[row_started(100, "chrome.exe", 9_000_000_000, 9_000)],
            t0 + Duration::from_secs(1),
        );
        assert_eq!(out[0].delta_bytes_per_sec, 0, "PID reuse must not inherit stale deltas");
    }
}
