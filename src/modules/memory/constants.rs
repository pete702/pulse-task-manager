//! Named constants for the memory module.

/// Maximum sane virtual memory size (1 TiB). Values exceeding this are clamped to RAM.
/// Used to filter out absurd virtual memory values from kernel/32-bit processes.
pub const MAX_SANE_VIRTUAL_BYTES: u64 = 1_u64 << 40;
