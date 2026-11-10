//! Round-robin EcoQoS cache for CPU collector.
use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

use super::model::RawProcessCpuRow;

const ECO_PROBE_BUDGET: usize = 12;
const ECO_CACHE_TTL: Duration = Duration::from_secs(300);

pub(crate) fn fill_eco_qos_with<F>(
    items: &mut [RawProcessCpuRow],
    eco_cache: &mut HashMap<(u32, u64), (bool, Instant)>,
    eco_cursor: &mut usize,
    mut probe_fn: F,
) where
    F: FnMut(u32, &mut u8) -> Option<bool>,
{
    if !eco_cache.is_empty() {
        let live: HashSet<(u32, u64)> = items.iter().map(|i| (i.pid, i.start_time)).collect();
        eco_cache.retain(|k, _| live.contains(k));
    }
    if items.is_empty() {
        *eco_cursor = 0;
        return;
    }
    let start = *eco_cursor;
    let now = Instant::now();
    for item in items.iter_mut() {
        if let Some(&(val, _)) = eco_cache.get(&(item.pid, item.start_time)) {
            item.eco_qos = Some(val);
        }
    }
    let mut budget: u8 = ECO_PROBE_BUDGET as u8;
    let mut max_attempts: u8 = 24;
    let mut probed: usize = 0;
    for count in 0..items.len() {
        let idx = (start + count) % items.len();
        let key = (items[idx].pid, items[idx].start_time);
        let needs =
            eco_cache.get(&key).is_none_or(|(_, ts)| now.duration_since(*ts) >= ECO_CACHE_TTL);
        if !needs {
            continue;
        }
        if probed >= ECO_PROBE_BUDGET || budget == 0 || max_attempts == 0 {
            *eco_cursor = (start + ECO_PROBE_BUDGET) % items.len();
            return;
        }
        max_attempts = max_attempts.saturating_sub(1);
        if let Some(val) = probe_fn(items[idx].pid, &mut budget) {
            items[idx].eco_qos = Some(val);
            eco_cache.insert(key, (val, now));
            probed += 1;
            if probed >= ECO_PROBE_BUDGET {
                *eco_cursor = (start + ECO_PROBE_BUDGET) % items.len();
                return;
            }
        }
    }
    *eco_cursor = (start + ECO_PROBE_BUDGET) % items.len();
}

pub(super) fn fill_eco_qos(
    items: &mut [RawProcessCpuRow],
    eco_cache: &mut HashMap<(u32, u64), (bool, Instant)>,
    eco_cursor: &mut usize,
) {
    fill_eco_qos_with(items, eco_cache, eco_cursor, super::qos_action::probe_eco_qos_budgeted);
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
            start_time: 100,
        }
    }

    #[test]
    fn round_robin_cursor_advances_and_prunes_dead_keys() {
        let mut items: Vec<RawProcessCpuRow> = (1..=30).map(|pid| raw(pid, "a.exe")).collect();
        let mut cache: HashMap<(u32, u64), (bool, Instant)> = HashMap::new();
        cache.insert((999, 1), (true, Instant::now()));
        let mut cursor = 0usize;
        let probe = |pid: u32, budget: &mut u8| -> Option<bool> {
            if *budget == 0 {
                return None;
            }
            *budget -= 1;
            Some(pid.is_multiple_of(2))
        };
        fill_eco_qos_with(&mut items, &mut cache, &mut cursor, probe);
        assert_eq!(cursor, 12);
        assert_eq!(items.iter().filter(|r| r.eco_qos.is_some()).count(), 12);
        assert!(!cache.contains_key(&(999, 1)));
        for r in &mut items {
            r.eco_qos = None;
        }
        fill_eco_qos_with(&mut items, &mut cache, &mut cursor, probe);
        assert_eq!(cursor, 24);
        assert_eq!(items.iter().filter(|r| r.eco_qos.is_some()).count(), 24);
        for r in &mut items {
            r.eco_qos = None;
        }
        fill_eco_qos_with(&mut items, &mut cache, &mut cursor, probe);
        assert_eq!(cursor, 6);
        assert!(items.iter().all(|r| r.eco_qos.is_some()));
        assert_eq!(items[0].eco_qos, Some(false));
        assert_eq!(items[1].eco_qos, Some(true));
    }

    #[test]
    fn probe_attempts_capped_when_all_none() {
        let mut items: Vec<RawProcessCpuRow> = (1..=100).map(|pid| raw(pid, "a.exe")).collect();
        let mut cache: HashMap<(u32, u64), (bool, Instant)> = HashMap::new();
        let mut cursor = 0usize;
        let mut calls = 0u32;
        let probe = |_pid: u32, _budget: &mut u8| -> Option<bool> {
            calls += 1;
            None
        };
        fill_eco_qos_with(&mut items, &mut cache, &mut cursor, probe);
        assert!(calls <= 24, "probe_fn called {calls} times");
        assert!(items.iter().all(|r| r.eco_qos.is_none()));
    }
}
