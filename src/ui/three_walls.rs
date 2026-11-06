//! Three Walls badge: at-a-glance Windows memory pressure indicator.

use eframe::egui;

use crate::core::SystemSnapshot;
use crate::modules::memory::formatters::format_bytes;
use crate::modules::memory::model::MemorySnapshot;
use crate::theme;

/// Honest Three Walls badge state: optimal, elevated, memory wall, or unconfirmed telemetry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WallState {
    Unconfirmed,
    Optimal,
    Elevated,
    MemoryWall,
}

impl WallState {
    #[allow(dead_code)]
    pub fn label(self) -> &'static str {
        match self {
            Self::Unconfirmed => "[ ⏳ INITIALIZING TELEMETRY ]",
            Self::Optimal => "[ ● SYSTEM OPTIMAL ]",
            Self::Elevated => "[ ● RAM ELEVATED ]",
            Self::MemoryWall => "[ ▲ MEMORY WALL ]",
        }
    }

    #[allow(dead_code)]
    pub fn color(self) -> egui::Color32 {
        match self {
            Self::Unconfirmed => theme::TEXT_MUTED,
            Self::Optimal => theme::OPTIMAL_EMERALD,
            Self::Elevated => theme::WARNING_AMBER,
            Self::MemoryWall => theme::CRITICAL_CORAL,
        }
    }
}

fn evaluate_memory(mem: &MemorySnapshot) -> WallState {
    // Never claim "optimal" without confirmed kernel/RAM telemetry.
    if mem.snapshot_id == 0 || mem.ram_total_bytes == 0 || !mem.kernel_metrics_valid {
        return WallState::Unconfirmed;
    }
    let (ram, commit) = (mem.ram_usage_percent(), mem.commit_charge_pct());
    if ram >= 85.0 {
        WallState::MemoryWall
    } else if ram >= 70.0 || commit >= 80.0 {
        WallState::Elevated
    } else {
        WallState::Optimal
    }
}

/// Evaluates the badge state for a telemetry snapshot.
pub fn evaluate(snap: &SystemSnapshot) -> WallState {
    match snap.memory.as_ref() {
        Some(mem) => evaluate_memory(mem),
        None => WallState::Unconfirmed,
    }
}

#[allow(dead_code)]
fn hover_text(snap: &SystemSnapshot) -> String {
    let Some(ref m) = snap.memory else {
        return "Awaiting memory telemetry".to_owned();
    };
    format!(
        "RAM: {:.1}%\nCommit: {:.1}% ({} / {})",
        m.ram_usage_percent(),
        m.commit_charge_pct(),
        format_bytes(m.commit_total_bytes),
        format_bytes(m.commit_limit_bytes)
    )
}

/// Renders the Three Walls badge into the top bar.
#[allow(dead_code)]
pub fn render_badge(ui: &mut egui::Ui, snap: &SystemSnapshot) {
    let state = evaluate(snap);
    let resp =
        ui.label(egui::RichText::new(state.label()).monospace().size(11.0).color(state.color()));
    if resp.hovered() {
        resp.on_hover_text(hover_text(snap));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snapshot(ram_used: u64, commit_used: u64) -> SystemSnapshot {
        let mem = MemorySnapshot {
            ram_total_bytes: 100,
            ram_used_bytes: ram_used,
            commit_total_bytes: commit_used,
            commit_limit_bytes: 100,
            snapshot_id: 1,
            kernel_metrics_valid: true,
            ..Default::default()
        };
        SystemSnapshot::new(1, 10.0, Some(mem))
    }

    #[test]
    fn test_memory_wall_state() {
        assert_eq!(evaluate(&snapshot(90, 10)), WallState::MemoryWall);
        assert_eq!(WallState::MemoryWall.label(), "[ ▲ MEMORY WALL ]");
    }

    #[test]
    fn test_ram_elevated_state() {
        assert_eq!(evaluate(&snapshot(75, 10)), WallState::Elevated);
        assert_eq!(evaluate(&snapshot(50, 85)), WallState::Elevated);
        assert_eq!(WallState::Elevated.label(), "[ ● RAM ELEVATED ]");
    }

    #[test]
    fn test_optimal_state_and_missing_memory() {
        assert_eq!(evaluate(&snapshot(50, 50)), WallState::Optimal);
        assert_eq!(evaluate(&SystemSnapshot::minimal(1, 10.0)), WallState::Unconfirmed);
        assert_eq!(WallState::Optimal.label(), "[ ● SYSTEM OPTIMAL ]");
        assert_eq!(WallState::Unconfirmed.label(), "[ ⏳ INITIALIZING TELEMETRY ]");
    }

    #[test]
    fn test_unconfirmed_until_telemetry_is_valid() {
        let mut mem = MemorySnapshot {
            ram_total_bytes: 100,
            ram_used_bytes: 10,
            commit_total_bytes: 10,
            commit_limit_bytes: 100,
            snapshot_id: 0,
            kernel_metrics_valid: true,
            ..Default::default()
        };
        assert_eq!(evaluate_memory(&mem), WallState::Unconfirmed);
        mem.snapshot_id = 3;
        mem.kernel_metrics_valid = false;
        assert_eq!(evaluate_memory(&mem), WallState::Unconfirmed);
        mem.kernel_metrics_valid = true;
        assert_eq!(evaluate_memory(&mem), WallState::Optimal);
    }
}
