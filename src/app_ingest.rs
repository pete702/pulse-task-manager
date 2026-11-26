//! Snapshot ingestion helpers for PulseApp, split from app.rs for the 280-line guard.
use std::sync::Arc;

use crate::app::PulseApp;
use crate::core::SystemSnapshot;
use crate::ui::system_gpu_tab::system_gpu_chart as gpu_chart;

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
        crate::modules::storage::view::ingest_storage_snapshot(&mut self.storage_view_state, snap);
        crate::modules::network::view::ingest_network_snapshot(&mut self.network_view_state, snap);
        let adapters = crate::ui::system_gpu_tab::background_adapters();
        // DXGI VRAM is recorded independently of PDH engine counters: a failed
        // GPU engine query yields `None` there, never a dropped VRAM sample.
        let vram_pct = adapters
            .iter()
            .any(|a| a.dedicated_limit_bytes > 0)
            .then(|| gpu_chart::vram_usage_percent(&adapters));
        let gpu_pct = snap.gpu.as_ref().and_then(|g| gpu_chart::total_gpu_pct(g, &adapters));
        let snapshot_id = snap.memory.as_ref().map_or(0, |m| m.snapshot_id);
        gpu_chart::push_sample(snap.uptime_secs, snapshot_id, gpu_pct, vram_pct);
        self.latest_snapshot = Some(Arc::clone(snap));
        true
    }

    pub(crate) fn poll_snapshots(&mut self) -> bool {
        if self.telemetry_paused {
            let mut batch: Vec<Arc<SystemSnapshot>> = Vec::new();
            while let Ok(snap) = self.snapshot_rx.try_recv() {
                batch.push(snap);
            }
            batch.extend(self.worker.drain_overflow_snapshots());
            if !batch.is_empty() {
                if batch.len() > 1 {
                    batch.sort_by(|a, b| {
                        let ka = a.memory.as_ref().map(|m| m.sample_elapsed_secs).unwrap_or(0.0);
                        let kb = b.memory.as_ref().map(|m| m.sample_elapsed_secs).unwrap_or(0.0);
                        ka.total_cmp(&kb)
                    });
                }
                if let Some(newest) = batch.into_iter().last() {
                    self.paused_pending_snapshot = Some(newest);
                }
            }
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
}
