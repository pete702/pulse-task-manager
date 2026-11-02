//! PULSE telemetry core: background worker, system snapshots, communication channels.

pub mod worker;

use std::sync::mpsc;

/// System state snapshot sent from background thread to UI.
/// Currently contains only uptime; will be extended with memory/CPU metrics in v0.0.3+.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SystemSnapshot {
    /// System uptime in seconds (from `sysinfo::System::uptime()`).
    pub uptime_secs: u64,
}

impl SystemSnapshot {
    /// Creates a new snapshot with the given uptime.
    #[inline]
    pub fn new(uptime_secs: u64) -> Self {
        Self { uptime_secs }
    }
}

/// Channel type for snapshot transmission: `Sender` in background thread, `Receiver` in UI.
pub type SnapshotTx = mpsc::Sender<SystemSnapshot>;
pub type SnapshotRx = mpsc::Receiver<SystemSnapshot>;

/// Spawns the background telemetry worker thread.
/// Returns a `Receiver` for reading snapshots in the main thread.
pub fn spawn_telemetry_worker() -> SnapshotRx {
    worker::spawn()
}
