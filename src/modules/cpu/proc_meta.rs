//! Lazy bounded metadata cache (CommandLine/User) for process rows.
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex, OnceLock};

const META_CAP: usize = 1024;

/// Lazily resolved metadata for one process identity.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ProcMeta {
    pub command_line: Arc<str>,
    pub user: Arc<str>,
}

type MetaCache = (HashMap<(u32, u64), ProcMeta>, VecDeque<(u32, u64)>);
static META: OnceLock<Mutex<MetaCache>> = OnceLock::new();

fn new_store() -> MetaCache {
    (HashMap::new(), VecDeque::new())
}

fn lock_store() -> std::sync::MutexGuard<'static, MetaCache> {
    META.get_or_init(|| Mutex::new(new_store())).lock().unwrap_or_else(|e| e.into_inner())
}

fn cache_get(store: &mut MetaCache, key: (u32, u64), command_line: &str, user: &str) -> ProcMeta {
    if let Some(meta) = store.0.get(&key) {
        return meta.clone();
    }
    let meta = ProcMeta { command_line: Arc::from(command_line), user: Arc::from(user) };
    crate::util::insert_capped_fifo(&mut store.0, &mut store.1, META_CAP, key, meta.clone());
    meta
}

/// Pure cache lookup: returns the stored strings on a hit, otherwise runs
/// `compute` exactly once and inserts the result into the bounded FIFO ring.
fn cache_get_or_compute<C>(store: &mut MetaCache, key: (u32, u64), compute: C) -> ProcMeta
where
    C: FnOnce() -> (Arc<str>, Arc<str>),
{
    if let Some(meta) = store.0.get(&key) {
        return meta.clone();
    }
    let (command_line, user) = compute();
    let meta = ProcMeta { command_line, user };
    crate::util::insert_capped_fifo(&mut store.0, &mut store.1, META_CAP, key, meta.clone());
    meta
}

/// Returns cached metadata for a process identity, inserting it on first use.
/// A bounded FIFO ring evicts the oldest identity once the cap is reached.
pub fn get_or_cache(pid: u32, start_time: u64, command_line: &str, user: &str) -> ProcMeta {
    cache_get(&mut lock_store(), (pid, start_time), command_line, user)
}

/// Lazily formats CommandLine/User only when the `(pid, start_time)` identity
/// is not cached yet. Repeated calls reuse the same `Arc<str>` allocations.
pub fn get_or_compute<C>(pid: u32, start_time: u64, compute: C) -> ProcMeta
where
    C: FnOnce() -> (Arc<str>, Arc<str>),
{
    cache_get_or_compute(&mut lock_store(), (pid, start_time), compute)
}

/// Pure extended Ctrl+C formatter: `Name | PID | RAM | CPU | Path | CommandLine`.
pub fn format_process_copy_line(
    name: &str,
    pid: u32,
    ram: &str,
    cpu: &str,
    path: &str,
    command_line: &str,
) -> String {
    format!("{name} | {pid} | {ram} | {cpu} | {path} | {command_line}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_lookup_uses_first_inserted_value() {
        let mut store = new_store();
        let a = cache_get(&mut store, (1, 1), "cmd-A", "user-A");
        assert_eq!(a.command_line.as_ref(), "cmd-A");
        let again = cache_get(&mut store, (1, 1), "changed", "changed");
        assert_eq!(again.command_line.as_ref(), "cmd-A");
        assert_eq!(again.user.as_ref(), "user-A");
    }

    #[test]
    fn bounded_fifo_evicts_oldest_identity() {
        let mut store = new_store();
        for i in 0u32..(META_CAP as u32) {
            let cmd = format!("cmd-{i}");
            cache_get(&mut store, (100 + i, 1), &cmd, "user");
        }
        assert_eq!(store.0.len(), META_CAP);
        assert_eq!(store.1.len(), META_CAP);
        cache_get(&mut store, (9999, 1), "new", "user");
        assert_eq!(store.0.len(), META_CAP);
        assert_eq!(store.1.len(), META_CAP);
        assert!(!store.0.contains_key(&(100, 1)));
        assert!(store.0.contains_key(&(101, 1)));
        assert!(store.0.contains_key(&(9999, 1)));
    }

    #[test]
    fn copy_line_formats_all_fields() {
        let line = format_process_copy_line(
            "app.exe",
            42,
            "12.0 MB",
            "5.0%",
            "C:\\apps\\app.exe",
            "app.exe --flag",
        );
        assert_eq!(line, "app.exe | 42 | 12.0 MB | 5.0% | C:\\apps\\app.exe | app.exe --flag");
    }

    #[test]
    fn get_or_compute_reuses_strings_and_refreshes_on_new_start_time() {
        let mut store = new_store();
        let calls = std::cell::Cell::new(0u32);
        let compute = |cmd: &'static str, usr: &'static str| {
            let calls = &calls;
            move || {
                calls.set(calls.get() + 1);
                (Arc::from(cmd), Arc::from(usr))
            }
        };

        let first = cache_get_or_compute(&mut store, (7, 100), compute("cmd-v1", "user-v1"));
        let second = cache_get_or_compute(&mut store, (7, 100), compute("cmd-v2", "user-v2"));
        assert_eq!(calls.get(), 1, "compute must run only on a cache miss");
        assert_eq!(first.command_line.as_ref(), "cmd-v1");
        assert_eq!(second.command_line.as_ref(), "cmd-v1");
        assert_eq!(second.user.as_ref(), "user-v1");
        assert!(
            Arc::ptr_eq(&first.command_line, &second.command_line),
            "cached CommandLine must be shared, not re-allocated"
        );
        assert!(
            Arc::ptr_eq(&first.user, &second.user),
            "cached User must be shared, not re-allocated"
        );

        let third = cache_get_or_compute(&mut store, (7, 200), compute("cmd-v3", "user-v3"));
        assert_eq!(calls.get(), 2, "new start_time must be a fresh cache key");
        assert_eq!(third.command_line.as_ref(), "cmd-v3");
        assert_eq!(third.user.as_ref(), "user-v3");
        assert_eq!(store.0.len(), 2);
    }
}
