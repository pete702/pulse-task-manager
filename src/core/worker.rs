//! Background telemetry worker with decoupled RAM and process polling.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::{Duration, Instant};

use sysinfo::System;

use crate::core::{LatestSlot, SnapshotTx, SystemSnapshot};
use crate::modules::memory::collector::{collect_memory, collect_process_memory};
use crate::modules::memory::delta_tracker::ProcessDeltaTracker;
use crate::modules::memory::model::RawProcessRow;
use crate::modules::memory::win32_api::own_working_set_mb;

/// Telemetry update interval (1000 ms = 1 second for 15-minute chart horizon).
const WORKER_INTERVAL: Duration = Duration::from_millis(1000);
/// Full process scans run every 2nd visible tick (2 seconds).
const PROCESS_SCAN_TICKS: u64 = 2;
/// RAM usage threshold that triggers the high-pressure banner.
const HIGH_RAM_PRESSURE_PCT: f32 = 85.0;

static PROCESS_SCAN_REQUEST: AtomicBool = AtomicBool::new(false);

/// Requests an immediate process scan on the next worker tick.
pub fn request_process_scan() {
    PROCESS_SCAN_REQUEST.store(true, Ordering::Relaxed);
}

fn take_process_scan_request() -> bool {
    PROCESS_SCAN_REQUEST.swap(false, Ordering::Relaxed)
}

/// Handle to the running background telemetry worker.
pub struct TelemetryWorker {
    pub rx: mpsc::Receiver<Arc<SystemSnapshot>>,
    pub latest: LatestSlot,
    pub is_minimized: Arc<AtomicBool>,
}

/// Result of publishing one snapshot to the channel.
pub(crate) enum PublishResult {
    Sent,
    Full,
    Disconnected,
}

/// Stores the freshest snapshot in the shared slot, then forwards it if the channel has room.
fn publish(tx: &SnapshotTx, latest: &LatestSlot, snapshot: Arc<SystemSnapshot>) -> PublishResult {
    latest.store(snapshot.clone());
    match tx.try_send(snapshot) {
        Ok(()) => PublishResult::Sent,
        Err(mpsc::TrySendError::Full(_)) => PublishResult::Full,
        Err(mpsc::TrySendError::Disconnected(_)) => PublishResult::Disconnected,
    }
}

/// Visible mode scans every 2nd tick; wake and action requests scan immediately.
/// Minimized mode pauses full process scans entirely.
pub(crate) fn should_scan_processes(
    minimized: bool,
    tick: u64,
    wake: bool,
    requested: bool,
) -> bool {
    !minimized && (wake || requested || tick % PROCESS_SCAN_TICKS == 0)
}

/// Spawns the background thread and returns its receiver, latest slot, and minimized flag.
pub fn spawn() -> TelemetryWorker {
    let (tx, rx): (SnapshotTx, mpsc::Receiver<Arc<SystemSnapshot>>) = mpsc::sync_channel(2);
    let latest = LatestSlot::new();
    let is_minimized = Arc::new(AtomicBool::new(false));

    let latest_worker = latest.clone();
    let minimized_worker = is_minimized.clone();
    thread::spawn(move || {
        let started = Instant::now();
        let mut sys = System::new();
        let mut latest_spike = None;
        let mut delta_tracker = ProcessDeltaTracker::new();
        let mut last_processes: Arc<Vec<RawProcessRow>> = Arc::new(Vec::new());
        let mut last_heavy = false;
        let mut was_minimized = false;
        let mut tick: u64 = 0;
        let mut snapshot_id: u64 = 0;
        let mut process_scan_id: u64 = 0;

        loop {
            let tick_start = Instant::now();
            let uptime = System::uptime();
            let minimized = minimized_worker.load(Ordering::Relaxed);
            let wake = was_minimized && !minimized;
            let requested = take_process_scan_request();
            was_minimized = minimized;
            let refresh = should_scan_processes(minimized, tick, wake, requested);

            let mut memory = collect_memory(&mut sys);
            if refresh {
                process_scan_id += 1;
                let ((processes, spike), heavy) = collect_process_memory(
                    &mut sys,
                    memory.ram_total_bytes,
                    memory.ram_usage_percent(),
                    &mut delta_tracker,
                    true,
                );
                let shared = Arc::new(processes);
                last_processes = Arc::clone(&shared);
                memory.processes = shared;
                memory.has_heavy_process = heavy;
                last_heavy = heavy;
                // Honest alert lifecycle: a spike is published only while RAM is still >= 85%.
                // When RAM drops back below the threshold, clear the stale banner.
                latest_spike = spike;
            } else {
                memory.processes = Arc::clone(&last_processes);
                memory.has_heavy_process = last_heavy;
                if memory.ram_usage_percent() < HIGH_RAM_PRESSURE_PCT {
                    latest_spike = None;
                }
            }
            memory.latest_spike.clone_from(&latest_spike);
            memory.scan_duration_ms = tick_start.elapsed().as_secs_f32() * 1000.0;

            let pulse_ram_mb = own_working_set_mb();
            snapshot_id += 1;
            memory.snapshot_id = snapshot_id;
            memory.process_scan_id = process_scan_id;
            memory.sample_elapsed_secs = started.elapsed().as_secs_f64();
            memory.channel_overflows = latest_worker.overflow_count();

            let elapsed = tick_start.elapsed();
            let mut sleep_overshoot_ms: u64 = 0;
            if elapsed < WORKER_INTERVAL {
                let requested = WORKER_INTERVAL - elapsed;
                let before_sleep = Instant::now();
                thread::sleep(requested);
                // Pure sleep overshoot: actual_sleep_ms - requested_sleep_ms. Scan and
                // collection time are already excluded because `elapsed` is subtracted first.
                sleep_overshoot_ms =
                    before_sleep.elapsed().as_millis().saturating_sub(requested.as_millis()) as u64;
            }
            let os_scheduler_jitter_ms = sleep_overshoot_ms.min(u32::MAX as u64) as u32;

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
