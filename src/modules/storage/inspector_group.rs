//! Grouped disk consumers with ABA-safe cache for storage inspector.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Instant;

use crate::modules::cpu::exe_cache::cached_exe_path;
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

type CacheKey = (usize, usize, u32, u32, u64);
type ConsumerCache = (CacheKey, Arc<Vec<Grouped>>);
static CONSUMER_CACHE: OnceLock<Mutex<ConsumerCache>> = OnceLock::new();
static LAST_SCAN: OnceLock<Mutex<Option<Instant>>> = OnceLock::new();
static FILTER_ON: OnceLock<Mutex<bool>> = OnceLock::new();

fn consumer_cache() -> &'static Mutex<ConsumerCache> {
    CONSUMER_CACHE.get_or_init(|| Mutex::new(((0, 0, 0, 0, 0), Arc::new(Vec::new()))))
}

pub fn last_scan() -> &'static Mutex<Option<Instant>> {
    LAST_SCAN.get_or_init(|| Mutex::new(None))
}

pub fn filter_on() -> &'static Mutex<bool> {
    // Default ON so the selected disk's letters auto-filter the table.
    FILTER_ON.get_or_init(|| Mutex::new(true))
}

pub(crate) fn exe_drive(pid: u32, start_time: u64) -> Option<char> {
    let path = cached_exe_path(pid, start_time);
    let b = path.as_bytes();
    if b.len() >= 2 && b[1] == b':' && b[0].is_ascii_alphabetic() {
        Some(b[0].to_ascii_uppercase() as char)
    } else {
        None
    }
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

fn scan_key(raw: &[RawProcessCpuRow], filter: Option<&[char]>) -> CacheKey {
    let mut rsum = 0.0f32;
    let mut wsum = 0.0f32;
    for r in raw {
        rsum += r.disk_read_mb_s;
        wsum += r.disk_write_mb_s;
    }
    (raw.as_ptr() as usize, raw.len(), rsum.to_bits(), wsum.to_bits(), drive_mask(filter))
}

/// Returns cached top-5 grouped consumers; ABA-safe key prevents stale hits.
/// `filter` is the set of drive letters for the selected disk (None = all).
pub fn cached_consumers(raw: &[RawProcessCpuRow], filter: Option<&[char]>) -> Arc<Vec<Grouped>> {
    let key = scan_key(raw, filter);
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
    let mut map: HashMap<String, Grouped> = HashMap::new();
    for row in raw {
        let drive = if row.disk_read_mb_s + row.disk_write_mb_s >= 0.05 {
            exe_drive(row.pid, row.start_time)
        } else {
            None
        };
        if let Some(letters) = filter {
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
    rows.truncate(5);
    rows
}
