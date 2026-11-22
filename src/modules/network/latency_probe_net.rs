//! Low-level networking helpers for the latency probe.
use std::net::{TcpStream, ToSocketAddrs};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Mutex, OnceLock,
};
use std::time::{Duration, Instant};

pub(crate) static PROBE_IN_FLIGHT: AtomicBool = AtomicBool::new(false);
static PROBE_SHUTDOWN: AtomicBool = AtomicBool::new(false);

pub(crate) fn request_probe_shutdown() {
    PROBE_SHUTDOWN.store(true, Ordering::Release);
}

pub(crate) fn reset_probe_shutdown() {
    PROBE_SHUTDOWN.store(false, Ordering::Release);
}

fn probe_shutdown_requested() -> bool {
    PROBE_SHUTDOWN.load(Ordering::Acquire)
}

static PROBE_IN_FLIGHT_SINCE: OnceLock<Mutex<Option<Instant>>> = OnceLock::new();

pub(crate) fn in_flight_since() -> &'static Mutex<Option<Instant>> {
    PROBE_IN_FLIGHT_SINCE.get_or_init(|| Mutex::new(None))
}

pub(crate) struct InFlightGuard;
impl Drop for InFlightGuard {
    fn drop(&mut self) {
        PROBE_IN_FLIGHT.store(false, Ordering::Release);
        *in_flight_since().lock().unwrap_or_else(|e| e.into_inner()) = None;
    }
}

pub(crate) fn is_probe_watchdog_expired() -> bool {
    if !PROBE_IN_FLIGHT.load(Ordering::Acquire) {
        return false;
    }
    let guard = in_flight_since().lock().unwrap_or_else(|e| e.into_inner());
    guard.map(|t| t.elapsed().as_millis() >= 5000).unwrap_or(false)
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
    let cached = {
        let guard = cache.lock().unwrap_or_else(|e| e.into_inner());
        guard.clone()
    };
    if !cached.is_empty() {
        return cached;
    }
    let mut resolved = Vec::new();
    for host in probe_hosts() {
        if let Some(addr) = host.to_socket_addrs().ok().and_then(|mut i| i.next()) {
            resolved.push(addr);
        }
    }
    if resolved.is_empty() {
        for host in ["1.1.1.1:53", "8.8.8.8:53"] {
            if let Some(addr) = host.to_socket_addrs().ok().and_then(|mut i| i.next()) {
                resolved.push(addr);
            }
        }
    }
    let mut guard = cache.lock().unwrap_or_else(|e| e.into_inner());
    if guard.is_empty() {
        *guard = resolved.clone();
    }
    guard.clone()
}

pub(crate) fn tcp_rtt_ms() -> Option<f32> {
    for addr in resolve_probe_targets() {
        if probe_shutdown_requested() {
            return None;
        }
        let start = Instant::now();
        if TcpStream::connect_timeout(&addr, Duration::from_millis(probe_timeout_ms())).is_ok() {
            return Some(start.elapsed().as_secs_f32() * 1000.0);
        }
    }
    None
}
