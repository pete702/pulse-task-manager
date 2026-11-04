//! PULSE telemetry core: background worker, system snapshots, communication channels.

pub mod selftest;
pub mod selftest_app;
pub mod win32;
pub mod worker;
pub mod worker_io;
pub mod worker_loop;
pub mod worker_policy;

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{mpsc, Arc, Mutex};

use crate::modules::memory::model::MemorySnapshot as MemSnapshot;

/// System state snapshot sent from background thread to UI.
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
    #[allow(dead_code)]
    #[inline]
    pub fn new(uptime_secs: u64, pulse_ram_mb: f32, memory: Option<MemSnapshot>) -> Self {
        Self { uptime_secs, pulse_ram_mb, memory, os_scheduler_jitter_ms: 0 }
    }

    #[inline]
    pub fn new_with_jitter(
        uptime_secs: u64,
        pulse_ram_mb: f32,
        memory: Option<MemSnapshot>,
        jitter_ms: u32,
    ) -> Self {
        Self { uptime_secs, pulse_ram_mb, memory, os_scheduler_jitter_ms: jitter_ms }
    }

    #[allow(dead_code)]
    #[inline]
    pub fn minimal(uptime_secs: u64, pulse_ram_mb: f32) -> Self {
        Self { uptime_secs, pulse_ram_mb, memory: None, os_scheduler_jitter_ms: 0 }
    }
}

/// Channel type for snapshot transmission: `Sender` in background thread, `Receiver` in UI.
pub type SnapshotTx = mpsc::SyncSender<Arc<SystemSnapshot>>;

/// Newest-snapshot slot. Tracks how many unread values were overwritten.
#[derive(Default)]
pub struct LatestSlot {
    inner: Arc<Mutex<Option<Arc<SystemSnapshot>>>>,
    overflows: Arc<AtomicU64>,
}

impl Clone for LatestSlot {
    fn clone(&self) -> Self {
        Self { inner: Arc::clone(&self.inner), overflows: Arc::clone(&self.overflows) }
    }
}

impl LatestSlot {
    pub fn new() -> Self {
        Self::default()
    }

    /// Replaces the slot contents and counts an overflow when a value was unread.
    pub fn store(&self, value: Arc<SystemSnapshot>) {
        if let Ok(mut slot) = self.inner.lock() {
            if slot.replace(value).is_some() {
                self.overflows.fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    /// Clones the newest value without clearing it.
    #[allow(dead_code)]
    pub fn get(&self) -> Option<Arc<SystemSnapshot>> {
        self.inner.lock().ok().and_then(|slot| slot.clone())
    }

    /// Takes the newest value, marking it as read.
    pub fn take(&self) -> Option<Arc<SystemSnapshot>> {
        self.inner.try_lock().ok().and_then(|mut slot| slot.take())
    }

    pub fn overflow_count(&self) -> u64 {
        self.overflows.load(Ordering::Relaxed)
    }
}

pub use worker::{spawn_telemetry_worker, spawn_telemetry_worker_with_wakeup, TelemetryWorker};
pub use worker_io::Wakeup;
pub use worker_loop::request_process_scan;
