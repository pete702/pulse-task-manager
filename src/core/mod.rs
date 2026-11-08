//! PULSE telemetry core: background worker, system snapshots, communication channels.

pub mod persist;
pub mod selftest;
pub mod selftest_app;
pub mod win32;
pub mod worker;
pub mod worker_io;
pub mod worker_loop;
pub mod worker_policy;

use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{mpsc, Arc, Mutex};

use crate::modules::cpu::model::CpuSnapshot;
use crate::modules::memory::model::MemorySnapshot as MemSnapshot;
use crate::modules::network::model::NetworkSnapshot;
use crate::modules::storage::model::StorageSnapshot;

/// System state snapshot sent from background thread to UI.
#[derive(Debug, Clone, PartialEq)]
pub struct SystemSnapshot {
    /// System uptime in seconds (from `sysinfo::System::uptime()`).
    pub uptime_secs: u64,
    /// RAM usage of the PULSE process itself in megabytes.
    pub pulse_ram_mb: f32,
    /// OS-level memory snapshot (RAM/Swap totals, usage, cached).
    pub memory: Option<MemSnapshot>,
    /// OS-level CPU snapshot (total %, per-core %, hottest core, frequency, processes).
    pub cpu: Option<CpuSnapshot>,
    /// OS-level storage snapshot (volumes, I/O rates, spike).
    pub storage: StorageSnapshot,
    /// OS-level network snapshot (adapters, rates, esports grade).
    pub network: NetworkSnapshot,
    /// OS scheduler jitter in milliseconds (how much the 1000ms sleep overshot).
    pub os_scheduler_jitter_ms: u32,
}

impl SystemSnapshot {
    #[cfg(test)]
    #[inline]
    pub fn new(uptime_secs: u64, pulse_ram_mb: f32, memory: Option<MemSnapshot>) -> Self {
        Self {
            uptime_secs,
            pulse_ram_mb,
            memory,
            cpu: None,
            storage: StorageSnapshot::default(),
            network: NetworkSnapshot::default(),
            os_scheduler_jitter_ms: 0,
        }
    }

    #[inline]
    pub fn new_with_jitter(
        uptime_secs: u64,
        pulse_ram_mb: f32,
        memory: Option<MemSnapshot>,
        cpu: Option<CpuSnapshot>,
        jitter_ms: u32,
    ) -> Self {
        Self {
            uptime_secs,
            pulse_ram_mb,
            memory,
            cpu,
            storage: StorageSnapshot::default(),
            network: NetworkSnapshot::default(),
            os_scheduler_jitter_ms: jitter_ms,
        }
    }
}

/// Channel type for snapshot transmission: `Sender` in background thread, `Receiver` in UI.
pub type SnapshotTx = mpsc::SyncSender<Arc<SystemSnapshot>>;

/// FIFO ring buffer for queued snapshots. Tracks how many unread values were overwritten.
#[derive(Default)]
pub struct LatestSlot {
    inner: Arc<Mutex<VecDeque<Arc<SystemSnapshot>>>>,
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

    /// Pushes a snapshot to the FIFO queue and counts an overflow when unread values exist.
    pub fn store(&self, value: Arc<SystemSnapshot>) {
        let mut queue = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        if !queue.is_empty() {
            self.overflows.fetch_add(1, Ordering::Relaxed);
        }
        if let Some(prev) = queue.back_mut() {
            if Arc::strong_count(prev) == 1 {
                let prev_mut = Arc::make_mut(prev);
                if let Some(mem) = &mut prev_mut.memory {
                    mem.processes = Arc::new(Vec::new());
                }
                if let Some(cpu) = &mut prev_mut.cpu {
                    cpu.raw_processes = Arc::from(&[][..]);
                }
            }
        }
        if queue.len() >= 960 {
            queue.pop_front();
        }
        queue.push_back(value);
    }

    /// Clones the newest value without clearing it.
    #[cfg(test)]
    pub fn get(&self) -> Option<Arc<SystemSnapshot>> {
        self.inner.lock().ok().and_then(|queue| queue.back().cloned())
    }

    /// Takes the newest value, marking it as read and clearing the queue.
    #[cfg(test)]
    pub fn take(&self) -> Option<Arc<SystemSnapshot>> {
        let mut queue = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        let last = queue.pop_back();
        queue.clear();
        last
    }

    /// Drains all queued snapshots in chronological FIFO order.
    pub fn drain_all(&self) -> Vec<Arc<SystemSnapshot>> {
        let mut queue = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        self.overflows.store(0, Ordering::Relaxed);
        queue.drain(..).collect()
    }

    pub fn overflow_count(&self) -> u64 {
        self.overflows.load(Ordering::Relaxed)
    }
}

pub use worker::{spawn_telemetry_worker, spawn_telemetry_worker_with_wakeup, TelemetryWorker};
pub use worker_io::Wakeup;
pub use worker_loop::request_process_scan;
