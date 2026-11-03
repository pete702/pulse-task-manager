//! Background telemetry worker: periodically collects uptime, self RAM usage, and OS memory,
//! sends via bounded mpsc channel (capacity 2) and updates a shared latest-snapshot slot.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use sysinfo::System;

use crate::core::{LatestSlot, SnapshotTx, SystemSnapshot};
use crate::modules::memory::collector::{collect_memory, collect_process_memory};
use crate::modules::memory::delta_tracker::ProcessDeltaTracker;
use crate::modules::memory::model::RawProcessRow;
use crate::modules::memory::win32_api::{own_working_set_mb, trim_own_working_set};

/// Telemetry update interval (1000 ms = 1 second for 15-minute chart horizon).
const WORKER_INTERVAL: Duration = Duration::from_millis(1000);

/// Handle to the running background telemetry worker.
pub struct TelemetryWorker {
    /// Bounded channel of snapshots for the UI thread.
    pub rx: mpsc::Receiver<Arc<SystemSnapshot>>,
    /// Newest snapshot, always updated even when the channel is full.
    pub latest: LatestSlot,
    /// Set by the UI: while minimized, heavy process scans are throttled.
    pub is_minimized: Arc<AtomicBool>,
}

/// Result of publishing one snapshot to the channel.
pub(crate) enum PublishResult {
    Sent,
    Full,
    Disconnected,
}

/// Stores the freshest snapshot in the shared slot, then forwards it if the channel has room.
/// A full channel never triggers a dummy snapshot: the UI reads the slot instead.
fn publish(tx: &SnapshotTx, latest: &LatestSlot, snapshot: Arc<SystemSnapshot>) -> PublishResult {
    if let Ok(mut slot) = latest.lock() {
        *slot = Some(snapshot.clone());
    }
    match tx.try_send(snapshot) {
        Ok(()) => PublishResult::Sent,
        Err(mpsc::TrySendError::Full(_)) => PublishResult::Full,
        Err(mpsc::TrySendError::Disconnected(_)) => PublishResult::Disconnected,
    }
}

/// Process scans run every tick normally, and every 5th tick while minimized.
fn should_scan_processes(minimized: bool, tick: u64) -> bool {
    !minimized || tick % 5 == 0
}

/// Spawns the background thread and returns its receiver, latest slot, and minimized flag.
pub fn spawn() -> TelemetryWorker {
    let (tx, rx): (SnapshotTx, mpsc::Receiver<Arc<SystemSnapshot>>) = mpsc::sync_channel(2);
    let latest: LatestSlot = Arc::new(Mutex::new(None));
    let is_minimized = Arc::new(AtomicBool::new(false));

    let latest_worker = latest.clone();
    let minimized_worker = is_minimized.clone();
    thread::spawn(move || {
        let mut sys = System::new();
        let mut latest_spike = None;
        let mut delta_tracker = ProcessDeltaTracker::new();
        let mut last_processes: Vec<RawProcessRow> = Vec::new();
        let mut last_heavy = false;
        let mut was_minimized = false;
        let mut tick: u64 = 0;

        trim_own_working_set();

        loop {
            let tick_start = Instant::now();
            let uptime = System::uptime();
            let minimized = minimized_worker.load(Ordering::Relaxed);
            let force_scan = minimized && !was_minimized;
            was_minimized = minimized;

            let mut memory = collect_memory(&mut sys);
            if force_scan || should_scan_processes(minimized, tick) {
                let ((processes, spike), heavy) = collect_process_memory(
                    &mut sys,
                    memory.ram_total_bytes,
                    memory.ram_usage_percent(),
                    &mut delta_tracker,
                );
                memory.processes = processes;
                memory.has_heavy_process = heavy;
                last_heavy = heavy;
                if minimized {
                    last_processes.clone_from(&memory.processes);
                }
                if spike.is_some() {
                    latest_spike = spike;
                }
            } else {
                memory.processes.clone_from(&last_processes);
                memory.has_heavy_process = last_heavy;
            }
            memory.latest_spike.clone_from(&latest_spike);

            let pulse_ram_mb = own_working_set_mb();

            let elapsed = tick_start.elapsed();
            if elapsed < WORKER_INTERVAL {
                thread::sleep(WORKER_INTERVAL - elapsed);
            }
            let os_scheduler_jitter_ms =
                tick_start.elapsed().as_millis().saturating_sub(WORKER_INTERVAL.as_millis()) as u32;

            let snapshot = Arc::new(SystemSnapshot::new_with_jitter(
                uptime,
                pulse_ram_mb,
                Some(memory),
                os_scheduler_jitter_ms,
            ));

            match publish(&tx, &latest_worker, snapshot) {
                PublishResult::Disconnected => break,
                PublishResult::Sent | PublishResult::Full => {}
            }
            tick = tick.wrapping_add(1);
        }
    });

    TelemetryWorker { rx, latest, is_minimized }
}

#[cfg(test)]
#[path = "worker_tests.rs"]
mod worker_tests;
