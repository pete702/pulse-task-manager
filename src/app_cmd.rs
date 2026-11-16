//! App-level command enum for clean UI-to-state dispatch.

use crate::app_tabs::ActiveTab;

/// High-level commands emitted by UI chrome and applied by `PulseApp`.
#[derive(Debug, Clone, Copy)]
pub enum AppCommand {
    SetTab(ActiveTab),
    TogglePause,
    RequestProcessScan,
    RequestMemoryTrim,
}
