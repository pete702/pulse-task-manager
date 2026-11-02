//! Background telemetry worker: periodically collects uptime and sends via mpsc channel.

use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use sysinfo::System;

use crate::core::{SnapshotTx, SystemSnapshot};

/// Telemetry update interval (500 ms).
const WORKER_INTERVAL: Duration = Duration::from_millis(500);

/// Spawns a background thread that reads `System::uptime()` every 500ms
/// and sends `SystemSnapshot` through the channel.
/// Returns a `Receiver` for the UI thread.
pub fn spawn() -> mpsc::Receiver<SystemSnapshot> {
    let (tx, rx): (SnapshotTx, mpsc::Receiver<SystemSnapshot>) = mpsc::channel();

    thread::spawn(move || {
        let mut sys = System::new_all();
        // Initial refresh
        sys.refresh_all();

        loop {
            let uptime = System::uptime();
            let snapshot = SystemSnapshot::new(uptime);

            // Send snapshot; if receiver is closed, terminate thread
            if tx.send(snapshot).is_err() {
                break;
            }

            thread::sleep(WORKER_INTERVAL);
        }
    });

    rx
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_worker_sends_first_snapshot() {
        let rx = spawn();

        // Wait for first snapshot (should arrive quickly)
        let snapshot = rx
            .recv_timeout(Duration::from_secs(2))
            .expect("Worker should send first snapshot within 2 seconds");

        // Verify uptime > 0 (system is running)
        assert!(snapshot.uptime_secs > 0, "Uptime should be positive");
    }

    #[test]
    fn test_multiple_snapshots_increasing() {
        let rx = spawn();

        let first = rx
            .recv_timeout(Duration::from_secs(2))
            .expect("First snapshot");
        let second = rx
            .recv_timeout(Duration::from_secs(2))
            .expect("Second snapshot");

        // Second snapshot should be later (or equal if very fast)
        assert!(
            second.uptime_secs >= first.uptime_secs,
            "Uptime should not decrease"
        );
    }
}
