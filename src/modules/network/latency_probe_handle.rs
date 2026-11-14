//! Latency probe handle.
use std::sync::{mpsc, Mutex};

use crate::modules::network::grade::classify_grade;
use crate::modules::network::latency_probe::ensure_worker;
use crate::modules::network::latency_probe::{dns_flush_state, probe_enabled, state, Cmd, TX};

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
        if let Some(tx) = self.tx.take() {
            let _ = tx.send(Cmd::Shutdown);
            drop(tx);
        }
        if let Some(h) = self.worker.take() {
            let _ = h.join();
        }
        if let Some(slot) = TX.get() {
            *slot.lock().unwrap_or_else(|e| e.into_inner()) = None;
        }
    }
}

pub fn probe_snapshot() -> (
    Option<f32>,
    Option<f32>,
    f32,
    crate::modules::network::model::ConnectionGrade,
    crate::modules::network::latency_probe::DnsFlushState,
) {
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
    let grade = classify_grade(ping, jitter, loss, enabled, 0.0, samples, failures);
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
