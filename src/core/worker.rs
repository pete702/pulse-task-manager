//! Background telemetry worker with decoupled RAM and process polling.

use crate::core::worker_io::Wakeup;
use crate::core::worker_loop::spawn_worker;

#[cfg(test)]
use crate::core::worker_io::{publish as io_publish, PublishResult as io_PublishResult};

/// Handle to the running background telemetry worker.
pub struct TelemetryWorker {
    rx: Option<std::sync::mpsc::Receiver<std::sync::Arc<crate::core::SystemSnapshot>>>,
    pub latest: crate::core::LatestSlot,
    pub is_minimized: std::sync::Arc<std::sync::atomic::AtomicBool>,
    /// True when the Processes sub-tab is active (drives 3s vs 5s scan cadence).
    pub processes_tab_active: std::sync::Arc<std::sync::atomic::AtomicBool>,
    #[allow(dead_code)]
    pub wakeup: Option<Wakeup>,
    /// Worker thread handle for instant wakeup via unpark().
    thread_handle: Option<std::thread::JoinHandle<()>>,
    /// Shutdown signal for graceful worker termination.
    shutdown: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl TelemetryWorker {
    /// Creates a new TelemetryWorker and extracts the snapshot receiver.
    pub fn new_with_rx(
        wakeup: Option<Wakeup>,
    ) -> (Self, std::sync::mpsc::Receiver<std::sync::Arc<crate::core::SystemSnapshot>>) {
        let processes_tab_active = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let (rx, latest, is_minimized, shutdown, wakeup, thread_handle) =
            spawn_worker(wakeup, processes_tab_active.clone());
        let mut worker = TelemetryWorker {
            rx: Some(rx),
            latest,
            is_minimized,
            processes_tab_active,
            wakeup,
            thread_handle: Some(thread_handle),
            shutdown,
        };
        let rx = worker.rx.take().unwrap();
        (worker, rx)
    }

    /// Requests an immediate process scan and unparks the worker thread for instant wakeup.
    pub fn request_process_scan(&self) {
        crate::core::worker_loop::request_process_scan();
        if let Some(h) = &self.thread_handle {
            h.thread().unpark();
        }
    }
}

impl Drop for TelemetryWorker {
    fn drop(&mut self) {
        // Signal shutdown
        self.shutdown.store(true, std::sync::atomic::Ordering::Release);
        // Unpark the worker thread so it wakes up and checks the shutdown flag
        if let Some(h) = &self.thread_handle {
            h.thread().unpark();
        }
        // Join the thread for clean shutdown
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
