//! Snapshot ingestion helpers for PulseApp, split from app.rs for the 280-line guard.
use std::sync::Arc;

use crate::app::PulseApp;
use crate::core::SystemSnapshot;

impl PulseApp {
    pub(crate) fn ingest_snapshot(&mut self, snap: Arc<SystemSnapshot>) {
        let is_new = self.latest_snapshot.as_ref().is_none_or(|prev| !Arc::ptr_eq(prev, &snap));
        if !is_new {
            return;
        }
        if let Some(ref mem) = snap.memory {
            self.memory_history.push(mem);
        }
        if let Some(ref cpu) = snap.cpu {
            // Single source: pause-adjusted timestamp_s from worker_loop.
            if let Some(ref mem) = snap.memory {
                self.cpu_history.push(cpu.total_pct, cpu.hottest_core_pct, mem.sample_elapsed_secs);
            }
        }
        self.latest_snapshot = Some(snap);
    }

    pub(crate) fn poll_snapshots(&mut self) {
        if self.telemetry_paused {
            while self.snapshot_rx.try_recv().is_ok() {}
            let _ = self.latest_slot.drain_all();
            return;
        }
        let mut batch: Vec<Arc<SystemSnapshot>> = Vec::new();
        while let Ok(snap) = self.snapshot_rx.try_recv() {
            batch.push(snap);
        }
        batch.extend(self.worker.drain_overflow_snapshots());
        batch.sort_by(|a, b| {
            let ka = a.memory.as_ref().map(|m| m.sample_elapsed_secs).unwrap_or(0.0);
            let kb = b.memory.as_ref().map(|m| m.sample_elapsed_secs).unwrap_or(0.0);
            ka.total_cmp(&kb)
        });
        for snap in batch {
            self.ingest_snapshot(snap);
        }
    }

    pub(crate) fn collect_snapshots(&mut self) {
        self.poll_snapshots();
    }
}
