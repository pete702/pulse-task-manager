//! Read-only Windows Services (SCM) and Autostart registry enumeration.
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

#[cfg(target_os = "windows")]
#[path = "services_startup_win.rs"]
mod win;

const REFRESH: Duration = Duration::from_secs(5);

#[cfg(target_os = "windows")]
pub(crate) const ERROR_MORE_DATA: u32 = 234;
#[cfg(target_os = "windows")]
static SERVICES_PARTIAL: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
#[cfg(target_os = "windows")]
static STARTUP_PARTIAL: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Next capacity for a `ERROR_MORE_DATA` retry, or `None` once the cap is reached.
#[cfg(target_os = "windows")]
pub(crate) fn retry_capacity(
    rc: u32,
    current: usize,
    reported: usize,
    cap: usize,
) -> Option<usize> {
    if rc != ERROR_MORE_DATA {
        return None;
    }
    let next = reported.max(current.saturating_mul(2)).min(cap);
    (next > current).then_some(next)
}

/// Decodes UTF-16LE bytes, stopping at the first NUL unit.
#[cfg(target_os = "windows")]
pub(crate) fn decode_utf16_le(bytes: &[u8]) -> String {
    let units: Vec<u16> = bytes
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
        .take_while(|&unit| unit != 0)
        .collect();
    String::from_utf16_lossy(&units)
}

/// Copies aligned `u64` storage into bytes without any unaligned typed slice.
#[cfg(target_os = "windows")]
pub(crate) fn wide_bytes(words: &[u64]) -> Vec<u8> {
    words.iter().flat_map(|word| word.to_le_bytes()).collect()
}

/// Byte counts from Win32 must be converted to `u64` element counts, never the reverse.
#[cfg(target_os = "windows")]
pub(crate) fn bytes_to_words(bytes: usize) -> usize {
    bytes.div_ceil(std::mem::size_of::<u64>())
}

/// True for the four transitional SCM states.
pub fn is_pending_state(state: u32) -> bool {
    matches!(state, 2 | 3 | 5 | 6)
}

/// True when the last enumeration had truncated or paged data.
pub fn services_partial() -> bool {
    #[cfg(target_os = "windows")]
    {
        SERVICES_PARTIAL.load(std::sync::atomic::Ordering::Relaxed)
    }
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

/// True when the last Run-key enumeration had truncated values.
pub fn startup_partial() -> bool {
    #[cfg(target_os = "windows")]
    {
        STARTUP_PARTIAL.load(std::sync::atomic::Ordering::Relaxed)
    }
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

/// One read-only service row from the Service Control Manager.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WindowsServiceEntry {
    pub name: String,
    pub display_name: String,
    pub state: u32,
    pub pid: u32,
}

/// One read-only Run-key autostart value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StartupEntry {
    pub scope: &'static str,
    pub name: String,
    pub command: String,
}

pub fn format_service_state(state: u32) -> &'static str {
    match state {
        4 => "RUNNING",
        1 => "STOPPED",
        2 => "START_PENDING",
        3 => "STOP_PENDING",
        5 => "CONTINUE_PENDING",
        6 => "PAUSE_PENDING",
        7 => "PAUSED",
        _ => "UNKNOWN",
    }
}

pub fn matches_service_filter(
    query: &str,
    name: &str,
    display_name: &str,
    state_label: &str,
    pid: u32,
) -> bool {
    let q = query.trim().to_ascii_lowercase();
    if q.is_empty() {
        return true;
    }
    name.to_ascii_lowercase().contains(&q)
        || display_name.to_ascii_lowercase().contains(&q)
        || state_label.to_ascii_lowercase().contains(&q)
        || pid.to_string().contains(query.trim())
}

pub fn matches_startup_filter(query: &str, scope: &str, name: &str, command: &str) -> bool {
    let q = query.trim().to_ascii_lowercase();
    if q.is_empty() {
        return true;
    }
    scope.to_ascii_lowercase().contains(&q)
        || name.to_ascii_lowercase().contains(&q)
        || command.to_ascii_lowercase().contains(&q)
}

struct ServiceCache {
    last: Instant,
    entries: Vec<WindowsServiceEntry>,
}

static SERVICES: OnceLock<Mutex<ServiceCache>> = OnceLock::new();

/// Cached service list, refreshed at most every 5 seconds unless forced.
pub fn query_windows_services(force_refresh: bool) -> Vec<WindowsServiceEntry> {
    let cache = SERVICES.get_or_init(|| {
        Mutex::new(ServiceCache { last: Instant::now() - REFRESH, entries: Vec::new() })
    });
    let mut guard = cache.lock().unwrap_or_else(|err| err.into_inner());
    if force_refresh || guard.last.elapsed() >= REFRESH {
        #[cfg(target_os = "windows")]
        {
            let (entries, partial) = win::windows_services();
            SERVICES_PARTIAL.store(partial, std::sync::atomic::Ordering::Relaxed);
            if !entries.is_empty() || guard.entries.is_empty() {
                guard.entries = entries;
            }
        }
        guard.last = Instant::now();
    }
    guard.entries.clone()
}

struct StartupCache {
    last: Instant,
    entries: Vec<StartupEntry>,
}

static STARTUP: OnceLock<Mutex<StartupCache>> = OnceLock::new();

/// Cached Run-key entries, refreshed at most every 5 seconds unless forced.
pub fn query_startup_entries(force_refresh: bool) -> Vec<StartupEntry> {
    let cache = STARTUP.get_or_init(|| {
        Mutex::new(StartupCache { last: Instant::now() - REFRESH, entries: Vec::new() })
    });
    let mut guard = cache.lock().unwrap_or_else(|err| err.into_inner());
    if force_refresh || guard.last.elapsed() >= REFRESH {
        #[cfg(target_os = "windows")]
        {
            let (entries, partial) = win::windows_startup();
            STARTUP_PARTIAL.store(partial, std::sync::atomic::Ordering::Relaxed);
            if !entries.is_empty() || guard.entries.is_empty() {
                guard.entries = entries;
            }
        }
        guard.last = Instant::now();
    }
    guard.entries.clone()
}

#[cfg(test)]
#[path = "services_startup_tests.rs"]
mod tests;
