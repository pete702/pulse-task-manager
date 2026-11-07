//! RAM delta computation for CpuTableState.
use std::collections::HashMap;
use std::time::Instant;

use crate::modules::cpu::model::RawProcessCpuRow;

use super::CpuTableState;

impl CpuTableState {
    pub(crate) fn compute_ram_deltas(
        &mut self,
        rows: &[RawProcessCpuRow],
    ) -> HashMap<(u32, u64), i64> {
        let now = Instant::now();
        let mut deltas = HashMap::with_capacity(rows.len());
        for row in rows {
            let delta = self.ram_baseline.get(&(row.pid, row.start_time)).map_or(0, |prev| {
                let dt = now.saturating_duration_since(prev.1).as_secs_f32().max(0.25);
                let diff = row.ram_bytes as i64 - prev.0 as i64;
                (diff as f32 * (10.0 / dt)).round() as i64
            });
            deltas.insert((row.pid, row.start_time), delta);
        }
        self.ram_baseline =
            rows.iter().map(|row| ((row.pid, row.start_time), (row.ram_bytes, now))).collect();
        deltas
    }
}
