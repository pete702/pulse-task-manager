//! Process memory delta tracker for PULSE.
//! Tracks per-process working set changes across collection ticks.

use std::collections::HashMap;

use crate::modules::memory::process_model::MemorySpikeEvent;

/// Row of raw process memory data: (PID, name, RAM bytes, virtual bytes, delta bytes/sec).
pub type RawProcessRow = (u32, String, u64, u64, i64);

/// Result of a process memory collection pass.
pub type ProcessMemoryScan = (Vec<RawProcessRow>, Option<MemorySpikeEvent>);

/// Internal state for tracking process memory deltas across ticks.
pub struct ProcessDeltaTracker {
    prev_ram_bytes: HashMap<u32, u64>,
}

impl ProcessDeltaTracker {
    pub fn new() -> Self {
        Self {
            prev_ram_bytes: HashMap::new(),
        }
    }

    /// Calculate delta for each process and update tracking.
    /// Returns vector of (PID, name, RAM bytes, virtual bytes, delta bytes/sec).
    pub fn track(&mut self, processes: &[(u32, String, u64, u64)]) -> Vec<RawProcessRow> {
        let mut result = Vec::with_capacity(processes.len());
        for (pid, name, ram, virt) in processes {
            let prev = self.prev_ram_bytes.get(pid).copied().unwrap_or(*ram);
            let delta = *ram as i64 - prev as i64;
            self.prev_ram_bytes.insert(*pid, *ram);
            result.push((*pid, name.clone(), *ram, *virt, delta));
        }
        // Clean up stale PIDs (optional, keep it simple for now)
        self.prev_ram_bytes
            .retain(|pid, _| processes.iter().any(|(p, _, _, _)| p == pid));
        result
    }
}

impl Default for ProcessDeltaTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_delta_tracker_first_tick() {
        let mut tracker = ProcessDeltaTracker::new();
        let processes = vec![
            (100, "chrome.exe".to_owned(), 1_000_000_000, 2_000_000_000),
            (200, "game.exe".to_owned(), 3_000_000_000, 4_000_000_000),
        ];

        let result = tracker.track(&processes);

        // First tick: delta should be 0 (no previous value)
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].4, 0); // delta for chrome
        assert_eq!(result[1].4, 0); // delta for game
    }

    #[test]
    fn test_process_delta_tracker_second_tick_growing() {
        let mut tracker = ProcessDeltaTracker::new();
        // First tick
        tracker.track(&[(100, "chrome.exe".to_owned(), 1_000_000_000, 2_000_000_000)]);
        // Second tick - process grew by 50 MB
        let result = tracker.track(&[(100, "chrome.exe".to_owned(), 1_050_000_000, 2_000_000_000)]);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].4, 50_000_000); // +50 MB delta
    }

    #[test]
    fn test_process_delta_tracker_second_tick_shrinking() {
        let mut tracker = ProcessDeltaTracker::new();
        // First tick
        tracker.track(&[(100, "chrome.exe".to_owned(), 1_050_000_000, 2_000_000_000)]);
        // Second tick - process shrank by 30 MB
        let result = tracker.track(&[(100, "chrome.exe".to_owned(), 1_020_000_000, 2_000_000_000)]);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].4, -30_000_000); // -30 MB delta
    }

    #[test]
    fn test_process_delta_tracker_new_process() {
        let mut tracker = ProcessDeltaTracker::new();
        // First tick
        tracker.track(&[(100, "chrome.exe".to_owned(), 1_000_000_000, 2_000_000_000)]);
        // Second tick - new process appears
        let result = tracker.track(&[
            (100, "chrome.exe".to_owned(), 1_000_000_000, 2_000_000_000),
            (200, "game.exe".to_owned(), 3_000_000_000, 4_000_000_000),
        ]);

        assert_eq!(result.len(), 2);
        // chrome: no change
        assert_eq!(result.iter().find(|p| p.0 == 100).unwrap().4, 0);
        // game: first appearance, delta = 0
        assert_eq!(result.iter().find(|p| p.0 == 200).unwrap().4, 0);
    }

    #[test]
    fn test_process_delta_tracker_stale_pid_cleanup() {
        let mut tracker = ProcessDeltaTracker::new();
        // First tick
        tracker.track(&[
            (100, "chrome.exe".to_owned(), 1_000_000_000, 2_000_000_000),
            (200, "game.exe".to_owned(), 3_000_000_000, 4_000_000_000),
        ]);
        // Second tick - game.exe terminated
        let result = tracker.track(&[(100, "chrome.exe".to_owned(), 1_000_000_000, 2_000_000_000)]);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, 100);
        // Internal map should only have PID 100
        assert_eq!(tracker.prev_ram_bytes.len(), 1);
        assert!(tracker.prev_ram_bytes.contains_key(&100));
        assert!(!tracker.prev_ram_bytes.contains_key(&200));
    }
}
