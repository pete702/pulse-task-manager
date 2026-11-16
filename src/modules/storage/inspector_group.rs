//! Grouped disk consumers with ABA-safe cache for storage inspector.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Instant;

use crate::modules::cpu::model::RawProcessCpuRow;

#[derive(Clone)]
pub struct Grouped {
    pub name: String,
    pub read: f32,
    pub write: f32,
    pub ram_bytes: u64,
    pub count: usize,
    pub total: f32,
    pub drive: Option<char>,
}

type CacheKey = (u64, bool, u64);
type ConsumerCache = (CacheKey, Arc<Vec<Grouped>>);
static CONSUMER_CACHE: OnceLock<Mutex<ConsumerCache>> = OnceLock::new();
static LAST_SCAN: OnceLock<Mutex<Option<Instant>>> = OnceLock::new();
static FILTER_ON: OnceLock<Mutex<bool>> = OnceLock::new();

fn consumer_cache() -> &'static Mutex<ConsumerCache> {
    CONSUMER_CACHE.get_or_init(|| Mutex::new(((0, false, 0), Arc::new(Vec::new()))))
}

pub fn last_scan() -> &'static Mutex<Option<Instant>> {
    LAST_SCAN.get_or_init(|| Mutex::new(None))
}

pub fn filter_on() -> &'static Mutex<bool> {
    // Default to All Drives so the table is not over-filtered on first view.
    FILTER_ON.get_or_init(|| Mutex::new(false))
}

fn drive_mask(filter: Option<&[char]>) -> u64 {
    match filter {
        None => 0,
        Some(letters) => {
            let mut mask = 0u64;
            for c in letters {
                let u = *c as u8;
                if u.is_ascii_uppercase() {
                    mask |= 1u64 << (u - b'A');
                } else if u.is_ascii_lowercase() {
                    mask |= 1u64 << (u.to_ascii_uppercase() - b'A');
                }
            }
            mask
        }
    }
}

fn cache_key(scan_id: u64, filter: Option<&[char]>) -> CacheKey {
    (scan_id, filter.is_some(), drive_mask(filter))
}

/// Returns cached top-8 grouped consumers; ABA-safe key prevents stale hits.
/// `filter` is the set of drive letters for the selected disk (None = all).
pub fn cached_consumers(
    scan_id: u64,
    raw: &[RawProcessCpuRow],
    filter: Option<&[char]>,
) -> Arc<Vec<Grouped>> {
    let key = cache_key(scan_id, filter);
    {
        let guard = consumer_cache().lock().unwrap_or_else(|e| e.into_inner());
        if guard.0 == key {
            return Arc::clone(&guard.1);
        }
    }
    let rows = Arc::new(group(raw, filter));
    {
        let mut guard = consumer_cache().lock().unwrap_or_else(|e| e.into_inner());
        *guard = (key, Arc::clone(&rows));
    }
    if !raw.is_empty() {
        *last_scan().lock().unwrap_or_else(|e| e.into_inner()) = Some(Instant::now());
    }
    rows
}

fn group(raw: &[RawProcessCpuRow], filter: Option<&[char]>) -> Vec<Grouped> {
    use std::collections::HashSet;
    let filter_set: Option<HashSet<char>> = filter.map(|letters| letters.iter().copied().collect());
    let mut map: HashMap<String, Grouped> = HashMap::new();
    for row in raw {
        let drive = if row.disk_read_mb_s + row.disk_write_mb_s >= 0.05 {
            row.drive_letter.map(|b| b as char)
        } else {
            None
        };
        if let Some(ref letters) = filter_set {
            let matches = drive.is_some_and(|d| letters.contains(&d));
            if !matches {
                continue;
            }
        }
        let entry = map.entry(row.name.to_string()).or_insert(Grouped {
            name: row.name.to_string(),
            read: 0.0,
            write: 0.0,
            ram_bytes: 0,
            count: 0,
            total: 0.0,
            drive: None,
        });
        entry.read += row.disk_read_mb_s;
        entry.write += row.disk_write_mb_s;
        entry.ram_bytes = entry.ram_bytes.saturating_add(row.ram_bytes);
        entry.count += 1;
        if entry.drive.is_none() {
            entry.drive = drive;
        }
    }
    let mut rows: Vec<Grouped> = map
        .into_values()
        .map(|mut g| {
            g.total = g.read + g.write;
            g
        })
        .collect();
    rows.sort_by(|a, b| {
        b.total.total_cmp(&a.total).then(b.ram_bytes.cmp(&a.ram_bytes)).then(a.name.cmp(&b.name))
    });
    rows.retain(|g| g.total >= 0.05);
    rows.truncate(8);
    rows
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    fn r(pid: u32, n: &str, rd: f32, wr: f32) -> RawProcessCpuRow {
        RawProcessCpuRow {
            pid,
            name: Arc::from(n),
            cpu_usage: 0.0,
            ram_bytes: 0,
            disk_read_mb_s: rd,
            disk_write_mb_s: wr,
            eco_qos: None,
            exe_path: None,
            drive_letter: None,
            creation_time: None,
            start_time: 0,
        }
    }
    #[test]
    fn group_excludes_idle_zero_io_entries() {
        let rows = vec![
            r(1, "active.exe", 3.0, 1.0),
            r(2, "chrome.exe", 0.0, 0.0),
            r(3, "svchost.exe", 0.0, 0.0),
        ];
        let g = group(&rows, None);
        assert_eq!(g.len(), 1);
        assert_eq!(g[0].name, "active.exe");
        let all_idle = vec![r(4, "idle.exe", 0.0, 0.0)];
        assert!(group(&all_idle, None).is_empty());
    }
}
