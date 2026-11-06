//! Connection grade classification split from latency_probe for the 280-line guard.

use crate::modules::network::model::ConnectionGrade;

/// Classifies the connection grade from probe + traffic readings.
/// Before any sample exists (startup), reports `Probing`.
/// Fast outage path: 3+ consecutive timeouts (or >= 50% loss) flips
/// to OFFLINE/PACKET_LOSS without waiting for the full 20-sample window.
/// Traffic-aware: active traffic above 0.25 Mbps proves connectivity, so
/// firewall-blocked probes (e.g. :53 refused) never fake OFFLINE/PACKET_LOSS.
#[allow(dead_code)]
pub fn classify_grade(
    ping_ms: Option<f32>,
    jitter_ms: Option<f32>,
    packet_loss_pct: f32,
    enabled: bool,
    traffic_mbps: f32,
    samples: usize,
    failures: u32,
) -> ConnectionGrade {
    if !enabled {
        return ConnectionGrade::ProbeDisabled;
    }
    if samples == 0 {
        return ConnectionGrade::Probing;
    }
    let jitter = jitter_ms.unwrap_or(0.0);
    if traffic_mbps > 0.25 {
        if ping_ms.is_none() {
            return ConnectionGrade::Stable;
        }
        if jitter >= 15.0 || ping_ms.is_some_and(|p| p >= 120.0) {
            return ConnectionGrade::BufferbloatJitter;
        }
        if ping_ms.is_some_and(|p| p <= 35.0) && jitter <= 5.0 {
            return ConnectionGrade::TournamentReady;
        }
        return ConnectionGrade::Stable;
    }
    if packet_loss_pct >= 99.0 {
        return ConnectionGrade::Offline;
    }
    if failures >= 3 || packet_loss_pct >= 50.0 {
        return if traffic_mbps < 0.05 {
            ConnectionGrade::Offline
        } else {
            ConnectionGrade::PacketLoss
        };
    }
    if ping_ms.is_none() && traffic_mbps < 0.01 {
        return ConnectionGrade::Offline;
    }
    if packet_loss_pct >= 5.0 {
        return ConnectionGrade::PacketLoss;
    }
    if jitter >= 15.0 || ping_ms.is_some_and(|p| p >= 120.0) {
        return ConnectionGrade::BufferbloatJitter;
    }
    if ping_ms.is_some_and(|p| p <= 35.0) && jitter <= 5.0 && packet_loss_pct <= 0.0 {
        return ConnectionGrade::TournamentReady;
    }
    ConnectionGrade::Stable
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn grade_classification_rules() {
        assert_eq!(
            classify_grade(Some(20.0), Some(2.0), 0.0, false, 10.0, 10, 0),
            ConnectionGrade::ProbeDisabled
        );
        assert_eq!(classify_grade(None, None, 0.0, true, 0.0, 0, 0), ConnectionGrade::Probing);
        assert_eq!(classify_grade(None, None, 0.0, true, 0.0, 5, 0), ConnectionGrade::Offline);
        assert_eq!(classify_grade(None, None, 15.0, true, 0.0, 20, 3), ConnectionGrade::Offline);
        // Traffic-aware: flowing traffic proves the link, blocked probes are ignored.
        assert_eq!(classify_grade(None, None, 15.0, true, 5.0, 20, 3), ConnectionGrade::Stable);
        assert_eq!(
            classify_grade(Some(50.0), Some(1.0), 99.0, true, 10.0, 10, 0),
            ConnectionGrade::Stable
        );
        assert_eq!(
            classify_grade(Some(50.0), Some(1.0), 5.0, true, 0.0, 10, 0),
            ConnectionGrade::PacketLoss
        );
        assert_eq!(
            classify_grade(Some(50.0), Some(15.0), 0.0, true, 10.0, 10, 0),
            ConnectionGrade::BufferbloatJitter
        );
        assert_eq!(
            classify_grade(Some(125.0), Some(1.0), 0.0, true, 10.0, 10, 0),
            ConnectionGrade::BufferbloatJitter
        );
        assert_eq!(
            classify_grade(Some(20.0), Some(3.0), 0.0, true, 10.0, 10, 0),
            ConnectionGrade::TournamentReady
        );
        assert_eq!(
            classify_grade(Some(60.0), Some(8.0), 0.0, true, 10.0, 10, 0),
            ConnectionGrade::Stable
        );
        assert_eq!(classify_grade(None, None, 0.0, true, 5.0, 10, 0), ConnectionGrade::Stable);
    }
    #[test]
    fn firewall_blocked_probes_never_fake_outage_when_traffic_flows() {
        assert_ne!(classify_grade(None, None, 100.0, true, 2.0, 20, 5), ConnectionGrade::Offline);
        assert_ne!(
            classify_grade(None, None, 100.0, true, 2.0, 20, 5),
            ConnectionGrade::PacketLoss
        );
        assert_eq!(classify_grade(None, None, 100.0, true, 2.0, 20, 5), ConnectionGrade::Stable);
        assert_eq!(classify_grade(None, None, 100.0, true, 0.0, 20, 5), ConnectionGrade::Offline);
    }
}
