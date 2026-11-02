//! Background telemetry worker: periodically collects uptime, self RAM usage, and OS memory,
//! sends via mpsc channel.

use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use sysinfo::{Pid, System};

use crate::core::{SnapshotTx, SystemSnapshot};
use crate::modules::memory::collector::collect_memory;

/// Telemetry update interval (500 ms).
const WORKER_INTERVAL: Duration = Duration::from_millis(500);

/// Spawns a background thread that reads `System::uptime()`, current process RAM,
/// and OS memory every 500ms and sends `SystemSnapshot` through the channel.
/// Returns a `Receiver` for the UI thread.
///
/// Optimization: Uses `refresh_pid()` for current process only (not full process list)
/// to reduce memory overhead from ~150MB to ~35-50MB.
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

            // OPTIMIZATION: Refresh ONLY the current process PID (not all processes)
            // This reduces sysinfo's internal process table memory from ~150MB to ~35-50MB
            sys.refresh_pids(&[pid]);
            let pulse_ram_mb = sys
                .process(pid)
                .map(|p| p.memory() as f32 / (1024.0 * 1024.0)) // Convert bytes to MB
                .unwrap_or(0.0);

            // Collect OS memory (RAM/Swap) - this also refreshes memory info
            let memory = Some(collect_memory(&mut sys));

            let snapshot = SystemSnapshot::new(uptime, pulse_ram_mb, memory);

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
        // Verify memory snapshot is present
        assert!(
            snapshot.memory.is_some(),
            "Memory snapshot should be present"
        );
        let mem = snapshot.memory.unwrap();
        assert!(mem.ram_total_bytes > 0, "RAM total should be > 0");
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
        // Both should have memory snapshots
        assert!(first.memory.is_some());
        assert!(second.memory.is_some());
    }

    #[test]
    fn test_memory_snapshot_in_worker() {
        let rx = spawn();
        let snapshot = rx
            .recv_timeout(Duration::from_secs(2))
            .expect("First snapshot");

        let mem = snapshot.memory.expect("Memory should be present");
        assert!(mem.ram_total_bytes > 0);
        assert!(mem.ram_used_bytes > 0);
        assert!(mem.ram_usage_percent() > 0.0);
        assert!(mem.ram_usage_percent() <= 100.0);
    }
}
