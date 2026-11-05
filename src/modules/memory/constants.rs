//! Named constants for the memory module.

/// Maximum sane virtual memory size (1 TiB). Values exceeding this are clamped to RAM.
/// Used to filter out absurd virtual memory values from kernel/32-bit processes.
pub const MAX_SANE_VIRTUAL_BYTES: u64 = 1_u64 << 40;

/// Memory hog threshold percentage. Processes using >= this % of total RAM are flagged as hogs.
pub const MEMORY_HOG_THRESHOLD_PCT: f32 = 15.0;

/// Full scale percentage for heat bar visualization in table cells.
/// Values >= this % fill the entire cell width.
pub const HEAT_BAR_FULL_SCALE_PCT: f32 = 20.0;

/// Chart left-edge anchor tolerance in seconds.
/// When history spans within this tolerance of the window boundary, the left edge is clamped.
pub const CHART_ANCHOR_TOLERANCE_SECS: f64 = 2.0;
