//! PULSE telemetry core: background worker, system snapshots, communication channels.

pub mod worker;

use std::sync::mpsc;

/// System state snapshot sent from background thread to UI.
/// Contains uptime and self-process RAM usage (MB).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SystemSnapshot {
    /// System uptime in seconds (from `sysinfo::System::uptime()`).
    pub uptime_secs: u64,
    /// RAM usage of the PULSE process itself in megabytes.
    pub pulse_ram_mb: f32,
}

impl SystemSnapshot {
    /// Creates a new snapshot with the given uptime and process RAM.
    #[inline]
    pub fn new(uptime_secs: u64, pulse_ram_mb: f32) -> Self {
        Self {
            uptime_secs,
            pulse_ram_mb,
        }
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
