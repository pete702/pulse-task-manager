//! Background telemetry worker loop logic.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::Instant;
use sysinfo::System;

use crate::core::{
    worker_io::{publish, PublishResult, Wakeup},
    worker_policy::*,
    LatestSlot, SnapshotTx, SystemSnapshot,
};
use crate::modules::memory::collector::{collect_memory, collect_process_memory};
use crate::modules::memory::delta_tracker::ProcessDeltaTracker;
use crate::modules::memory::model::RawProcessRow;
use crate::modules::memory::win32_api::own_working_set_mb;

/// Spawns the background thread and returns its receiver, latest slot, minimized flag, shutdown flag, wakeup callback, and thread handle.
#[allow(clippy::type_complexity)]
pub fn spawn_worker(
    wakeup: Option<Wakeup>,
    processes_tab_active: Arc<AtomicBool>,
) -> (
    mpsc::Receiver<Arc<SystemSnapshot>>,
    LatestSlot,
    Arc<AtomicBool>,
    Arc<AtomicBool>,
    Option<Wakeup>,
    thread::JoinHandle<()>,
) {
    let (tx, rx): (SnapshotTx, mpsc::Receiver<Arc<SystemSnapshot>>) = mpsc::sync_channel(2);
    let latest = LatestSlot::new();
    let is_minimized = Arc::new(AtomicBool::new(false));
    let shutdown = Arc::new(AtomicBool::new(false));
    let wakeup = wakeup.unwrap_or_else(|| Arc::new(|| {}));

    let latest_worker = latest.clone();
    let minimized_worker = is_minimized.clone();
    let shutdown_worker = shutdown.clone();
    let wakeup_worker = wakeup.clone();
    let tab_active_worker = processes_tab_active.clone();

    let handle = thread::spawn(move || {
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
            // Check for graceful shutdown signal
            if shutdown_worker.load(Ordering::Acquire) {
                break;
            }

            let tick_start = Instant::now();
            let uptime = System::uptime();
            let minimized = minimized_worker.load(Ordering::Relaxed);
            let wake = was_minimized && !minimized;
            let requested = take_process_scan_request();
            was_minimized = minimized;
            let processes_tab_active = tab_active_worker.load(Ordering::Relaxed);
            let refresh =
                should_scan_processes(minimized, tick, wake, requested, processes_tab_active);

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
            let interval =
                if minimized { WORKER_INTERVAL_MINIMIZED } else { WORKER_INTERVAL_VISIBLE };
            let mut sleep_overshoot_ms: u64 = 0;
            if elapsed < interval {
                let requested_sleep = interval - elapsed;
                let before_sleep = Instant::now();
                // Use park_timeout for instant wakeup via unpark()
                thread::park_timeout(requested_sleep);
                sleep_overshoot_ms =
                    before_sleep.elapsed().as_millis().saturating_sub(requested_sleep.as_millis())
                        as u64;
            }
            let os_scheduler_jitter_ms = sleep_overshoot_ms.min(u32::MAX as u64) as u32;

            let snapshot = Arc::new(SystemSnapshot::new_with_jitter(
                uptime,
                pulse_ram_mb,
                Some(memory),
                os_scheduler_jitter_ms,
            ));

            match publish(&tx, &latest_worker, snapshot, &wakeup_worker, minimized) {
                PublishResult::Disconnected => break,
                PublishResult::Sent | PublishResult::Full => {}
            }
            tick = tick.wrapping_add(1);
        }
    });

    (rx, latest, is_minimized, shutdown, Some(wakeup), handle)
}

/// Requests an immediate process scan on the next worker tick.
/// Also unparks the worker thread for instant wakeup (0 ms latency).
pub fn request_process_scan() {
    PROCESS_SCAN_REQUEST.store(true, Ordering::Relaxed);
}

pub(crate) fn take_process_scan_request() -> bool {
    PROCESS_SCAN_REQUEST.swap(false, Ordering::Relaxed)
}

static PROCESS_SCAN_REQUEST: AtomicBool = AtomicBool::new(false);

/// Visible mode scans every SCAN_TICKS_VISIBLE tick when Processes tab is active,
/// or every SCAN_TICKS_BACKGROUND when Overview tab is open.
/// Wake and action requests scan immediately. Minimized mode pauses full process scans entirely.
pub fn should_scan_processes(
    minimized: bool,
    tick: u64,
    wake: bool,
    requested: bool,
    processes_tab_active: bool,
) -> bool {
    !minimized
        && (wake
            || requested
            || tick % if processes_tab_active { SCAN_TICKS_VISIBLE } else { SCAN_TICKS_BACKGROUND }
                == 0)
}
