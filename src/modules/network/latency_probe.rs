//! Non-blocking esports ping/jitter probe plus safe DNS flush.
//! A single persistent background thread serves probe and flush requests
//! over a channel so no thread is spawned per sample.

use std::collections::VecDeque;
use std::net::{TcpStream, ToSocketAddrs};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Mutex, OnceLock};
use std::time::{Duration, Instant};

#[derive(Debug, Default)]
struct ProbeState {
    ping_ms: Option<f32>,
    jitter_ms: Option<f32>,
    prev_rtt: Option<f32>,
    recent: VecDeque<bool>,
    last_probe: Option<Instant>,
    consecutive_failures: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DnsFlushState {
    Idle,
    Running,
    Success,
    Failed,
}

enum Cmd {
    Probe,
    Flush,
}

static STATE: OnceLock<Mutex<ProbeState>> = OnceLock::new();
static PROBE_ENABLED: AtomicBool = AtomicBool::new(true);
static PROBE_IN_FLIGHT: AtomicBool = AtomicBool::new(false);
static FAST_PROBE_MODE: AtomicBool = AtomicBool::new(false);
static TX: OnceLock<Mutex<Option<mpsc::Sender<Cmd>>>> = OnceLock::new();
static DNS_STATE: OnceLock<Mutex<(DnsFlushState, Option<Instant>)>> = OnceLock::new();

/// Adaptive probe interval in ms: 1000 when fast (Network tab), else 10000.
/// Sleeps in 250 ms steps: 4 steps (1000 ms) fast, 40 steps (10_000 ms) slow.
fn probe_interval_ms() -> u64 {
    if FAST_PROBE_MODE.load(Ordering::Relaxed) {
        1000
    } else {
        10_000
    }
}

/// Sets adaptive probe mode; switching to fast wakes the throttle early.
#[allow(dead_code)]
pub fn set_fast_probe_mode(fast: bool) {
    let prev = FAST_PROBE_MODE.swap(fast, Ordering::Relaxed);
    if fast && !prev {
        if let Ok(mut g) = state().try_lock() {
            g.last_probe = None;
        }
    }
}

fn state() -> &'static Mutex<ProbeState> {
    STATE.get_or_init(|| Mutex::new(ProbeState::default()))
}
fn dns_slot() -> &'static Mutex<(DnsFlushState, Option<Instant>)> {
    DNS_STATE.get_or_init(|| Mutex::new((DnsFlushState::Idle, None)))
}

struct InFlightReset;
impl Drop for InFlightReset {
    fn drop(&mut self) {
        PROBE_IN_FLIGHT.store(false, Ordering::Release);
    }
}

#[allow(dead_code)]
pub fn probe_enabled() -> bool {
    PROBE_ENABLED.load(Ordering::Relaxed)
}
#[allow(dead_code)]
pub fn set_probe_enabled(value: bool) {
    PROBE_ENABLED.store(value, Ordering::Relaxed);
    if !value {
        let mut guard = state().lock().unwrap_or_else(|e| e.into_inner());
        guard.ping_ms = None;
        guard.jitter_ms = None;
        guard.prev_rtt = None;
        guard.consecutive_failures = 0;
        guard.recent.clear();
    }
}
#[allow(dead_code)]
pub fn probe_state() -> (Option<f32>, Option<f32>, f32, bool) {
    if !probe_enabled() {
        return (None, None, 0.0, false);
    }
    let guard = state().lock().unwrap_or_else(|e| e.into_inner());
    let loss = if guard.recent.is_empty() {
        0.0
    } else {
        let fails = guard.recent.iter().filter(|ok| !**ok).count() as f32;
        fails / guard.recent.len() as f32 * 100.0
    };
    (guard.ping_ms, guard.jitter_ms, loss, true)
}
#[allow(dead_code)]
pub fn probe_sample_count() -> usize {
    state().lock().unwrap_or_else(|e| e.into_inner()).recent.len()
}
#[allow(dead_code)]
pub fn probe_failures() -> u32 {
    state().lock().unwrap_or_else(|e| e.into_inner()).consecutive_failures
}
#[allow(dead_code)]
pub fn dns_flush_state() -> DnsFlushState {
    dns_slot().lock().unwrap_or_else(|e| e.into_inner()).0
}
#[allow(dead_code)]
pub fn dns_flushed_recently() -> bool {
    let guard = dns_slot().lock().unwrap_or_else(|e| e.into_inner());
    guard.0 == DnsFlushState::Success && guard.1.is_some_and(|t| t.elapsed().as_millis() < 3000)
}

