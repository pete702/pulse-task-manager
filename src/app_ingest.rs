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
            let timestamp = snap
                .memory
                .as_ref()
                .map(|m| m.sample_elapsed_secs)
                .unwrap_or(snap.uptime_secs as f64);
            self.cpu_history.push(cpu.total_pct, cpu.hottest_core_pct, timestamp);
        }
        self.latest_snapshot = Some(snap);
    }

    pub(crate) fn collect_snapshots(&mut self) {
        if self.telemetry_paused {
            while self.snapshot_rx.try_recv().is_ok() {}
            let _ = self.latest_slot.take();
            return;
        }
        while let Ok(snap) = self.snapshot_rx.try_recv() {
            self.ingest_snapshot(snap);
        }
        if let Some(snap) = self.latest_slot.take() {
            self.ingest_snapshot(snap);
        }
    }
}
