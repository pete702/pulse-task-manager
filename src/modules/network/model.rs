//! Network telemetry model: adapters, esports grade, history.
//! History itself lives in `history.rs` for the 280-line guard.

use std::sync::Arc;

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ConnectionGrade {
    TournamentReady,
    #[default]
    Stable,
    Probing,
    BufferbloatJitter,
    PacketLoss,
    Offline,
    ProbeDisabled,
}

impl ConnectionGrade {
    #[allow(dead_code)]
    pub fn badge(self) -> &'static str {
        match self {
            ConnectionGrade::TournamentReady => "TOURNAMENT READY",
            ConnectionGrade::Stable => "STABLE",
            ConnectionGrade::Probing => "PROBING...",
            ConnectionGrade::BufferbloatJitter => "BUFFERBLOAT / JITTER",
            ConnectionGrade::PacketLoss => "PACKET LOSS",
            ConnectionGrade::Offline => "OFFLINE",
            ConnectionGrade::ProbeDisabled => "PROBE OFF",
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct NetworkAdapterInfo {
    pub name: Arc<str>,
    pub rx_mbps: f32,
    pub tx_mbps: f32,
    pub total_rx_bytes: u64,
    pub total_tx_bytes: u64,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct NetworkSnapshot {
    pub adapters: Arc<[NetworkAdapterInfo]>,
    pub rx_mbps: f32,
    pub tx_mbps: f32,
    pub total_rx_gib: f32,
    pub total_tx_gib: f32,
    pub ping_ms: Option<f32>,
    pub jitter_ms: Option<f32>,
    pub packet_loss_pct: f32,
    pub grade: ConnectionGrade,
    pub probe_enabled: bool,
}

#[allow(unused_imports)]
pub use crate::modules::network::history::NetworkHistory;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn grade_badges() {
        assert_eq!(ConnectionGrade::TournamentReady.badge(), "TOURNAMENT READY");
        assert_eq!(ConnectionGrade::Stable.badge(), "STABLE");
        assert_eq!(ConnectionGrade::BufferbloatJitter.badge(), "BUFFERBLOAT / JITTER");
        assert_eq!(ConnectionGrade::PacketLoss.badge(), "PACKET LOSS");
        assert_eq!(ConnectionGrade::Offline.badge(), "OFFLINE");
        assert_eq!(ConnectionGrade::ProbeDisabled.badge(), "PROBE OFF");
    }
}