fn ensure_worker() -> Option<mpsc::Sender<Cmd>> {
    let slot = TX.get_or_init(|| Mutex::new(None));
    let mut guard = slot.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(tx) = guard.as_ref() {
        return Some(tx.clone());
    }
    let (tx, rx) = mpsc::channel::<Cmd>();
    let tx2 = tx.clone();
    std::thread::spawn(move || {
        while let Ok(cmd) = rx.recv() {
            match cmd {
                Cmd::Probe => {
                    let _reset = InFlightReset;
                    let rtt = tcp_rtt_ms();
                    record_sample(rtt);
                }
                Cmd::Flush => run_flush(),
            }
        }
    });
    *guard = Some(tx2.clone());
    Some(tx2)
}

#[allow(dead_code)]
pub fn maybe_trigger_probe() {
    if !probe_enabled() || PROBE_IN_FLIGHT.swap(true, Ordering::AcqRel) {
        return;
    }
    let now = Instant::now();
    let interval = probe_interval_ms();
    let due = {
        let mut guard = state().lock().unwrap_or_else(|e| e.into_inner());
        let due =
            guard.last_probe.is_none_or(|t| now.duration_since(t).as_millis() >= interval as u128);
        if due {
            guard.last_probe = Some(now);
        }
        due
    };
    if !due {
        PROBE_IN_FLIGHT.store(false, Ordering::Release);
        return;
    }
    let Some(tx) = ensure_worker() else {
        PROBE_IN_FLIGHT.store(false, Ordering::Release);
        return;
    };
    if tx.send(Cmd::Probe).is_err() {
        PROBE_IN_FLIGHT.store(false, Ordering::Release);
    }
}

fn tcp_rtt_ms() -> Option<f32> {
    for host in ["1.1.1.1:53", "8.8.8.8:53", "1.1.1.1:443", "8.8.8.8:443"] {
        let Some(addr) = host.to_socket_addrs().ok().and_then(|mut i| i.next()) else {
            continue;
        };
        let start = Instant::now();
        if TcpStream::connect_timeout(&addr, Duration::from_millis(800)).is_ok() {
            return Some(start.elapsed().as_secs_f32() * 1000.0);
        }
    }
    None
}

fn record_sample(rtt: Option<f32>) {
    let mut guard = state().lock().unwrap_or_else(|e| e.into_inner());
    guard.recent.push_back(rtt.is_some());
    while guard.recent.len() > 20 {
        guard.recent.pop_front();
    }
    if let Some(value) = rtt {
        guard.consecutive_failures = 0;
        let jitter = if let Some(prev) = guard.prev_rtt {
            let cur = guard.jitter_ms.unwrap_or(0.0);
            0.5 * cur + 0.5 * (value - prev).abs()
        } else {
            0.0
        };
        guard.prev_rtt = Some(value);
        guard.ping_ms = Some(value);
        guard.jitter_ms = Some(jitter);
    } else {
        guard.consecutive_failures = guard.consecutive_failures.saturating_add(1);
        guard.ping_ms = None;
        guard.jitter_ms = None;
        guard.prev_rtt = None;
    }
}

fn run_flush() {
    {
        let mut g = dns_slot().lock().unwrap_or_else(|e| e.into_inner());
        g.0 = DnsFlushState::Running;
        g.1 = None;
    }
    let mut cmd = {
        #[cfg(target_os = "windows")]
        {
            let exe = std::env::var_os("SystemRoot")
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|| std::path::PathBuf::from(r"C:\Windows"))
                .join("System32")
                .join("ipconfig.exe");
            std::process::Command::new(exe)
        }
        #[cfg(not(target_os = "windows"))]
        {
            std::process::Command::new("ipconfig")
        }
    };
    cmd.args(["/flushdns"]);
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000);
    }
    let ok = cmd.status().is_ok_and(|s| s.success());
    let mut g = dns_slot().lock().unwrap_or_else(|e| e.into_inner());
    g.0 = if ok { DnsFlushState::Success } else { DnsFlushState::Failed };
    g.1 = Some(Instant::now());
}

#[allow(dead_code)]
pub fn trigger_dns_flush() {
    let mut g = dns_slot().lock().unwrap_or_else(|e| e.into_inner());
    if g.0 == DnsFlushState::Running {
        return;
    }
    g.0 = DnsFlushState::Running;
    drop(g);
    let Some(tx) = ensure_worker() else {
        let mut g2 = dns_slot().lock().unwrap_or_else(|e| e.into_inner());
        g2.0 = DnsFlushState::Idle;
        return;
    };
    if tx.send(Cmd::Flush).is_err() {
        let mut g2 = dns_slot().lock().unwrap_or_else(|e| e.into_inner());
        g2.0 = DnsFlushState::Idle;
    }
}

#[cfg(test)]
#[path = "latency_probe_tests.rs"]
mod tests;
