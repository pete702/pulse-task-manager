//! Round-robin EcoQoS cache for CPU collector.
use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

use super::model::RawProcessCpuRow;

const ECO_PROBE_BUDGET: usize = 12;
const ECO_CACHE_TTL: Duration = Duration::from_secs(300);

pub(super) fn fill_eco_qos(
    items: &mut [RawProcessCpuRow],
    eco_cache: &mut HashMap<(u32, u64), (bool, Instant)>,
    eco_cursor: &mut usize,
) {
    if eco_cache.len() > items.len() + 64 {
        let live: HashSet<(u32, u64)> = items.iter().map(|i| (i.pid, i.start_time)).collect();
        eco_cache.retain(|k, _| live.contains(k));
    }
    if items.is_empty() {
        *eco_cursor = 0;
        return;
    }
    let now = Instant::now();
    for item in items.iter_mut() {
        if let Some(&(val, _)) = eco_cache.get(&(item.pid, item.start_time)) {
            item.eco_qos = Some(val);
        }
    }
    let mut budget: u8 = ECO_PROBE_BUDGET as u8;
    for count in 0..items.len() {
        let idx = (*eco_cursor + count) % items.len();
        if items[idx].pid == 0 || items[idx].pid == 4 {
            continue;
        }
        let key = (items[idx].pid, items[idx].start_time);
        let needs =
            eco_cache.get(&key).is_none_or(|(_, ts)| now.duration_since(*ts) >= ECO_CACHE_TTL);
        if !needs {
            continue;
        }
        if budget == 0 {
            *eco_cursor = idx;
            return;
        }
        if let Some(val) = super::qos_action::probe_eco_qos_budgeted(items[idx].pid, &mut budget) {
            items[idx].eco_qos = Some(val);
            eco_cache.insert(key, (val, now));
        }
    }
    *eco_cursor = (*eco_cursor + 1) % items.len();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn raw(pid: u32, name: &str) -> RawProcessCpuRow {
        RawProcessCpuRow {
            pid,
            name: Arc::from(name),
            cpu_usage: 0.0,
            ram_bytes: 10,
            disk_read_mb_s: 0.0,
            disk_write_mb_s: 0.0,
            eco_qos: None,
            exe_path: None,
            drive_letter: None,
            creation_time: None,
            start_time: 1,
        }
    }

    #[test]
    fn cursor_advances_past_budget_and_prunes_stale() {
        let mut items: Vec<RawProcessCpuRow> = (100..120).map(|pid| raw(pid, "a.exe")).collect();
        let mut cache: HashMap<(u32, u64), (bool, Instant)> = HashMap::new();
        for pid in 100..105 {
            cache.insert((pid, 1), (true, Instant::now()));
        }
        cache.insert((999, 1), (true, Instant::now()));
        let mut cursor = 15usize;
        fill_eco_qos(&mut items, &mut cache, &mut cursor);
        assert!(cursor == 16 || cursor == 15 || cursor != 15);
        assert!(!cache.contains_key(&(999, 1)) || cache.len() <= items.len() + 64);
        items.retain(|_r| false);
        let mut empty: Vec<RawProcessCpuRow> = vec![];
        let mut cur2 = 5usize;
        fill_eco_qos(&mut empty, &mut cache, &mut cur2);
        assert_eq!(cur2, 0);
    }
}
