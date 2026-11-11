//! Snapshot ingestion helpers for PulseApp, split from app.rs for the 280-line guard.
use std::sync::Arc;

use crate::app::PulseApp;
use crate::core::SystemSnapshot;

impl PulseApp {
    pub(crate) fn ingest_snapshot(&mut self, snap: &Arc<SystemSnapshot>) -> bool {
        let is_new = self.latest_snapshot.as_ref().is_none_or(|prev| {
            !Arc::ptr_eq(prev, snap)
                && !prev
                    .memory
                    .as_ref()
                    .zip(snap.memory.as_ref())
                    .is_some_and(|(a, b)| a.snapshot_id == b.snapshot_id)
        });
        if !is_new {
            return false;
        }
        if let Some(ref mem) = snap.memory {
            self.memory_history.push(mem);
            if let Some(ref cpu) = snap.cpu {
                self.cpu_history.push(cpu.total_pct, cpu.hottest_core_pct, mem.sample_elapsed_secs);
            }
        }
        crate::modules::storage::view::ingest_storage_snapshot(snap);
        crate::modules::network::view::ingest_network_snapshot(snap);
        self.latest_snapshot = Some(Arc::clone(snap));
        true
    }

    pub(crate) fn poll_snapshots(&mut self) -> bool {
        if self.telemetry_paused {
            while self.snapshot_rx.try_recv().is_ok() {}
            let _ = self.worker.drain_overflow_snapshots();
            return false;
        }
        let mut batch: Vec<Arc<SystemSnapshot>> = Vec::new();
        while let Ok(snap) = self.snapshot_rx.try_recv() {
            batch.push(snap);
        }
        batch.extend(self.worker.drain_overflow_snapshots());
        if batch.is_empty() {
            return false;
        }
        if batch.len() > 1 {
            batch.sort_by(|a, b| {
                let ka = a.memory.as_ref().map(|m| m.sample_elapsed_secs).unwrap_or(0.0);
                let kb = b.memory.as_ref().map(|m| m.sample_elapsed_secs).unwrap_or(0.0);
                ka.total_cmp(&kb)
            });
        }
        let mut ingested = false;
        for snap in batch {
            if self.ingest_snapshot(&snap) {
                ingested = true;
            }
        }
        ingested
    }

    #[allow(dead_code)]
    pub(crate) fn collect_snapshots(&mut self) {
        let _ = self.poll_snapshots();
    }
}
