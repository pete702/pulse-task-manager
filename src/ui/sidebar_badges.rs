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

pub fn cached_network_badge(total_mbps: f32) -> Arc<str> {
    let tenths = (total_mbps * 10.0).round() as u64;
    let key = tenths;
    static NET_CACHE: OnceLock<Mutex<(u64, Arc<str>)>> = OnceLock::new();
    let mut cache = NET_CACHE
        .get_or_init(|| Mutex::new((0, Arc::from(""))))
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    if cache.0 != key {
        cache.0 = key;
        cache.1 = if total_mbps >= 1000.0 {
            Arc::from(format!("{:.2} Gbps", total_mbps / 1000.0))
        } else {
            Arc::from(format!("{:.1} Mbps", total_mbps))
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

pub fn cached_storage_badge(total_mb_s: f32) -> Arc<str> {
    let tenths = (total_mb_s * 10.0).round() as u32;
    let key = tenths;
    static STORAGE_CACHE: OnceLock<Mutex<(u32, Arc<str>)>> = OnceLock::new();
    let mut cache = STORAGE_CACHE
        .get_or_init(|| Mutex::new((0, Arc::from(""))))
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    if cache.0 != key {
        cache.0 = key;
        cache.1 = Arc::from(format!("{:.1} MB/s", total_mb_s));
    }
    Arc::clone(&cache.1)
}

/// Stable accent with hysteresis: on above 0.15, off below 0.05.
pub fn badge_active(total: f32, state: &mut bool) -> bool {
    if *state {
        if total < 0.05 {
            *state = false;
        }
    } else if total > 0.15 {
        *state = true;
    }
    *state
}
