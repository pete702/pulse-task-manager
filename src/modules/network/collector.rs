//! Network collector: adapter rates from sysinfo counters.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use sysinfo::Networks;

use crate::modules::network::grade::classify_grade;
use crate::modules::network::latency_probe::{
    maybe_trigger_probe, probe_failures, probe_sample_count, probe_state,
};
use crate::modules::network::model::{NetworkAdapterInfo, NetworkSnapshot};

const GIB_BYTES: f64 = 1_073_741_824.0;

/// True for virtual/loopback pseudo-interfaces that must not be shown.
#[allow(dead_code)]
fn is_virtual(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    lower.contains("loopback")
        || lower.contains("vethernet")
        || lower.contains("wsl")
        || lower.contains("teredo")
        || lower.contains("isatap")
        || lower.contains("npcap")
        || lower.contains("wfp")
        || lower.contains("lightweight filter")
        || lower.contains("qos packet")
        || lower.contains("virtualbox")
        || lower.contains("vmware")
        || lower.contains("default switch")
}

/// Bytes-per-interval to megabits/sec, finite-clamped.
#[allow(dead_code)]
fn compute_mbps(delta_bytes: u64, elapsed_secs: f32) -> f32 {
    let rate = delta_bytes as f32 * 8.0 / (elapsed_secs.max(0.2) * 1_000_000.0);
    if rate.is_finite() {
        rate.clamp(0.0, 10_000_000.0)
    } else {
        0.0
    }
}

/// Pure per-adapter tick math: rate from lifetime delta, session from baseline.
/// `prev_lifetime` must be the raw sysinfo counter, never session bytes.
#[allow(dead_code)]
fn rate_and_session(prev_lifetime: u64, baseline: u64, now: u64, elapsed: f32) -> (f32, u64) {
    let mbps = compute_mbps(now.saturating_sub(prev_lifetime), elapsed);
    let session = now.saturating_sub(baseline);
    (mbps, session)
}

/// Stateful network collector over sysinfo `Networks`.
#[allow(dead_code)]
pub struct NetworkCollector {
    networks: Networks,
    last_refresh: Option<Instant>,
    prev: HashMap<Arc<str>, (u64, u64)>,
    adapter_baseline: HashMap<Arc<str>, (u64, u64)>,
    scratch: HashMap<Arc<str>, (u64, u64)>,
}

impl NetworkCollector {
    /// Creates a collector; first `collect` primes the counter baseline.
    #[allow(dead_code)]
    pub fn new() -> Self {
        let mut networks = Networks::new();
        networks.refresh_list();
        Self {
            networks,
            last_refresh: None,
            prev: HashMap::new(),
            adapter_baseline: HashMap::new(),
            scratch: HashMap::new(),
        }
    }

    /// Collects one network snapshot (call once per worker tick).
    #[allow(dead_code)]
    pub fn collect(&mut self) -> NetworkSnapshot {
        maybe_trigger_probe();
        let now = Instant::now();
        let first = self.last_refresh.is_none();
        let (elapsed, adapters, rx_total, tx_total) = if first {
            self.networks.refresh_list();
            let (adapters, rx_total, tx_total) = self.read_adapters(0.0);
            self.last_refresh = Some(now);
            (0.0, adapters, rx_total, tx_total)
        } else {
            let elapsed = now.duration_since(self.last_refresh.unwrap_or(now)).as_secs_f32();
            self.networks.refresh();
            let out = self.read_adapters(elapsed);
            self.last_refresh = Some(now);
            (elapsed, out.0, out.1, out.2)
        };
        let _ = elapsed;
        let rx_mbps: f32 = adapters.iter().map(|a| a.rx_mbps).sum();
        let tx_mbps: f32 = adapters.iter().map(|a| a.tx_mbps).sum();
        let (ping_ms, jitter_ms, packet_loss_pct, enabled) = probe_state();
        let grade = classify_grade(
            ping_ms,
            jitter_ms,
            packet_loss_pct,
            enabled,
            rx_mbps + tx_mbps,
            probe_sample_count(),
            probe_failures(),
        );
        // Single baseline: session is the sum of per-adapter session bytes
        // already computed in read_adapters (raw - adapter_baseline).
        let session_rx = rx_total;
        let session_tx = tx_total;
        NetworkSnapshot {
            adapters: Arc::from(adapters),
            rx_mbps,
            tx_mbps,
            total_rx_gib: (session_rx as f64 / GIB_BYTES) as f32,
            total_tx_gib: (session_tx as f64 / GIB_BYTES) as f32,
            ping_ms,
            jitter_ms,
            packet_loss_pct,
            grade,
            probe_enabled: enabled,
        }
    }

