//! Memory module translations for PULSE i18n.
//! **100% English Code** — Ukrainian translations removed.

use super::{Key, Lang};

impl Lang {
    /// Translates a key for this language.
    /// This covers memory module keys and process table keys.
    pub(super) fn translate_memory(self, key: Key) -> Option<&'static str> {
        match (self, key) {
            // Memory module - Card headers
            (Lang::En, Key::PhysicalRamTitle) => Some("◈ PHYSICAL MEMORY (RAM)"),
            (Lang::En, Key::SwapCommitTitle) => Some("▣ SWAP FILE & COMMIT"),

            // Memory module - Grid labels (Card 1: RAM)
            (Lang::En, Key::UsedTotal) => Some("Used / Total"),
            (Lang::En, Key::AvailableFree) => Some("Available / Free"),
            (Lang::En, Key::CachedStandby) => Some("Cached (Standby)"),
            (Lang::En, Key::PeakSession) => Some("Peak (Session)"),

            // Memory module - Grid labels (Card 2: Swap/Commit)
            (Lang::En, Key::SwapUsedTotal) => Some("Swap Used / Total"),
            (Lang::En, Key::CommitChargeLimit) => Some("Commit Charge / Limit"),
            (Lang::En, Key::CommitPercent) => Some("Commit %"),

            // Kernel mini-tiles
            (Lang::En, Key::SystemCache) => Some("System Cache"),
            (Lang::En, Key::KernelPool) => Some("Kernel Pool"),
            (Lang::En, Key::CommitCharge) => Some("Committed"),
            (Lang::En, Key::OsActivityTitle) => Some("OS Activity"),

            // Chart
            (Lang::En, Key::MemoryHistoryTitle) => Some("● MEMORY UTILIZATION HISTORY"),
            (Lang::En, Key::RamPercent) => Some("RAM %"),
            (Lang::En, Key::SwapPercent) => Some("Swap %"),

            // Memory module - legacy keys (keep for backward compat)
            (Lang::En, Key::MemoryTotal) => Some("Total RAM"),
            (Lang::En, Key::MemoryAvailable) => Some("Available"),
            (Lang::En, Key::MemoryUsed) => Some("Used"),
            (Lang::En, Key::SwapTotal) => Some("Total Swap"),
            (Lang::En, Key::SwapUsed) => Some("Swap Used"),
            (Lang::En, Key::SwapFree) => Some("Swap Free"),
            (Lang::En, Key::ClearCacheButton) => Some("Clear Cache"),
            (Lang::En, Key::CacheCleared) => Some("Cache cleared"),
            (Lang::En, Key::CacheClearFailed) => Some("Cache clear failed"),

            // Process module (v0.0.8+)
            (Lang::En, Key::ProcessTitle) => Some("◈ RAM PROCESSES"),
            (Lang::En, Key::ProcessSearch) => Some("Search by name or PID..."),
            (Lang::En, Key::ProcessGrouped) => Some(" (grouped)"),
            (Lang::En, Key::ProcessName) => Some("Name"),
            (Lang::En, Key::ProcessPid) => Some("PID"),
            (Lang::En, Key::ProcessRam) => Some("RAM"),
            (Lang::En, Key::ProcessVirtual) => Some("Virtual"),
            (Lang::En, Key::ProcessPct) => Some("% RAM"),
            (Lang::En, Key::ProcessCount) => Some("Count"),
            (Lang::En, Key::ProcessHog) => Some("⚠ HOG"),
            (Lang::En, Key::ProcessSpikeEvent) => Some("MEMORY SPIKE: {} ({}%) — {} (PID {})"),

            _ => None,
        }
    }
}
