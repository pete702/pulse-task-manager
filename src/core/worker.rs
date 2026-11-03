//! Background telemetry worker: periodically collects uptime, self RAM usage, and OS memory,
//! sends via mpsc channel.

use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use sysinfo::System;

use crate::core::{SnapshotTx, SystemSnapshot};
use crate::modules::memory::collector::{collect_memory, collect_process_memory};
use crate::modules::memory::delta_tracker::ProcessDeltaTracker;
use crate::modules::memory::win32_api::{own_working_set_mb, trim_own_working_set};

/// Telemetry update interval (1000 ms = 1 second for 15-minute chart horizon).
const WORKER_INTERVAL: Duration = Duration::from_millis(1000);

/// Spawns a background thread that reads `System::uptime()`, current process RAM,
/// OS memory, and the process memory table every 1s and sends `SystemSnapshot`
/// through the channel. Returns a `Receiver` for the UI thread.
///
/// Optimization: starts with `System::new()` and refreshes only memory and process
/// memory, avoiding a full system scan. Own RSS is measured directly with Win32
/// `K32GetProcessMemoryInfo` after trimming the working set.
/// Trim is performed ONLY ONCE at startup to avoid page-fault thrashing while the GUI window is open.
pub fn spawn() -> mpsc::Receiver<SystemSnapshot> {
    let (tx, rx): (SnapshotTx, mpsc::Receiver<SystemSnapshot>) = mpsc::channel();

    thread::spawn(move || {
        let mut sys = System::new();
        let mut latest_spike = None;
        let mut delta_tracker = ProcessDeltaTracker::new();

        // Trim working set ONCE at startup (before the loop) to avoid page-fault thrashing
        // while the GUI window is open. Periodic trimming causes page-fault storms when
        // the OpenGL/DirectX window is actively rendering.
        trim_own_working_set();

        loop {
            let uptime = System::uptime();

            // Collect OS memory (RAM/Swap), then the per-process memory table.
            let mut memory = collect_memory(&mut sys);
            let (processes, spike) = collect_process_memory(
                &mut sys,
                memory.ram_total_bytes,
                memory.ram_usage_percent(),
                &mut delta_tracker,
            );
            memory.processes = processes;
            if spike.is_some() {
                latest_spike = spike;
            }
            memory.latest_spike = latest_spike.clone();

            // Measure own RSS after trimming
            let pulse_ram_mb = own_working_set_mb();

            // Measure OS scheduler jitter (how much the 1000ms sleep overshot)
            let sleep_start = std::time::Instant::now();
            thread::sleep(WORKER_INTERVAL);
            let os_scheduler_jitter_ms =
                sleep_start.elapsed().as_millis().saturating_sub(1000) as u32;

            let snapshot = SystemSnapshot::new_with_jitter(
                uptime,
                pulse_ram_mb,
                Some(memory),
                os_scheduler_jitter_ms,
            );

            // Send snapshot; if receiver is closed, terminate thread
            if tx.send(snapshot).is_err() {
                break;
            }
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
            .recv_timeout(Duration::from_secs(3))
            .expect("Worker should send first snapshot within 3 seconds");

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
            .recv_timeout(Duration::from_secs(3))
            .expect("First snapshot");
        let second = rx
            .recv_timeout(Duration::from_secs(3))
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
            .recv_timeout(Duration::from_secs(3))
            .expect("First snapshot");

        let mem = snapshot.memory.expect("Memory should be present");
        assert!(mem.ram_total_bytes > 0);
        assert!(mem.ram_used_bytes > 0);
        assert!(mem.ram_usage_percent() > 0.0);
        assert!(mem.ram_usage_percent() <= 100.0);
    }

    #[test]
    fn test_os_scheduler_jitter_recorded() {
        let rx = spawn();
        let snapshot = rx
            .recv_timeout(Duration::from_secs(3))
            .expect("First snapshot");
        // Jitter should be recorded (may be 0 on precise systems)
        let _ = snapshot.os_scheduler_jitter_ms;
    }
}
