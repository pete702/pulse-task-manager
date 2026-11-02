//! Background telemetry worker: periodically collects uptime and self RAM usage, sends via mpsc channel.

use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use sysinfo::{Pid, System};

use crate::core::{SnapshotTx, SystemSnapshot};

/// Telemetry update interval (500 ms).
const WORKER_INTERVAL: Duration = Duration::from_millis(500);

/// Spawns a background thread that reads `System::uptime()` and current process RAM every 500ms
/// and sends `SystemSnapshot` through the channel.
/// Returns a `Receiver` for the UI thread.
pub fn spawn() -> mpsc::Receiver<SystemSnapshot> {
    let (tx, rx): (SnapshotTx, mpsc::Receiver<SystemSnapshot>) = mpsc::channel();

    thread::spawn(move || {
        let mut sys = System::new_all();
        // Initial refresh
        sys.refresh_all();

        // Get current process PID
        let pid = Pid::from(std::process::id() as usize);

        loop {
            let uptime = System::uptime();

            // Refresh process list to get current process memory
            sys.refresh_processes();
            let pulse_ram_mb = sys
                .process(pid)
                .map(|p| p.memory() as f32 / 1024.0) // Convert KB to MB
                .unwrap_or(0.0);

            let snapshot = SystemSnapshot::new(uptime, pulse_ram_mb);

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
        // Verify pulse_ram_mb is non-negative
        assert!(
            snapshot.pulse_ram_mb >= 0.0,
            "Pulse RAM should be non-negative"
        );
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
        // RAM should be non-negative
        assert!(first.pulse_ram_mb >= 0.0);
        assert!(second.pulse_ram_mb >= 0.0);
    }
}
