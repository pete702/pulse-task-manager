//! Cached badge text helpers for the sidebar (extracted from sidebar.rs).

use std::sync::{Arc, Mutex, OnceLock};

pub fn cached_pct(pct: f32) -> Arc<str> {
    let key = (pct * 10.0).round() as u16;
    static RAM_CACHE: OnceLock<Mutex<(u16, Arc<str>)>> = OnceLock::new();
    let mut cache = RAM_CACHE
        .get_or_init(|| Mutex::new((0, Arc::from(""))))
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    if cache.0 != key {
        cache.0 = key;
        cache.1 = Arc::from(format!("{}%", (pct.round() as u32)));
    }
    Arc::clone(&cache.1)
}

pub fn cached_cpu_pct(pct: f32) -> Arc<str> {
    let key = (pct * 10.0).round() as u16;
    static CPU_CACHE: OnceLock<Mutex<(u16, Arc<str>)>> = OnceLock::new();
    let mut cache = CPU_CACHE
        .get_or_init(|| Mutex::new((0, Arc::from(""))))
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    if cache.0 != key {
        cache.0 = key;
        cache.1 = Arc::from(format!("{}%", (pct.round() as u32)));
    }
    Arc::clone(&cache.1)
}

pub fn cached_network_badge(total_mbps: f32, ping_ms: Option<f32>) -> Arc<str> {
    let tenths = (total_mbps * 10.0).round() as u64;
    let ping_key = ping_ms.map(|p| p.round() as u64).unwrap_or(u64::MAX);
    let key = (tenths << 32) | (ping_key & 0xFFFF_FFFF);
    static NET_CACHE: OnceLock<Mutex<(u64, Arc<str>)>> = OnceLock::new();
    let mut cache = NET_CACHE
        .get_or_init(|| Mutex::new((0, Arc::from(""))))
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    if cache.0 != key {
        cache.0 = key;
        cache.1 = if total_mbps >= 0.1 {
            Arc::from(format!("{:.1} Mbps", total_mbps))
        } else if let Some(ms) = ping_ms {
            Arc::from(format!("{:.0} ms", ms))
        } else {
            Arc::from("IDLE")
        };
    }
    Arc::clone(&cache.1)
}

pub fn cached_count(count: u32) -> Arc<str> {
    static COUNT_CACHE: OnceLock<Mutex<(u32, Arc<str>)>> = OnceLock::new();
    let mut cache = COUNT_CACHE
        .get_or_init(|| Mutex::new((0, Arc::from(""))))
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    if cache.0 != count {
        cache.0 = count;
        cache.1 = Arc::from(count.to_string());
    }
    Arc::clone(&cache.1)
}

pub fn cached_storage_badge(total_mb_s: f32, volumes: usize) -> Arc<str> {
    let tenths = (total_mb_s * 10.0).round() as u32;
    let key = if total_mb_s >= 0.1 { tenths | 0x8000_0000 } else { volumes as u32 };
    static STORAGE_CACHE: OnceLock<Mutex<(u32, Arc<str>)>> = OnceLock::new();
    let mut cache = STORAGE_CACHE
        .get_or_init(|| Mutex::new((0, Arc::from(""))))
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    if cache.0 != key {
        cache.0 = key;
        cache.1 = if total_mb_s >= 0.1 {
            Arc::from(format!("{:.1} MB/s", total_mb_s))
        } else {
            Arc::from(format!("{} VOL", volumes))
        };
    }
    Arc::clone(&cache.1)
}
