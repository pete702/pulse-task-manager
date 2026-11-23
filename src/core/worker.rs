//! Background telemetry worker with decoupled RAM and process polling.

use crate::core::worker_io::Wakeup;
use crate::core::worker_loop::{spawn_worker, SpawnedWorker};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

/// Per-instance control state shared with exactly one worker thread.
#[derive(Default)]
pub struct WorkerControl {
    pub alive: AtomicBool,
    pub paused: AtomicBool,
    pub scan_requested: AtomicBool,
    forced_unpark: AtomicBool,
    thread: Mutex<Option<std::thread::Thread>>,
}

impl WorkerControl {
    pub fn set_paused(&self, paused: bool) {
        let was = self.paused.swap(paused, Ordering::SeqCst);
        if was && !paused {
            self.unpark();
        }
    }
    #[cfg(test)]
    pub fn is_paused(&self) -> bool {
        self.paused.load(Ordering::SeqCst)
    }
    pub fn request_scan(&self) -> bool {
        if !self.alive.load(Ordering::Relaxed) {
            return false;
        }
        self.scan_requested.store(true, Ordering::Relaxed);
        self.unpark()
    }
    pub fn take_scan_request(&self) -> bool {
        self.scan_requested.swap(false, Ordering::Relaxed)
    }
    pub fn take_forced_unpark(&self) -> bool {
        self.forced_unpark.swap(false, Ordering::Relaxed)
    }
    pub fn set_thread(&self, thread: std::thread::Thread) {
        *self.thread.lock().unwrap_or_else(|e| e.into_inner()) = Some(thread);
    }
    fn clear_thread(&self) {
        *self.thread.lock().unwrap_or_else(|e| e.into_inner()) = None;
    }
    fn unpark(&self) -> bool {
        if let Some(thread) = self.thread.lock().unwrap_or_else(|e| e.into_inner()).as_ref() {
            self.forced_unpark.store(true, Ordering::Relaxed);
            thread.unpark();
            true
        } else {
            false
        }
    }
    pub fn exit_cleanup(&self) {
        self.clear_thread();
        self.scan_requested.store(false, Ordering::SeqCst);
        self.alive.store(false, Ordering::SeqCst);
    }
}

/// Handle to the running background telemetry worker.
pub struct TelemetryWorker {
    rx: Option<std::sync::mpsc::Receiver<std::sync::Arc<crate::core::SystemSnapshot>>>,
    pub latest: crate::core::LatestSlot,
    pub is_minimized: Arc<AtomicBool>,
    /// True when the Processes sub-tab is active (drives 3s vs 5s scan cadence).
    pub processes_tab_active: Arc<AtomicBool>,
    /// Worker thread handle for instant wakeup via unpark().
    thread_handle: Option<std::thread::JoinHandle<()>>,
    /// Shutdown signal for graceful worker termination.
    shutdown: Arc<AtomicBool>,
    /// Per-instance control shared with this worker's thread.
    control: Arc<WorkerControl>,
}

impl TelemetryWorker {
    /// Creates a new TelemetryWorker and extracts the snapshot receiver.
    pub fn new_with_rx(
        wakeup: Option<Wakeup>,
    ) -> (Self, std::sync::mpsc::Receiver<std::sync::Arc<crate::core::SystemSnapshot>>) {
        let processes_tab_active = Arc::new(AtomicBool::new(false));
        let spawned: SpawnedWorker = spawn_worker(wakeup, processes_tab_active.clone());
        let control = Arc::clone(&spawned.control);
        let mut worker = TelemetryWorker {
            rx: Some(spawned.rx),
            latest: spawned.latest,
            is_minimized: spawned.is_minimized,
            processes_tab_active,
            thread_handle: spawned.handle,
            shutdown: spawned.shutdown,
            control,
        };
        let rx = worker.rx.take().expect("worker rx initialized");
        (worker, rx)
    }

    /// Enables the 3-second process cadence while the Processes tab is visible.
    pub fn set_fast_process_cadence(&self, fast: bool) {
        self.processes_tab_active.store(fast, Ordering::Relaxed);
    }

    /// Sets minimized watchdog state for the worker loop.
    pub fn set_minimized(&self, min: bool) {
        self.is_minimized.store(min, Ordering::Relaxed);
    }

    /// Pauses/resumes this worker only, without touching another instance.
    pub fn set_paused(&self, paused: bool) {
        self.control.set_paused(paused);
    }

    /// Requests an immediate process scan and unparks this worker's thread.
    pub fn request_process_scan(&self) -> bool {
        self.control.request_scan()
    }

    /// Drains all queued overflow snapshots in chronological FIFO order.
    pub fn drain_overflow_snapshots(&self) -> Vec<std::sync::Arc<crate::core::SystemSnapshot>> {
        self.latest.drain_all()
    }
}

impl Drop for TelemetryWorker {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::Release);
        if let Some(h) = &self.thread_handle {
            h.thread().unpark();
        }
        if let Some(h) = self.thread_handle.take() {
            let _ = h.join();
        }
    }
}

/// Spawns the background telemetry worker thread (no wakeup callback).
pub fn spawn_telemetry_worker(
) -> (TelemetryWorker, std::sync::mpsc::Receiver<std::sync::Arc<crate::core::SystemSnapshot>>) {
    TelemetryWorker::new_with_rx(None)
}

/// Spawns the background telemetry worker thread with a wakeup callback.
/// The callback is invoked after each snapshot is published to trigger UI repaint.
pub fn spawn_telemetry_worker_with_wakeup(
    wakeup: Wakeup,
) -> (TelemetryWorker, std::sync::mpsc::Receiver<std::sync::Arc<crate::core::SystemSnapshot>>) {
    TelemetryWorker::new_with_rx(Some(wakeup))
}

#[cfg(test)]
#[path = "worker_tests.rs"]
mod worker_tests;

#[cfg(test)]
mod control_tests {
    use super::*;

    #[test]
    fn worker_controls_are_isolated_per_instance() {
        let a = Arc::new(WorkerControl::default());
        let b = Arc::new(WorkerControl::default());
        a.alive.store(true, Ordering::SeqCst);
        b.alive.store(true, Ordering::SeqCst);
        a.set_paused(true);
        assert!(a.is_paused());
        assert!(!b.is_paused());
        a.scan_requested.store(true, Ordering::Relaxed);
        assert!(a.take_scan_request());
        assert!(!b.take_scan_request());
        assert!(!a.take_scan_request());
        a.exit_cleanup();
        assert!(!a.alive.load(Ordering::SeqCst));
        assert!(b.alive.load(Ordering::SeqCst));
    }

    #[test]
    fn spawned_workers_have_distinct_controls() {
        let (w1, _rx1) = spawn_telemetry_worker();
        let (w2, _rx2) = spawn_telemetry_worker();
        assert!(!Arc::ptr_eq(&w1.control, &w2.control));
    }
}
