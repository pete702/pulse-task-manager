use std::{
    collections::VecDeque,
    net::{TcpStream, ToSocketAddrs},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Mutex, OnceLock,
    },
    time::{Duration, Instant},
};
#[derive(Debug, Default)]
pub(crate) struct ProbeState {
    pub(crate) ping_ms: Option<f32>,
    pub(crate) jitter_ms: Option<f32>,
    pub(crate) prev_rtt: Option<f32>,
    pub(crate) recent: VecDeque<bool>,
    pub(crate) last_probe: Option<Instant>,
    pub(crate) consecutive_failures: u32,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DnsFlushState {
    Idle,
    Running,
    Success,
    Failed,
}
pub(crate) enum Cmd {
    Probe,
    Flush,
    Shutdown,
}
static STATE: OnceLock<Mutex<ProbeState>> = OnceLock::new();
static PROBE_ENABLED: AtomicBool = AtomicBool::new(true);
static PROBE_IN_FLIGHT: AtomicBool = AtomicBool::new(false);
static FAST_PROBE_MODE: AtomicBool = AtomicBool::new(false);
#[allow(clippy::type_complexity)]
pub(crate) static TX: OnceLock<
    Mutex<Option<(mpsc::Sender<Cmd>, Option<std::thread::JoinHandle<()>>)>>,
> = OnceLock::new();
static DNS_STATE: OnceLock<Mutex<(DnsFlushState, Option<Instant>)>> = OnceLock::new();
fn probe_interval_ms() -> u64 {
    if FAST_PROBE_MODE.load(Ordering::Relaxed) {
        1000
    } else {
        10_000
    }
}
pub fn set_fast_probe_mode(fast: bool) {
    let prev = FAST_PROBE_MODE.swap(fast, Ordering::Relaxed);
    if fast && !prev {
        if let Ok(mut g) = state().try_lock() {
            g.last_probe = None;
        }
    }
}
pub(crate) fn state() -> &'static Mutex<ProbeState> {
    STATE.get_or_init(|| Mutex::new(ProbeState::default()))
}
pub(crate) fn dns_slot() -> &'static Mutex<(DnsFlushState, Option<Instant>)> {
    DNS_STATE.get_or_init(|| Mutex::new((DnsFlushState::Idle, None)))
}
static PROBE_IN_FLIGHT_SINCE: OnceLock<Mutex<Option<Instant>>> = OnceLock::new();
fn in_flight_since() -> &'static Mutex<Option<Instant>> {
    PROBE_IN_FLIGHT_SINCE.get_or_init(|| Mutex::new(None))
}
pub fn is_probe_watchdog_expired() -> bool {
    if !PROBE_IN_FLIGHT.load(Ordering::Acquire) {
        return false;
    }
    let guard = in_flight_since().lock().unwrap_or_else(|e| e.into_inner());
    guard.map(|t| t.elapsed().as_millis() >= 5000).unwrap_or(false)
}
pub(crate) struct InFlightGuard;
impl Drop for InFlightGuard {
    fn drop(&mut self) {
        PROBE_IN_FLIGHT.store(false, Ordering::Release);
        *in_flight_since().lock().unwrap_or_else(|e| e.into_inner()) = None;
    }
}
pub fn probe_enabled() -> bool {
    PROBE_ENABLED.load(Ordering::Relaxed)
}
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
pub fn probe_sample_count() -> usize {
    state().lock().unwrap_or_else(|e| e.into_inner()).recent.len()
}
pub fn probe_failures() -> u32 {
    state().lock().unwrap_or_else(|e| e.into_inner()).consecutive_failures
}
pub fn dns_flush_state() -> DnsFlushState {
    dns_slot().lock().unwrap_or_else(|e| e.into_inner()).0
}
pub(crate) fn ensure_worker() -> Option<mpsc::Sender<Cmd>> {
    let slot = TX.get_or_init(|| Mutex::new(None));
    let mut guard = slot.lock().unwrap_or_else(|e| e.into_inner());
    if let Some((tx, _)) = guard.as_ref() {
        return Some(tx.clone());
    }
    let (tx, rx) = mpsc::channel::<Cmd>();
    let tx2 = tx.clone();
    let handle = std::thread::spawn(move || {
        while let Ok(cmd) = rx.recv() {
            match cmd {
                Cmd::Probe => {
                    let _guard = InFlightGuard;
                    let rtt = tcp_rtt_ms();
                    record_sample(rtt);
                }
                Cmd::Flush => crate::modules::network::latency_probe_handle::run_flush(),
                Cmd::Shutdown => break,
            }
        }
    });
    *guard = Some((tx2.clone(), Some(handle)));
    Some(tx2)
}
pub fn maybe_trigger_probe() {
    let _ = (
        crate::modules::network::latency_probe_handle::probe_snapshot(),
        probe_state(),
        probe_sample_count(),
        probe_failures(),
    );
    if is_probe_watchdog_expired() {
        PROBE_IN_FLIGHT.store(false, Ordering::Release);
        *in_flight_since().lock().unwrap_or_else(|e| e.into_inner()) = None;
    }
    if !probe_enabled() || PROBE_IN_FLIGHT.swap(true, Ordering::AcqRel) {
        return;
    }
    *in_flight_since().lock().unwrap_or_else(|e| e.into_inner()) = Some(Instant::now());
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
pub(crate) fn sanitize_probe_target(raw: &str) -> Option<&str> {
    let t = raw.trim_matches(|c: char| c.is_ascii_whitespace());
    if t.is_empty() || t.len() > 253 {
        return None;
    }
    if t.bytes().any(|b| !(b.is_ascii_alphanumeric() || b == b'.' || b == b'-' || b == b':')) {
        return None;
    }
    Some(t)
}
pub(crate) fn probe_timeout_ms() -> u64 {
    2000
}
pub(crate) fn probe_hosts() -> Vec<String> {
    if let Ok(raw) = std::env::var("PULSE_PROBE_HOSTS") {
        let parsed: Vec<String> =
            raw.split(',').filter_map(sanitize_probe_target).map(str::to_owned).collect();
        if !parsed.is_empty() {
            return parsed;
        }
    }
    vec![
        "1.1.1.1:53".to_owned(),
        "8.8.8.8:53".to_owned(),
        "1.1.1.1:443".to_owned(),
        "8.8.8.8:443".to_owned(),
    ]
}
static CACHED_PROBE_TARGETS: OnceLock<Mutex<Vec<std::net::SocketAddr>>> = OnceLock::new();

pub(crate) fn resolve_probe_targets() -> Vec<std::net::SocketAddr> {
    let cache = CACHED_PROBE_TARGETS.get_or_init(|| Mutex::new(Vec::new()));
    let mut guard = cache.lock().unwrap_or_else(|e| e.into_inner());
    if guard.is_empty() {
        for host in probe_hosts() {
            if let Some(addr) = host.to_socket_addrs().ok().and_then(|mut i| i.next()) {
                guard.push(addr);
            }
        }
        if guard.is_empty() {
            for host in ["1.1.1.1:53", "8.8.8.8:53"] {
                if let Some(addr) = host.to_socket_addrs().ok().and_then(|mut i| i.next()) {
                    guard.push(addr);
                }
            }
        }
    }
    guard.clone()
}

fn tcp_rtt_ms() -> Option<f32> {
    for addr in resolve_probe_targets() {
        let start = Instant::now();
        if TcpStream::connect_timeout(&addr, Duration::from_millis(probe_timeout_ms())).is_ok() {
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
pub use crate::modules::network::latency_probe_handle::{probe_snapshot, LatencyProbe};
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
