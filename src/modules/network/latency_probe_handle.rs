//! Latency probe handle.
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{mpsc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use crate::modules::network::grade::classify_grade;
use crate::modules::network::latency_probe::ensure_worker;
use crate::modules::network::latency_probe::{dns_flush_state, probe_enabled, state, Cmd, TX};

type ProbeSnapshot = (
    Option<f32>,
    Option<f32>,
    f32,
    crate::modules::network::model::ConnectionGrade,
    crate::modules::network::latency_probe::DnsFlushState,
);

pub struct LatencyProbe {
    tx: Option<mpsc::Sender<Cmd>>,
    worker: Option<std::thread::JoinHandle<()>>,
}

impl LatencyProbe {
    pub fn new() -> Self {
        let tx = ensure_worker();
        let worker = TX
            .get_or_init(|| Mutex::new(None))
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .as_mut()
            .and_then(|(_, h)| h.take());
        Self { tx, worker }
    }
}

impl Drop for LatencyProbe {
    fn drop(&mut self) {
        crate::modules::network::latency_probe::request_probe_shutdown();
        if let Some(tx) = self.tx.take() {
            let _ = tx.send(Cmd::Shutdown);
            drop(tx);
        }
        if let Some(h) = self.worker.take() {
            let deadline = Instant::now() + Duration::from_millis(50);
            while !h.is_finished() && Instant::now() < deadline {
                thread::sleep(Duration::from_millis(5));
            }
            if h.is_finished() {
                let _ = h.join();
            }
        }
        if let Some(slot) = TX.get() {
            *slot.lock().unwrap_or_else(|e| e.into_inner()) = None;
        }
    }
}

static PROBE_TRAFFIC_BITS: AtomicU32 = AtomicU32::new(0);

/// Publishes the current rx+tx rate for callers using the no-argument snapshot.
pub fn set_probe_traffic_mbps(mbps: f32) {
    PROBE_TRAFFIC_BITS.store(mbps.max(0.0).to_bits(), Ordering::Relaxed);
}

pub fn probe_snapshot() -> ProbeSnapshot {
    let mbps = f32::from_bits(PROBE_TRAFFIC_BITS.load(Ordering::Relaxed));
    probe_snapshot_with_traffic(mbps)
}

pub fn probe_snapshot_with_traffic(traffic_mbps: f32) -> ProbeSnapshot {
    let (ping, jitter, loss, samples, failures, enabled) = {
        let guard = state().lock().unwrap_or_else(|e| e.into_inner());
        let loss = if guard.recent.is_empty() {
            0.0
        } else {
            let fails = guard.recent.iter().filter(|ok| !**ok).count() as f32;
            fails / guard.recent.len() as f32 * 100.0
        };
        (
            guard.ping_ms,
            guard.jitter_ms,
            loss,
            guard.recent.len(),
            guard.consecutive_failures,
            probe_enabled(),
        )
    };
    let grade = classify_grade(ping, jitter, loss, enabled, traffic_mbps, samples, failures);
    let dns = dns_flush_state();
    (ping, jitter, loss, grade, dns)
}

pub fn run_flush() {
    use std::process::Command;
    let mut cmd = {
        #[cfg(target_os = "windows")]
        {
            let exe = std::env::var_os("SystemRoot")
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|| std::path::PathBuf::from(r"C:\Windows"))
                .join("System32")
                .join("ipconfig.exe");
            Command::new(exe)
        }
        #[cfg(not(target_os = "windows"))]
        {
            Command::new("ipconfig")
        }
    };
    cmd.args(["/flushdns"]);
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000);
    }
    std::thread::spawn(move || {
        let ok = cmd.status().is_ok_and(|s| s.success());
        let mut g = crate::modules::network::latency_probe::dns_slot()
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        g.0 = if ok {
            crate::modules::network::latency_probe::DnsFlushState::Success
        } else {
            crate::modules::network::latency_probe::DnsFlushState::Failed
        };
        g.1 = Some(std::time::Instant::now());
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::network::model::ConnectionGrade;

    #[test]
    fn active_traffic_with_failed_probes_is_not_offline() {
        // Failed probes (all samples failed) plus real rx+tx traffic proves the link.
        let grade = classify_grade(None, None, 100.0, true, 9.0, 20, 5);
        assert_eq!(grade, ConnectionGrade::Stable);
        assert_ne!(grade, ConnectionGrade::Offline);
        assert_ne!(grade, ConnectionGrade::PacketLoss);
        // The old hardcoded 0.0 traffic still reports a false outage.
        assert_eq!(classify_grade(None, None, 100.0, true, 0.0, 20, 5), ConnectionGrade::Offline);
    }
}
