//! Per-PID exe path cache queried only on hover.
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

/// Cached tooltip path for a PID; queries Win32 only on first hover.
pub fn cached_exe_path(pid: u32, start_time: u64) -> String {
    static CACHE: OnceLock<Mutex<HashMap<(u32, u64), String>>> = OnceLock::new();
    let slot = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let key = (pid, start_time);
    {
        let g = slot.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(v) = g.get(&key) {
            return v.clone();
        }
    }
    let v = crate::modules::memory::win32_api::query_process_exe_path(pid)
        .unwrap_or_else(|| format!("PID {pid}"));
    {
        let mut g = slot.lock().unwrap_or_else(|e| e.into_inner());
        if g.len() >= 2048 {
            let keep = g.len() / 2;
            let drop_count = g.len() - keep;
            let stale: Vec<(u32, u64)> = g.keys().take(drop_count).copied().collect();
            for k in stale {
                g.remove(&k);
            }
        }
        g.insert(key, v.clone());
    }
    v
}
