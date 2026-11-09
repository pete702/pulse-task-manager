use crate::core::{
    worker_io::{publish, PublishResult, Wakeup},
    worker_policy::*,
    LatestSlot, SnapshotTx, SystemSnapshot,
};
use crate::modules::cpu::collector::{collect_cpu, warmup_cpu, CpuCollector};
use crate::modules::cpu::model::{CpuSnapshot, RawProcessCpuRow};
use crate::modules::cpu::win32_cpu::init_cpu_brand_and_base_mhz;
use crate::modules::memory::collector::{collect_memory, collect_process_memory};
use crate::modules::memory::delta_tracker::ProcessDeltaTracker;
use crate::modules::memory::model::RawProcessRow;
use crate::modules::memory::win32_api::own_working_set_mb;
use crate::modules::network::collector::NetworkCollector;
use crate::modules::storage::collector::StorageCollector;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Mutex, OnceLock};
use std::thread;
use std::time::{Duration, Instant};
use sysinfo::System;
#[derive(Debug, Clone, Copy)]
pub struct WorkerConfig {
    pub paused: bool,
}
static WORKER_PAUSED: AtomicBool = AtomicBool::new(false);
pub fn set_worker_paused(cfg: WorkerConfig) {
    if WORKER_PAUSED.swap(cfg.paused, Ordering::SeqCst) && !cfg.paused {
        if let Some(slot) = WORKER_THREAD.get() {
            let guard = slot.lock().unwrap_or_else(|e| e.into_inner());
            if let Some(t) = guard.as_ref() {
                t.unpark();
            }
        }
    }
}
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
    WORKER_ALIVE.store(true, Ordering::SeqCst);
    let handle = match thread::Builder::new().name("pulse-worker".into()).spawn(move || {
        let _exit_guard = WorkerExitGuard;
        {
            let slot = WORKER_THREAD.get_or_init(|| Mutex::new(None));
            *slot.lock().unwrap_or_else(|e| e.into_inner()) = Some(thread::current());
        }
        let started = Instant::now();
        let mut sys = System::new();
        warmup_cpu(&mut sys);
        std::thread::sleep(Duration::from_millis(120));
        init_cpu_brand_and_base_mhz(&mut sys);
        let mut latest_spike = None;
        let mut delta_tracker = ProcessDeltaTracker::new();
        let mut cpu_collector = CpuCollector::new();
        let mut storage_collector = StorageCollector::new();
        let mut network_collector = NetworkCollector::new();
        let mut last_processes: Arc<Vec<RawProcessRow>> = Arc::new(Vec::new());
        let mut last_cpu_raw_rows: Arc<[RawProcessCpuRow]> = Arc::from(Vec::new());
        let mut last_heavy = false;
        let mut was_minimized = false;
        let mut tick: u64 = 0;
        let mut process_scan_id: u64 = 0;
        let mut paused_accum = Duration::ZERO;
        let mut last_cpu_tick = Instant::now() - Duration::from_secs(2);
        let mut last_cpu: Option<CpuSnapshot> = None;
        let mut last_timestamp_s = 0.0;
        let mut collector_primed = false;
        loop {
            if shutdown_worker.load(Ordering::Acquire) {
                break;
            }
            if WORKER_PAUSED.load(Ordering::Relaxed) {
                let pause_start = Instant::now();
                thread::park_timeout(Duration::from_millis(1000));
                paused_accum = paused_accum.saturating_add(pause_start.elapsed());
                continue;
            }
            let tick_start = Instant::now();
            let minimized = minimized_worker.load(Ordering::Relaxed);
            let wake = was_minimized && !minimized;
            let requested = take_process_scan_request();
            was_minimized = minimized;
            let refresh = should_scan_processes(
                minimized,
                tick,
                wake,
                requested,
                tab_active_worker.load(Ordering::Relaxed),
            );
            let do_refresh = refresh || (!minimized && tick == 0);
            let candidate_s = started.elapsed().saturating_sub(paused_accum).as_secs_f64();
            let due = last_cpu_tick.elapsed() >= Duration::from_millis(450);
            if due || last_cpu.is_none() {
                last_timestamp_s = candidate_s;
            }
            let timestamp_s = last_timestamp_s;
            let mut cpu = last_cpu.clone().unwrap_or_default();
            if due || last_cpu.is_none() {
                cpu = collect_cpu(&mut sys);
                last_cpu_tick = Instant::now();
                last_cpu = Some(cpu.clone());
            }
            let logical_cores = cpu.logical_cores;
            let mut memory = collect_memory(&mut sys);
            let mut scanned = false;
            if do_refresh {
                let cpu_raw_rows = cpu_collector.collect_process_cpu(&mut sys, logical_cores);
                if !cpu_raw_rows.is_empty() {
                    collector_primed = true;
                }
                if !cpu_raw_rows.is_empty() || !collector_primed {
                    scanned = true;
                    process_scan_id += 1;
                    cpu.raw_processes = Arc::from(cpu_raw_rows);
                    last_cpu_raw_rows = Arc::clone(&cpu.raw_processes);
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
                }
            }
            if !scanned {
                cpu.raw_processes = Arc::clone(&last_cpu_raw_rows);
                memory.processes = Arc::clone(&last_processes);
                memory.has_heavy_process = last_heavy;
                if memory.ram_usage_percent() < HIGH_RAM_PRESSURE_PCT {
                    latest_spike = None;
                }
            }
            cpu.process_count = cpu.raw_processes.len();
            cpu.total_pct = cpu.total_pct.max(0.0);
            cpu.hottest_core_pct = cpu.hottest_core_pct.max(0.0);
            memory.latest_spike.clone_from(&latest_spike);
            memory.scan_duration_ms = tick_start.elapsed().as_secs_f32() * 1000.0;
            memory.snapshot_id = tick + 1;
            memory.process_scan_id = process_scan_id;
            memory.sample_elapsed_secs = timestamp_s;
            memory.channel_overflows = latest_worker.overflow_count();
            let elapsed = tick_start.elapsed();
            let requested_sleep =
                WORKER_INTERVAL_VISIBLE.saturating_sub(elapsed).max(Duration::from_millis(50));
            let before_sleep = Instant::now();
            thread::park_timeout(requested_sleep);
            let over =
                before_sleep.elapsed().as_millis().saturating_sub(requested_sleep.as_millis())
                    as u64;
            let os_scheduler_jitter_ms = over.min(u32::MAX as u64) as u32;
            let storage = storage_collector.collect(&cpu.raw_processes, process_scan_id);
            let network = network_collector.collect();
            let mut snapshot_obj = SystemSnapshot::new_with_jitter(
                System::uptime(),
                own_working_set_mb(),
                Some(memory),
                Some(cpu),
                os_scheduler_jitter_ms,
            );
            snapshot_obj.storage = storage;
            snapshot_obj.network = network;
            let snapshot = Arc::new(snapshot_obj);
            match publish(
                &tx,
                &latest_worker,
                snapshot,
                &wakeup_worker,
                minimized_worker.load(Ordering::Relaxed),
            ) {
                PublishResult::Disconnected => break,
                PublishResult::Sent | PublishResult::Full => {}
            }
            tick = tick.wrapping_add(1);
        }
    }) {
        Ok(h) => h,
        Err(_) => {
            WORKER_ALIVE.store(false, Ordering::SeqCst);
            thread::Builder::new().spawn(|| {}).unwrap_or_else(|_| thread::spawn(|| {}))
        }
    };
    (rx, latest, is_minimized, shutdown, Some(wakeup), handle)
}
pub fn request_process_scan() -> bool {
    if !WORKER_ALIVE.load(Ordering::Relaxed) {
        return false;
    }
    PROCESS_SCAN_REQUEST.store(true, Ordering::Relaxed);
    if let Some(slot) = WORKER_THREAD.get() {
        let guard = slot.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(thread) = guard.as_ref() {
            thread.unpark();
            return true;
        }
    }
    false
}
pub(crate) fn take_process_scan_request() -> bool {
    PROCESS_SCAN_REQUEST.swap(false, Ordering::Relaxed)
}
static PROCESS_SCAN_REQUEST: AtomicBool = AtomicBool::new(false);
static WORKER_THREAD: OnceLock<Mutex<Option<thread::Thread>>> = OnceLock::new();
static WORKER_ALIVE: AtomicBool = AtomicBool::new(false);
struct WorkerExitGuard;
impl Drop for WorkerExitGuard {
    fn drop(&mut self) {
        clear_thread_slot();
        PROCESS_SCAN_REQUEST.store(false, Ordering::SeqCst);
        WORKER_ALIVE.store(false, Ordering::SeqCst);
    }
}
fn clear_thread_slot() {
    if let Some(slot) = WORKER_THREAD.get() {
        *slot.lock().unwrap_or_else(|e| e.into_inner()) = None;
    }
    WORKER_ALIVE.store(false, Ordering::Relaxed);
}
