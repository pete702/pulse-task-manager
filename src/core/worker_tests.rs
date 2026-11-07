//! Unit tests for the telemetry worker, latest-slot overflow, and decoupled scans.

#[cfg(test)]
mod tests {
    use crate::core::worker::spawn_telemetry_worker;
    use crate::core::worker_io::{publish, PublishResult};
    use crate::core::worker_policy::should_scan_processes;
    use crate::core::{LatestSlot, SystemSnapshot};
    use std::sync::{mpsc, Arc};
    use std::time::Duration;

    #[test]
    fn test_worker_integration_single_cycle() {
        let (worker, rx) = spawn_telemetry_worker();
        let snapshot = rx
            .recv_timeout(Duration::from_secs(15))
            .expect("Worker should send first snapshot within 15 seconds");
        assert!(snapshot.uptime_secs > 0, "Uptime should be positive");
        assert!(snapshot.pulse_ram_mb >= 0.0, "Pulse RAM should be non-negative");
        let mem = snapshot.memory.as_ref().expect("Memory snapshot should be present");
        assert!(mem.ram_total_bytes > 0, "RAM total should be > 0");
        assert!(mem.ram_used_bytes > 0);
        assert!(mem.ram_usage_percent() > 0.0);
        assert!(mem.ram_usage_percent() <= 100.0);
        assert!(mem.sample_elapsed_secs > 0.0, "Monotonic sample clock is stamped");
        assert_eq!(mem.snapshot_id, 1, "First snapshot id is 1");
        assert_eq!(mem.process_scan_id, 1, "First tick performs a full process scan");
        let _ = snapshot.os_scheduler_jitter_ms;
        assert!(!worker.is_minimized.load(std::sync::atomic::Ordering::Relaxed));
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
        let wakeup: Arc<dyn Fn() + Send + Sync + 'static> = Arc::new(|| {});

        let first = Arc::new(SystemSnapshot::new(10, 1.0, Some(Default::default())));
        assert!(matches!(publish(&tx, &slot, first, &wakeup, false), PublishResult::Sent));

        let newest = Arc::new(SystemSnapshot::new(20, 2.0, Some(Default::default())));
        assert!(matches!(publish(&tx, &slot, newest, &wakeup, false), PublishResult::Full));
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
    fn test_latest_slot_drain_all_preserves_fifo_history() {
        let (tx, rx) = mpsc::sync_channel(2);
        let slot = LatestSlot::new();
        for i in 1..=2u64 {
            let s = SystemSnapshot::new(i, i as f32, Some(Default::default()));
            tx.try_send(Arc::new(s)).expect("channel slot free");
        }
        for i in 3..=27u64 {
            let mem = crate::modules::memory::model::MemorySnapshot {
                sample_elapsed_secs: i as f64,
                snapshot_id: i,
                ..Default::default()
            };
            let s = SystemSnapshot::new(i, i as f32, Some(mem));
            slot.store(Arc::new(s));
        }
        let mut all: Vec<Arc<SystemSnapshot>> = Vec::new();
        while let Ok(s) = rx.try_recv() {
            all.push(s);
        }
        all.extend(slot.drain_all());
        assert_eq!(all.len(), 27, "rx plus drain yields every tick");
        for (idx, snap) in all.iter().enumerate() {
            assert_eq!(snap.uptime_secs, idx as u64 + 1, "strict chronological order");
        }
        assert_eq!(slot.overflow_count(), 0, "drain resets overflow count");
        assert!(slot.drain_all().is_empty(), "second drain is empty");
    }
    #[test]
    fn test_process_scan_is_decoupled_from_ram_ticks() {
        // With SCAN_TICKS_VISIBLE = 3, scans happen at tick 0, 3, 6, ...
        assert!(should_scan_processes(false, 0, false, false, true));
        assert!(!should_scan_processes(false, 1, false, false, true));
        assert!(!should_scan_processes(false, 2, false, false, true));
        assert!(should_scan_processes(false, 3, false, false, true), "every 3rd tick scans");
        assert!(should_scan_processes(false, 1, true, false, true), "wake forces a scan");
        assert!(should_scan_processes(false, 1, false, true, true), "action forces a scan");
        for tick in 0..10 {
            assert!(
                !should_scan_processes(true, tick, false, false, true),
                "minimized mode pauses process scans"
            );
        }
    }
}
