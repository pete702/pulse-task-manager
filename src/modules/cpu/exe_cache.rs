//! Per-PID exe path cache queried only on hover, via one persistent worker.
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Mutex, OnceLock};
use std::time::{Duration, Instant};

#[derive(Clone, Debug)]
struct ExeCacheEntry {
    text: String,
    retry_after: Option<Instant>,
}

const FALLBACK_TTL: Duration = Duration::from_secs(30);
const CACHE_CAP: usize = 512;

type ExeCache = (HashMap<(u32, u64), ExeCacheEntry>, VecDeque<(u32, u64)>);
static EXE_CACHE: OnceLock<Mutex<ExeCache>> = OnceLock::new();
static QUEUED: OnceLock<Mutex<HashSet<(u32, u64)>>> = OnceLock::new();
static EXE_WORKER: OnceLock<mpsc::Sender<(u32, u64)>> = OnceLock::new();
static WORKER_FAILED: AtomicBool = AtomicBool::new(false);

#[cfg(test)]
type TestResolver = Option<fn(u32) -> Option<String>>;
#[cfg(test)]
static TEST_RESOLVER: OnceLock<Mutex<TestResolver>> = OnceLock::new();

fn cache_slot() -> &'static Mutex<ExeCache> {
    EXE_CACHE.get_or_init(|| Mutex::new((HashMap::new(), VecDeque::new())))
}

fn queued_slot() -> &'static Mutex<HashSet<(u32, u64)>> {
    QUEUED.get_or_init(|| Mutex::new(HashSet::new()))
}

pub fn format_exe_fallback(pid: u32, in_flight: bool) -> String {
    if in_flight {
        format!("Resolving... (PID {pid})")
    } else {
        format!("Access denied (PID {pid})")
    }
}

fn resolve_path(pid: u32) -> Option<String> {
    #[cfg(test)]
    {
        if let Some(slot) = TEST_RESOLVER.get() {
            if let Some(f) = slot.lock().unwrap_or_else(|e| e.into_inner()).as_ref() {
                return f(pid);
            }
        }
    }
    crate::modules::memory::query_process_exe_path(pid)
}

fn worker_tx() -> Option<&'static mpsc::Sender<(u32, u64)>> {
    if WORKER_FAILED.load(Ordering::Acquire) {
        return None;
    }
    Some(EXE_WORKER.get_or_init(|| {
        let (tx, rx) = mpsc::channel::<(u32, u64)>();
        std::thread::Builder::new()
            .name("pulse-exe-cache".into())
            .spawn(move || {
                while let Ok((pid, start_time)) = rx.recv() {
                    let resolved = resolve_path(pid).filter(|p| !p.is_empty());
                    let entry = match resolved {
                        Some(path) => ExeCacheEntry { text: path, retry_after: None },
                        None => ExeCacheEntry {
                            text: format_exe_fallback(pid, false),
                            retry_after: Some(Instant::now() + FALLBACK_TTL),
                        },
                    };
                    let mut g = cache_slot().lock().unwrap_or_else(|e| e.into_inner());
                    let (map, order) = &mut *g;
                    crate::util::insert_capped_fifo(
                        map,
                        order,
                        CACHE_CAP,
                        (pid, start_time),
                        entry,
                    );
                    let mut q = queued_slot().lock().unwrap_or_else(|e| e.into_inner());
                    q.remove(&(pid, start_time));
                }
            })
            .unwrap_or_else(|_| {
                WORKER_FAILED.store(true, Ordering::Release);
                panic!("spawn exe cache worker")
            });
        tx
    }))
}

fn finish_as_failed(pid: u32, start_time: u64) {
    let entry = ExeCacheEntry {
        text: format_exe_fallback(pid, false),
        retry_after: Some(Instant::now() + FALLBACK_TTL),
    };
    let mut g = cache_slot().lock().unwrap_or_else(|e| e.into_inner());
    let (map, order) = &mut *g;
    crate::util::insert_capped_fifo(map, order, CACHE_CAP, (pid, start_time), entry);
    queued_slot().lock().unwrap_or_else(|e| e.into_inner()).remove(&(pid, start_time));
}

