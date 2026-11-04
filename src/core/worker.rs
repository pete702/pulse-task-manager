//! Background telemetry worker with decoupled RAM and process polling.

use crate::core::worker_io::Wakeup;
use crate::core::worker_loop::spawn_worker;

#[cfg(test)]
use crate::core::worker_io::{publish as io_publish, PublishResult as io_PublishResult};

/// Handle to the running background telemetry worker.
pub struct TelemetryWorker {
    pub rx: std::sync::mpsc::Receiver<std::sync::Arc<crate::core::SystemSnapshot>>,
    pub latest: crate::core::LatestSlot,
    pub is_minimized: std::sync::Arc<std::sync::atomic::AtomicBool>,
    #[allow(dead_code)]
    pub wakeup: Option<Wakeup>,
}

/// Spawns the background thread and returns its receiver, latest slot, minimized flag, and wakeup callback.
pub fn spawn(wakeup: Option<Wakeup>) -> TelemetryWorker {
    let (rx, latest, is_minimized, wakeup) = spawn_worker(wakeup);
    TelemetryWorker { rx, latest, is_minimized, wakeup }
}

/// Spawns the background telemetry worker thread (no wakeup callback).
pub fn spawn_telemetry_worker() -> TelemetryWorker {
    spawn(None)
}

/// Spawns the background telemetry worker thread with a wakeup callback.
/// The callback is invoked after each snapshot is published to trigger UI repaint.
pub fn spawn_telemetry_worker_with_wakeup(wakeup: Wakeup) -> TelemetryWorker {
    spawn(Some(wakeup))
}

#[cfg(test)]
#[path = "worker_tests.rs"]
mod worker_tests;
