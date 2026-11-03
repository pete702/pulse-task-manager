//! PULSE telemetry core: background worker, system snapshots, communication channels.

pub mod selftest;
pub mod selftest_app;
pub mod win32;
pub mod worker;

use std::sync::{mpsc, Arc, Mutex};

use crate::modules::memory::model::MemorySnapshot as MemSnapshot;

/// System state snapshot sent from background thread to UI.
/// Contains uptime, self-process RAM usage (MB), and OS memory snapshot.
#[derive(Debug, Clone, PartialEq)]
pub struct SystemSnapshot {
    /// System uptime in seconds (from `sysinfo::System::uptime()`).
    pub uptime_secs: u64,
    /// RAM usage of the PULSE process itself in megabytes.
    pub pulse_ram_mb: f32,
    /// OS-level memory snapshot (RAM/Swap totals, usage, cached).
    pub memory: Option<MemSnapshot>,
    /// OS scheduler jitter in milliseconds (how much the 1000ms sleep overshot).
    pub os_scheduler_jitter_ms: u32,
}

impl SystemSnapshot {
    /// Creates a new snapshot with the given uptime, process RAM, and memory.
    #[allow(dead_code)]
    #[inline]
    pub fn new(uptime_secs: u64, pulse_ram_mb: f32, memory: Option<MemSnapshot>) -> Self {
        Self { uptime_secs, pulse_ram_mb, memory, os_scheduler_jitter_ms: 0 }
    }

    /// Creates a new snapshot with OS scheduler jitter.
    #[inline]
    pub fn new_with_jitter(
        uptime_secs: u64,
        pulse_ram_mb: f32,
        memory: Option<MemSnapshot>,
        jitter_ms: u32,
    ) -> Self {
        Self { uptime_secs, pulse_ram_mb, memory, os_scheduler_jitter_ms: jitter_ms }
    }

    /// Creates a minimal snapshot (for backward compatibility / tests).
    #[allow(dead_code)]
    #[inline]
    pub fn minimal(uptime_secs: u64, pulse_ram_mb: f32) -> Self {
        Self { uptime_secs, pulse_ram_mb, memory: None, os_scheduler_jitter_ms: 0 }
    }
}

/// Channel type for snapshot transmission: `Sender` in background thread, `Receiver` in UI.
pub type SnapshotTx = mpsc::SyncSender<Arc<SystemSnapshot>>;

/// Shared slot for the latest snapshot (newest always wins).
pub type LatestSlot = Arc<Mutex<Option<Arc<SystemSnapshot>>>>;

pub use worker::TelemetryWorker;

/// Spawns the background telemetry worker thread.
/// Returns the worker handle (receiver, latest slot, minimized flag).
pub fn spawn_telemetry_worker() -> TelemetryWorker {
    worker::spawn()
}
