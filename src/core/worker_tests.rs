//! Unit tests for the telemetry worker, latest-slot overflow, and decoupled scans.

#[cfg(test)]
mod tests {
    use crate::core::worker::{publish, should_scan_processes, spawn, PublishResult};
    use crate::core::{LatestSlot, SystemSnapshot};
    use std::sync::{mpsc, Arc};
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
        assert_eq!(mem.snapshot_id, 1, "First snapshot id is 1");
        assert_eq!(mem.process_scan_id, 1, "First tick performs a full process scan");
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
        let (m1, m2) = (first.memory.as_ref().unwrap(), second.memory.as_ref().unwrap());
        assert!(m2.snapshot_id > m1.snapshot_id, "snapshot_id increments every tick");
        assert!(
            m2.process_scan_id >= m1.process_scan_id,
            "process_scan_id only advances on real scans"
        );
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
        assert!(mem.sample_elapsed_secs > 0.0, "Monotonic sample clock is stamped");
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
        assert!(worker.latest.get().is_some(), "Latest slot should be updated");
    }

    #[test]
    fn test_latest_slot_counts_overwrites_until_read() {
        let slot = LatestSlot::new();
        slot.store(Arc::new(SystemSnapshot::new(10, 1.0, Some(Default::default()))));
        assert_eq!(slot.overflow_count(), 0, "First store is not an overflow");
        slot.store(Arc::new(SystemSnapshot::new(20, 2.0, Some(Default::default()))));
        assert_eq!(slot.overflow_count(), 1, "Replacing an unread value overflows");
        let taken = slot.take().expect("newest value");
        assert_eq!(taken.uptime_secs, 20);
        assert!(slot.take().is_none(), "take clears the slot");
        slot.store(Arc::new(SystemSnapshot::new(30, 3.0, Some(Default::default()))));
        assert_eq!(slot.overflow_count(), 1, "Reading clears the overflow condition");
    }

    #[test]
    fn test_latest_slot_wins_when_channel_is_full() {
        let (tx, rx) = mpsc::sync_channel(1);
        let slot = LatestSlot::new();

        let first = Arc::new(SystemSnapshot::new(10, 1.0, Some(Default::default())));
        assert!(matches!(publish(&tx, &slot, first), PublishResult::Sent));

        let newest = Arc::new(SystemSnapshot::new(20, 2.0, Some(Default::default())));
        assert!(matches!(publish(&tx, &slot, newest), PublishResult::Full));
        assert_eq!(slot.overflow_count(), 1);

        let stored = slot.get().expect("slot always holds the newest");
        assert_eq!(stored.uptime_secs, 20);
        assert_eq!(stored.pulse_ram_mb, 2.0);

        let queued = rx.try_recv().expect("first snapshot remains queued");
        assert_eq!(queued.uptime_secs, 10);
        assert!(queued.memory.is_some());
        assert!(rx.try_recv().is_err(), "no extra dummy snapshot in the channel");
    }

    #[test]
    fn test_process_scan_is_decoupled_from_ram_ticks() {
        assert!(should_scan_processes(false, 0, false, false));
        assert!(!should_scan_processes(false, 1, false, false));
        assert!(should_scan_processes(false, 2, false, false));
        assert!(should_scan_processes(false, 1, true, false), "wake forces a scan");
        assert!(should_scan_processes(false, 1, false, true), "action forces a scan");
        for tick in 0..10 {
            assert!(
                !should_scan_processes(true, tick, false, false),
                "minimized mode pauses process scans"
            );
        }
    }
}
