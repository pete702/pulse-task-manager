//! Per-PID exe path cache queried only on hover.
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

#[derive(Clone, Debug)]
struct ExeCacheEntry {
    text: String,
    retry_after: Option<Instant>,
}

const FALLBACK_TTL: Duration = Duration::from_secs(30);
const MAX_IN_FLIGHT: usize = 4;

static EXE_CACHE: OnceLock<Mutex<HashMap<(u32, u64), ExeCacheEntry>>> = OnceLock::new();
static IN_FLIGHT: AtomicUsize = AtomicUsize::new(0);

fn cache_slot() -> &'static Mutex<HashMap<(u32, u64), ExeCacheEntry>> {
    EXE_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn evict_one(g: &mut HashMap<(u32, u64), ExeCacheEntry>) {
    if g.len() >= 512 {
        if let Some(k) = g.keys().next().copied() {
            g.remove(&k);
        }
    }
}

struct InFlightGuard;
impl Drop for InFlightGuard {
    fn drop(&mut self) {
        IN_FLIGHT.fetch_sub(1, Ordering::AcqRel);
    }
}

/// Resolves exe path for hover; known_path short-circuits the cache.
pub fn resolve_exe_cached(pid: u32, start_time: u64, known_path: Option<&str>) -> String {
    let key = (pid, start_time);
    if let Some(s) = known_path {
        if !s.is_empty() {
            let mut g = cache_slot().lock().unwrap_or_else(|e| e.into_inner());
            evict_one(&mut g);
            g.insert(key, ExeCacheEntry { text: s.to_owned(), retry_after: None });
            return s.to_owned();
        }
    }
    let now = Instant::now();
    {
        let g = cache_slot().lock().unwrap_or_else(|e| e.into_inner());
        if let Some(entry) = g.get(&key) {
            match entry.retry_after {
                None => return entry.text.clone(),
                Some(until) if now < until => return entry.text.clone(),
                Some(_) => {}
            }
        }
    }
    if pid == 0 || pid == 4 {
        return format!("PID {pid}");
    }
    let mut cur = IN_FLIGHT.load(Ordering::Acquire);
    loop {
        if cur >= MAX_IN_FLIGHT {
            return format!("PID {pid}");
        }
        match IN_FLIGHT.compare_exchange_weak(cur, cur + 1, Ordering::AcqRel, Ordering::Acquire) {
            Ok(_) => break,
            Err(v) => cur = v,
        }
    }
    {
        let mut g = cache_slot().lock().unwrap_or_else(|e| e.into_inner());
        evict_one(&mut g);
        g.insert(
            key,
            ExeCacheEntry { text: format!("PID {pid}"), retry_after: Some(now + FALLBACK_TTL) },
        );
    }
    std::thread::spawn(move || {
        let _guard = InFlightGuard;
        let resolved = crate::modules::memory::query_process_exe_path(pid);
        let mut g = cache_slot().lock().unwrap_or_else(|e| e.into_inner());
        match resolved {
            Some(p) if !p.is_empty() => {
                g.insert(key, ExeCacheEntry { text: p, retry_after: None });
            }
            _ => {
                g.insert(
                    key,
                    ExeCacheEntry {
                        text: format!("PID {pid}"),
                        retry_after: Some(Instant::now() + FALLBACK_TTL),
                    },
                );
            }
        }
    });
    format!("PID {pid}")
}

/// Cached tooltip path for a PID; queries Win32 only on background thread.
pub fn cached_exe_path(pid: u32, start_time: u64) -> String {
    resolve_exe_cached(pid, start_time, None)
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
    #[test]
    fn saturated_in_flight_returns_without_sticky_insert() {
        IN_FLIGHT.store(MAX_IN_FLIGHT, Ordering::SeqCst);
        {
            let mut g = cache_slot().lock().unwrap_or_else(|e| e.into_inner());
            g.remove(&(777_777_777, 1));
        }
        let v = resolve_exe_cached(777_777_777, 1, None);
        assert_eq!(v, "PID 777777777");
        {
            let g = cache_slot().lock().unwrap_or_else(|e| e.into_inner());
            assert!(!g.contains_key(&(777_777_777, 1)));
        }
        IN_FLIGHT.store(0, Ordering::SeqCst);
    }
    #[test]
    fn expired_fallback_retryable_resolved_sticky() {
        IN_FLIGHT.store(MAX_IN_FLIGHT, Ordering::SeqCst);
        {
            let mut g = cache_slot().lock().unwrap_or_else(|e| e.into_inner());
            g.insert(
                (666_666_666, 1),
                ExeCacheEntry {
                    text: "OLD-FALLBACK".to_owned(),
                    retry_after: Some(Instant::now()),
                },
            );
            g.insert(
                (666_666_667, 1),
                ExeCacheEntry { text: "C:\\real.exe".to_owned(), retry_after: None },
            );
        }
        let v = resolve_exe_cached(666_666_666, 1, None);
        assert_eq!(v, "PID 666666666");
        let v2 = resolve_exe_cached(666_666_667, 1, None);
        assert_eq!(v2, "C:\\real.exe");
        {
            let g = cache_slot().lock().unwrap_or_else(|e| e.into_inner());
            assert_eq!(g.get(&(666_666_667, 1)).map(|e| e.text.as_str()), Some("C:\\real.exe"));
        }
        IN_FLIGHT.store(0, Ordering::SeqCst);
    }
}
