//! Persistent PDH query: total adapter VRAM, per-process VRAM, GPU % and paged est.
#[cfg(target_os = "windows")]
use super::pdh_gpu_counters::{aggregate, read_wide, wide, CounterRows};
pub use super::pdh_gpu_counters::{detect_ai_model_tag, is_vram_spill, GpuTelemetrySnapshot};
#[cfg(target_os = "windows")]
pub use super::pdh_gpu_counters::{AdapterMemoryTotals, GpuHistoryRing, GpuHistorySample};
use std::collections::HashMap;
#[cfg(target_os = "windows")]
use std::time::{Duration, Instant};
#[cfg(target_os = "windows")]
const PDH_FMT_DOUBLE: u32 = 0x0000_0200;
#[cfg(target_os = "windows")]
const PDH_FMT_NOCAP100: u32 = 0x0000_8000;
#[cfg(target_os = "windows")]
const PDH_MORE_DATA: i32 = 0x8000_07D2u32 as i32;
#[cfg(target_os = "windows")]
const REFRESH_INTERVAL: Duration = Duration::from_secs(2);
#[cfg(any(target_os = "windows", test))]
const INITIAL_BACKOFF_SECS: u64 = 5;
#[cfg(any(target_os = "windows", test))]
const MAX_BACKOFF_SECS: u64 = 60;

/// True when PdhCollectQueryData succeeded or returned benign PDH_NO_DATA.
#[cfg(any(target_os = "windows", test))]
pub fn collect_status_ok(status: i32) -> bool {
    const PDH_NO_DATA: i32 = 0x8000_07D5u32 as i32;
    status == 0 || status == PDH_NO_DATA
}

/// Pure backoff progression: 5 -> 10 -> 20 -> 40 -> 60 -> 60.
#[cfg(any(target_os = "windows", test))]
pub fn next_backoff_secs(current: u64) -> u64 {
    if current == 0 {
        INITIAL_BACKOFF_SECS
    } else {
        current.saturating_mul(2).min(MAX_BACKOFF_SECS)
    }
}
#[cfg(target_os = "windows")]
#[link(name = "pdh")]
extern "system" {
    fn PdhOpenQueryW(data_source: *const u16, user_data: usize, query: *mut isize) -> i32;
    fn PdhAddEnglishCounterW(q: isize, p: *const u16, ud: usize, c: *mut isize) -> i32;
    fn PdhCollectQueryData(query: isize) -> i32;
    fn PdhGetFormattedCounterArrayW(
        counter: isize,
        format: u32,
        size: *mut u32,
        count: *mut u32,
        buffer: *mut PdhItem,
    ) -> i32;
    fn PdhCloseQuery(query: isize) -> i32;
}
#[repr(C)]
#[cfg(target_os = "windows")]
struct PdhItem {
    sz_name: *mut u16,
    c_status: u32,
    _pad: u32,
    double_value: f64,
}
#[cfg(target_os = "windows")]
struct PdhGpuCollector {
    query: isize,
    counters: [isize; 8],
    last_collect: Instant,
    retry_after: Instant,
    backoff_secs: u64,
    snapshot: GpuTelemetrySnapshot,
}
#[cfg(target_os = "windows")]
impl PdhGpuCollector {
    #[allow(unsafe_code)]
    fn add(query: isize, path: &str) -> isize {
        let text = wide(path);
        let mut counter = 0isize;
        if unsafe { PdhAddEnglishCounterW(query, text.as_ptr(), 0, &mut counter) } != 0 {
            return 0;
        }
        counter
    }
    #[allow(unsafe_code)]
    fn open() -> Option<Self> {
        let mut query = 0isize;
        if unsafe { PdhOpenQueryW(std::ptr::null(), 0, &mut query) } != 0 || query == 0 {
            return None;
        }
        let counters = [
            Self::add(query, "\\GPU Process Memory(*)\\Local Usage"),
            Self::add(query, "\\GPU Process Memory(*)\\Non Local Usage"),
            Self::add(query, "\\GPU Process Memory(*)\\Total Committed"),
            Self::add(query, "\\GPU Process Memory(*)\\Dedicated Usage"),
            Self::add(query, "\\GPU Process Memory(*)\\Shared Usage"),
            Self::add(query, "\\GPU Adapter Memory(*)\\Dedicated Usage"),
            Self::add(query, "\\GPU Adapter Memory(*)\\Shared Usage"),
            Self::add(query, "\\GPU Engine(*)\\Utilization Percentage"),
        ];
        let status = unsafe { PdhCollectQueryData(query) };
        if !collect_status_ok(status) {
            unsafe { PdhCloseQuery(query) };
            return None;
        }
        let now = Instant::now();
        Some(Self {
            query,
            counters,
            last_collect: now - REFRESH_INTERVAL,
            retry_after: now,
            backoff_secs: INITIAL_BACKOFF_SECS,
            snapshot: GpuTelemetrySnapshot::default(),
        })
    }
    fn penalize(&mut self) {
        self.retry_after = Instant::now() + Duration::from_secs(self.backoff_secs);
        self.backoff_secs = next_backoff_secs(self.backoff_secs);
    }
    #[allow(unsafe_code)]
    fn collect(&mut self) -> bool {
        let status = unsafe { PdhCollectQueryData(self.query) };
        if !collect_status_ok(status) {
            self.penalize();
            return false;
        }
        let c = self.counters;
        let rows = CounterRows {
            local: unsafe { self.read_array(c[0]) },
            non_local: unsafe { self.read_array(c[1]) },
            committed: unsafe { self.read_array(c[2]) },
            proc_dedicated: unsafe { self.read_array(c[3]) },
            proc_shared: unsafe { self.read_array(c[4]) },
            adapter_dedicated: unsafe { self.read_array(c[5]) },
            adapter_shared: unsafe { self.read_array(c[6]) },
            engines: unsafe { self.read_array(c[7]) },
        };
        if rows.is_empty() {
            self.penalize();
            return false;
        }
        let epoch = self.snapshot.epoch.wrapping_add(1);
        self.snapshot = aggregate(rows, epoch, sample_gpu_history().len());
        self.backoff_secs = INITIAL_BACKOFF_SECS;
        true
    }
    #[allow(unsafe_code)]
    unsafe fn read_array(&self, counter: isize) -> Vec<(String, f64)> {
        if counter == 0 {
            return Vec::new();
        }
        let format = PDH_FMT_DOUBLE | PDH_FMT_NOCAP100;
        let mut bytes = 0u32;
        let mut count = 0u32;
        let none: *mut PdhItem = std::ptr::null_mut();
        let rc = PdhGetFormattedCounterArrayW(counter, format, &mut bytes, &mut count, none);
        if rc != PDH_MORE_DATA || bytes == 0 {
            return Vec::new();
        }
        let slots = (bytes as usize).div_ceil(8);
        let mut buf: Vec<u64> = vec![0u64; slots];
        let mut actual = bytes;
        count = 0;
        let out = buf.as_mut_ptr().cast::<PdhItem>();
        let rc = PdhGetFormattedCounterArrayW(counter, format, &mut actual, &mut count, out);
        if rc != 0 || count == 0 {
            return Vec::new();
        }
        let used = (count as usize).saturating_mul(std::mem::size_of::<PdhItem>());
        if count as usize > slots || used > slots * 8 {
            return Vec::new();
        }
        let items = std::slice::from_raw_parts(buf.as_ptr().cast::<PdhItem>(), count as usize);
        let mut rows = Vec::new();
        for item in items {
            if item.sz_name.is_null() {
                continue;
            }
            if item.c_status != 0 && item.c_status != 1 {
                continue;
            }
            let value = item.double_value;
            if value.is_finite() {
                rows.push((read_wide(item.sz_name), value));
            }
        }
        rows
    }
}
#[cfg(target_os = "windows")]
#[allow(unsafe_code)]
impl Drop for PdhGpuCollector {
    fn drop(&mut self) {
        if self.query != 0 {
            unsafe { PdhCloseQuery(self.query) };
        }
    }
}
#[cfg(target_os = "windows")]
static COLLECTOR: std::sync::OnceLock<std::sync::Mutex<Option<PdhGpuCollector>>> =
    std::sync::OnceLock::new();