    /// Reads filtered adapters and updates the counter baseline.
    /// `prev` always holds raw lifetime counters; `total_*_bytes` hold session.
    #[allow(dead_code)]
    fn read_adapters(&mut self, elapsed: f32) -> (Vec<NetworkAdapterInfo>, u64, u64) {
        let mut adapters: Vec<NetworkAdapterInfo> = Vec::new();
        // Reuse scratch capacity: clear instead of allocating a fresh map.
        self.scratch.clear();
        for (name, data) in self.networks.list().iter() {
            if is_virtual(name.as_str()) {
                continue;
            }
            let rx_now = data.received();
            let tx_now = data.transmitted();
            let prev = self.prev.get(name.as_str()).copied().unwrap_or((rx_now, tx_now));
            let base =
                self.adapter_baseline.entry(Arc::from(name.as_str())).or_insert((rx_now, tx_now));
            let (rx_mbps, rx_session) = rate_and_session(prev.0, base.0, rx_now, elapsed);
            let (tx_mbps, tx_session) = rate_and_session(prev.1, base.1, tx_now, elapsed);
            adapters.push(NetworkAdapterInfo {
                name: Arc::from(name.as_str()),
                rx_mbps: if elapsed > 0.0 { rx_mbps } else { 0.0 },
                tx_mbps: if elapsed > 0.0 { tx_mbps } else { 0.0 },
                total_rx_bytes: rx_session,
                total_tx_bytes: tx_session,
            });
            self.scratch.insert(Arc::from(name.as_str()), (rx_now, tx_now));
        }
        adapters.sort_by(|a, b| a.name.cmp(&b.name));
        let rx_total: u64 = adapters.iter().map(|a| a.total_rx_bytes).sum();
        let tx_total: u64 = adapters.iter().map(|a| a.total_tx_bytes).sum();
        std::mem::swap(&mut self.prev, &mut self.scratch);
        (adapters, rx_total, tx_total)
    }
}

impl Default for NetworkCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn virtual_interface_filter() {
        assert!(is_virtual("Loopback Pseudo-Interface 1"));
        assert!(is_virtual("vEthernet (Default Switch)"));
        assert!(is_virtual("WSL (Hyper-V firewall)"));
        assert!(is_virtual("Teredo Tunneling Pseudo-Interface"));
        assert!(is_virtual("ISATAP interface"));
        assert!(is_virtual("Npcap Loopback Adapter"));
        assert!(is_virtual("WFP Lightweight Filter"));
        assert!(is_virtual("VirtualBox Host-Only Ethernet Adapter"));
        assert!(is_virtual("VMware Network Adapter VMnet1"));
        assert!(!is_virtual("Ethernet"));
        assert!(!is_virtual("Wi-Fi"));
        assert!(!is_virtual("Intel(R) Ethernet"));
    }

    #[test]
    fn mbps_math_and_clamp() {
        // 1 MB in 1 s = 8 Mbps
        assert!((compute_mbps(1_000_000, 1.0) - 8.0).abs() < 0.01);
        assert_eq!(compute_mbps(0, 1.0), 0.0);
        assert_eq!(compute_mbps(u64::MAX, f32::INFINITY), 0.0);
        assert!(compute_mbps(1_000_000, 0.0) >= 0.0);
    }

    #[test]
    fn first_collect_priming() {
        let mut c = NetworkCollector::new();
        let snap = c.collect();
        assert_eq!(snap.rx_mbps, 0.0);
        assert_eq!(snap.tx_mbps, 0.0);
        assert!(snap.total_rx_gib >= 0.0);
        let _ = crate::modules::network::latency_probe::probe_enabled();
    }

    #[test]
    fn three_ticks_nonzero_baseline_session_and_rate() {
        let gib: u64 = 1_073_741_824;
        let base = 10 * gib;
        let mut prev = base;
        // Tick 1: +1_000_000 bytes in 1s -> 8 Mbps, session = 1_000_000.
        let (m1, s1) = rate_and_session(prev, base, base + 1_000_000, 1.0);
        assert!((m1 - 8.0).abs() < 0.05);
        assert_eq!(s1, 1_000_000);
        prev = base + 1_000_000;
        // Tick 2: +2_000_000 bytes in 1s -> 16 Mbps, session = 3_000_000 (not lifetime).
        let (m2, s2) = rate_and_session(prev, base, prev + 2_000_000, 1.0);
        assert!((m2 - 16.0).abs() < 0.05);
        assert_eq!(s2, 3_000_000);
        prev = base + 3_000_000;
        // Tick 3: idle -> 0 Mbps, session stays 3_000_000.
        let (m3, s3) = rate_and_session(prev, base, prev, 1.0);
        assert_eq!(m3, 0.0);
        assert_eq!(s3, 3_000_000);
    }
    #[test]
    fn mid_session_hot_plug_single_baseline() {
        // Existing adapter accumulates session via its own baseline.
        let (rx_mbps, rx_session) = rate_and_session(1_000, 0, 2_000, 1.0);
        assert_eq!(rx_session, 2_000);
        assert!(rx_mbps > 0.0);
        // Hot-plugged adapter appears with baseline == now: zero rate/session.
        let (new_mbps, new_session) = rate_and_session(50_000_000, 50_000_000, 50_000_000, 1.0);
        assert_eq!(new_mbps, 0.0);
        assert_eq!(new_session, 0);
        // Total session is the sum, unaffected by the new adapter baseline.
        assert_eq!(rx_session + new_session, 2_000);
    }
}
