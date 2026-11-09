//! RAM delta computation for CpuTableState.
use std::collections::{HashMap, HashSet};
use std::time::Instant;

use crate::modules::cpu::model::RawProcessCpuRow;

use super::CpuTableState;

impl CpuTableState {
    pub(crate) fn compute_ram_deltas(
        &mut self,
        rows: &[RawProcessCpuRow],
        now: Instant,
        scan_changed: bool,
        scan_generation: u64,
    ) -> HashMap<(u32, u64), i64> {
        if rows.is_empty() {
            self.ram_baseline.clear();
        }
        if self.ram_baseline.len() > rows.len() + 32 {
            let live: HashSet<(u32, u64)> = rows.iter().map(|r| (r.pid, r.start_time)).collect();
            self.ram_baseline.retain(|k, _| live.contains(k));
        }
        if !scan_changed {
            let mut deltas = HashMap::with_capacity(rows.len());
            for row in rows {
                let delta =
                    self.ram_baseline.get(&(row.pid, row.start_time)).map_or(0, |prev| prev.2);
                deltas.insert((row.pid, row.start_time), delta);
            }
            return deltas;
        }
        let mut deltas = HashMap::with_capacity(rows.len());
        for row in rows {
            let key = (row.pid, row.start_time);
            let delta = self.ram_baseline.get(&key).map_or(0, |prev| {
                let raw_dt = now.saturating_duration_since(prev.1).as_secs_f32();
                if raw_dt > 15.0 {
                    0
                } else {
                    let dt = raw_dt.max(1.0);
                    let scale = (10.0 / dt).clamp(1.0 / 30.0, 10.0);
                    let diff = row.ram_bytes as i64 - prev.0 as i64;
                    ((diff as f32) * scale).round() as i64
                }
            });
            deltas.insert(key, delta);
        }
        self.ram_baseline = rows
            .iter()
            .map(|row| {
                let delta = deltas.get(&(row.pid, row.start_time)).copied().unwrap_or(0);
                ((row.pid, row.start_time), (row.ram_bytes, now, delta))
            })
            .collect();
        self.baseline_scan_gen = scan_generation;
        deltas
    }
    pub fn spawn_menu_action(&mut self, job: impl FnOnce() -> super::ActionMsg + Send + 'static) {
        if self.action_rx.is_some() {
            return;
        }
        self.last_action_msg = Some(("Working...".to_owned(), Instant::now()));
        let (tx, rx) = std::sync::mpsc::channel();
        self.action_rx = Some(rx);
        std::thread::spawn(move || {
            let _ = tx.send(job());
        });
    }
    pub fn poll_menu_action(&mut self) {
        if let Some(rx) = self.action_rx.take() {
            match rx.try_recv() {
                Ok((msg, eco, rollbacks)) => {
                    if let Some((pid, st, on)) = eco {
                        if self.eco_overrides.len() >= 512 {
                            self.eco_overrides.clear();
                        }
                        self.eco_overrides.insert((pid, st), on);
                        crate::modules::cpu::collector::mark_pending_eco(pid, st, on);
                        crate::core::request_process_scan();
                    }
                    if !rollbacks.is_empty() {
                        self.apply_eco_rollbacks(&rollbacks);
                    }
                    self.last_action_msg = Some((msg, Instant::now()));
                    self.dirty = true;
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    self.action_rx = Some(rx);
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.action_rx = None;
                    self.last_action_msg =
                        Some(("Action failed (worker disconnected)".to_owned(), Instant::now()));
                }
            }
        }
    }
}
