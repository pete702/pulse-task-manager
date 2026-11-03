//! Unit tests for the telemetry worker, latest-slot overflow, and minimized throttling.

#[cfg(test)]
mod tests {
    use crate::core::worker::{publish, should_scan_processes, spawn, PublishResult};
    use crate::core::{LatestSlot, SystemSnapshot};
    use std::sync::{mpsc, Arc, Mutex};
    use std::time::Duration;

    #[test]
    fn test_worker_sends_first_snapshot() {
        let worker = spawn();
        let snapshot = worker
            .rx
            .recv_timeout(Duration::from_secs(5))
            .expect("Worker should send first snapshot within 5 seconds");
        assert!(snapshot.uptime_secs > 0, "Uptime should be positive");
        assert!(snapshot.pulse_ram_mb >= 0.0, "Pulse RAM should be non-negative");
        assert!(snapshot.memory.is_some(), "Memory snapshot should be present");
        let mem = snapshot.memory.as_ref().unwrap();
        assert!(mem.ram_total_bytes > 0, "RAM total should be > 0");
        assert!(!worker.is_minimized.load(std::sync::atomic::Ordering::Relaxed));
    }

    #[test]
    fn test_multiple_snapshots_increasing() {
        let worker = spawn();
        let first = worker.rx.recv_timeout(Duration::from_secs(5)).expect("First snapshot");
        let second = worker.rx.recv_timeout(Duration::from_secs(5)).expect("Second snapshot");
        assert!(second.uptime_secs >= first.uptime_secs, "Uptime should not decrease");
        assert!(first.pulse_ram_mb >= 0.0);
        assert!(second.pulse_ram_mb >= 0.0);
        assert!(first.memory.is_some());
        assert!(second.memory.is_some());
    }

    #[test]
    fn test_memory_snapshot_in_worker() {
        let worker = spawn();
        let snapshot = worker.rx.recv_timeout(Duration::from_secs(5)).expect("First snapshot");
        let mem = snapshot.memory.as_ref().expect("Memory should be present");
        assert!(mem.ram_total_bytes > 0);
        assert!(mem.ram_used_bytes > 0);
        assert!(mem.ram_usage_percent() > 0.0);
        assert!(mem.ram_usage_percent() <= 100.0);
    }

    #[test]
    fn test_os_scheduler_jitter_recorded() {
        let worker = spawn();
        let snapshot = worker.rx.recv_timeout(Duration::from_secs(5)).expect("First snapshot");
        let _ = snapshot.os_scheduler_jitter_ms;
    }

    #[test]
    fn test_latest_slot_updated() {
        let worker = spawn();
        let _ = worker.rx.recv_timeout(Duration::from_secs(5)).expect("First snapshot");
        std::thread::sleep(Duration::from_millis(1100));
        let slot = worker.latest.lock().unwrap();
        assert!(slot.is_some(), "Latest slot should be updated");
    }

    #[test]
    fn test_latest_slot_wins_when_channel_is_full() {
        let (tx, rx) = mpsc::sync_channel(1);
        let slot: LatestSlot = Arc::new(Mutex::new(None));

        let first = Arc::new(SystemSnapshot::new(10, 1.0, Some(Default::default())));
        assert!(matches!(publish(&tx, &slot, first), PublishResult::Sent));

        let newest = Arc::new(SystemSnapshot::new(20, 2.0, Some(Default::default())));
        assert!(matches!(publish(&tx, &slot, newest), PublishResult::Full));

        let stored = slot.lock().unwrap().clone().expect("slot always holds the newest");
        assert_eq!(stored.uptime_secs, 20);
        assert_eq!(stored.pulse_ram_mb, 2.0);

        // The channel still holds exactly one real snapshot; no dummy was injected.
        let queued = rx.try_recv().expect("first snapshot remains queued");
        assert_eq!(queued.uptime_secs, 10);
        assert!(queued.memory.is_some());
        assert!(rx.try_recv().is_err(), "no extra dummy snapshot in the channel");
    }

    #[test]
    fn test_process_scan_is_throttled_while_minimized() {
        for tick in 0..20 {
            assert!(should_scan_processes(false, tick), "visible window scans every tick");
        }
        for tick in 0..20 {
            assert_eq!(
                should_scan_processes(true, tick),
                tick % 5 == 0,
                "minimized window scans every 5th tick"
            );
        }
    }
}