/// Resolves exe path for hover; known_path short-circuits the cache.
pub fn resolve_exe_cached(pid: u32, start_time: u64, known_path: Option<&str>) -> String {
    let key = (pid, start_time);
    if let Some(s) = known_path {
        if !s.is_empty() {
            let mut g = cache_slot().lock().unwrap_or_else(|e| e.into_inner());
            let (map, order) = &mut *g;
            crate::util::insert_capped_fifo(
                map,
                order,
                CACHE_CAP,
                key,
                ExeCacheEntry { text: s.to_owned(), retry_after: None },
            );
            return s.to_owned();
        }
    }
    let now = Instant::now();
    {
        let g = cache_slot().lock().unwrap_or_else(|e| e.into_inner());
        if let Some(entry) = g.0.get(&key) {
            match entry.retry_after {
                None => return entry.text.clone(),
                Some(until) if now < until => return entry.text.clone(),
                Some(_) => {}
            }
        }
    }
    if pid == 0 || pid == 4 {
        return format_exe_fallback(pid, false);
    }
    {
        let mut q = queued_slot().lock().unwrap_or_else(|e| e.into_inner());
        if !q.insert(key) {
            return format_exe_fallback(pid, true);
        }
    }
    let mut g = cache_slot().lock().unwrap_or_else(|e| e.into_inner());
    let (map, order) = &mut *g;
    crate::util::insert_capped_fifo(
        map,
        order,
        CACHE_CAP,
        key,
        ExeCacheEntry {
            text: format_exe_fallback(pid, true),
            retry_after: Some(now + FALLBACK_TTL),
        },
    );
    drop(g);
    match worker_tx() {
        Some(tx) if tx.send(key).is_ok() => {}
        _ => finish_as_failed(pid, start_time),
    }
    format_exe_fallback(pid, true)
}

/// Cached tooltip path for a PID; queries Win32 only on the persistent worker.
#[cfg(test)]
pub fn cached_exe_path(pid: u32, start_time: u64) -> String {
    resolve_exe_cached(pid, start_time, None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration as StdDuration;

    fn set_resolver(f: Option<fn(u32) -> Option<String>>) {
        *TEST_RESOLVER.get_or_init(|| Mutex::new(None)).lock().unwrap_or_else(|e| e.into_inner()) =
            f;
    }

    fn wait_for_text(key: (u32, u64), expected: &str) {
        for _ in 0..200 {
            {
                let g = cache_slot().lock().unwrap_or_else(|e| e.into_inner());
                if g.0.get(&key).map(|e| e.text.as_str()) == Some(expected) {
                    return;
                }
            }
            std::thread::sleep(StdDuration::from_millis(2));
        }
        panic!("worker did not produce expected text: {expected}");
    }

    #[test]
    fn fallback_messages_distinguish_states() {
        assert_eq!(format_exe_fallback(42, true), "Resolving... (PID 42)");
        assert_eq!(format_exe_fallback(42, false), "Access denied (PID 42)");
    }

    #[test]
    fn persistent_worker_resolves_successfully() {
        fn fake(pid: u32) -> Option<String> {
            if pid == 424_242 {
                Some("C:\\resolved.exe".to_owned())
            } else {
                crate::modules::memory::query_process_exe_path(pid)
            }
        }
        set_resolver(Some(fake));
        let key = (424_242, 9001);
        let first = cached_exe_path(key.0, key.1);
        assert_eq!(first, "Resolving... (PID 424242)");
        wait_for_text(key, "C:\\resolved.exe");
        assert_eq!(cached_exe_path(key.0, key.1), "C:\\resolved.exe");
        set_resolver(None);
    }

    #[test]
    fn failed_lookup_caches_access_denied() {
        fn fake(pid: u32) -> Option<String> {
            if pid == 424_243 {
                None
            } else {
                crate::modules::memory::query_process_exe_path(pid)
            }
        }
        set_resolver(Some(fake));
        let key = (424_243, 9002);
        let first = cached_exe_path(key.0, key.1);
        assert_eq!(first, "Resolving... (PID 424243)");
        wait_for_text(key, "Access denied (PID 424243)");
        assert_eq!(cached_exe_path(key.0, key.1), "Access denied (PID 424243)");
        set_resolver(None);
    }

    #[test]
    fn known_path_short_circuits_without_queueing() {
        let v = resolve_exe_cached(424_244, 9003, Some("C:\\known.exe"));
        assert_eq!(v, "C:\\known.exe");
    }
}
