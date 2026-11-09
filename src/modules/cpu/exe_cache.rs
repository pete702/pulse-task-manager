//! Per-PID exe path cache queried only on hover.
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, OnceLock};

static CACHE: OnceLock<Mutex<HashMap<(u32, u64), String>>> = OnceLock::new();
static IN_FLIGHT: AtomicUsize = AtomicUsize::new(0);

/// Cached tooltip path for a PID; queries Win32 only on background thread.
pub fn cached_exe_path(pid: u32, start_time: u64) -> String {
    let slot = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let key = (pid, start_time);
    let fallback = format!("PID {pid}");
    {
        let mut g = slot.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(v) = g.get(&key) {
            return v.clone();
        }
        if g.len() >= 2048 {
            let keep = g.len() / 2;
            let drop_count = g.len() - keep;
            let stale: Vec<(u32, u64)> = g.keys().take(drop_count).copied().collect();
            for k in stale {
                g.remove(&k);
            }
        }
        g.insert(key, fallback.clone());
    }
    if IN_FLIGHT.fetch_add(1, Ordering::AcqRel) < 4 {
        std::thread::spawn(move || {
            if let Some(resolved) = crate::modules::memory::query_process_exe_path(pid) {
                let s = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
                let mut g = s.lock().unwrap_or_else(|e| e.into_inner());
                g.insert(key, resolved);
            }
            IN_FLIGHT.fetch_sub(1, Ordering::AcqRel);
        });
    } else {
        IN_FLIGHT.fetch_sub(1, Ordering::AcqRel);
    }
    fallback
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn first_call_caches_fallback_immediately() {
        let v = cached_exe_path(999_999_999, 42);
        assert_eq!(v, "PID 999999999");
        let v2 = cached_exe_path(999_999_999, 42);
        assert_eq!(v2, "PID 999999999");
    }
}