#[cfg(target_os = "windows")]
static HISTORY: std::sync::OnceLock<std::sync::Mutex<GpuHistoryRing>> = std::sync::OnceLock::new();
#[cfg(target_os = "windows")]
fn push_history(snap: &GpuTelemetrySnapshot) {
    let lock = HISTORY.get_or_init(|| std::sync::Mutex::new(GpuHistoryRing::with_cap(120)));
    let mut ring = lock.lock().unwrap_or_else(|err| err.into_inner());
    let sum = |f: fn(&AdapterMemoryTotals) -> u64| {
        snap.adapters.values().fold(0u64, |a, t| a.saturating_add(f(t)))
    };
    ring.push(GpuHistorySample {
        epoch: snap.epoch,
        dedicated_used_bytes: sum(|t| t.dedicated_used_bytes),
        shared_used_bytes: sum(|t| t.shared_used_bytes),
        busiest_gpu_pct: snap.processes.values().filter_map(|p| p.gpu_pct).fold(0.0f32, f32::max),
    });
}
#[cfg(target_os = "windows")]
fn windows_sample() -> GpuTelemetrySnapshot {
    let lock = COLLECTOR.get_or_init(|| std::sync::Mutex::new(None));
    let mut guard = lock.lock().unwrap_or_else(|err| err.into_inner());
    if guard.is_none() {
        *guard = PdhGpuCollector::open();
    }
    let Some(collector) = guard.as_mut() else {
        return GpuTelemetrySnapshot::default();
    };
    if Instant::now() < collector.retry_after || collector.last_collect.elapsed() < REFRESH_INTERVAL
    {
        return collector.snapshot.clone();
    }
    if collector.collect() {
        push_history(&collector.snapshot);
    }
    collector.last_collect = Instant::now();
    collector.snapshot.clone()
}
#[cfg(target_os = "windows")]
pub fn sample_gpu_telemetry() -> GpuTelemetrySnapshot {
    windows_sample()
}
#[cfg(not(target_os = "windows"))]
pub fn sample_gpu_telemetry() -> GpuTelemetrySnapshot {
    GpuTelemetrySnapshot::default()
}
#[cfg(target_os = "windows")]
pub fn sample_gpu_history() -> GpuHistoryRing {
    let lock = HISTORY.get_or_init(|| std::sync::Mutex::new(GpuHistoryRing::with_cap(120)));
    lock.lock().unwrap_or_else(|err| err.into_inner()).clone()
}
#[cfg(target_os = "windows")]
pub fn sample_process_gpu_memory() -> HashMap<u32, (u64, u64)> {
    let mut out: HashMap<u32, (u64, u64)> = HashMap::new();
    for p in windows_sample().processes.into_values() {
        let entry = out.entry(p.pid).or_insert((0, 0));
        entry.0 = entry.0.saturating_add(p.local_bytes);
        entry.1 = entry.1.saturating_add(p.non_local_bytes);
    }
    out
}
#[cfg(not(target_os = "windows"))]
pub fn sample_process_gpu_memory() -> HashMap<u32, (u64, u64)> {
    HashMap::new()
}
#[cfg(test)]
#[path = "pdh_gpu_tests.rs"]
mod tests;
