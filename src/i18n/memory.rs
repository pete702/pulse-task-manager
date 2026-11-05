//! Memory module translations for PULSE i18n.
//! **100% English Code** — Ukrainian translations removed.

use super::{Key, Lang};

impl Lang {
    /// Translates a key for this language.
    /// This covers memory module keys and process table keys.
    pub(super) fn translate_memory(self, key: Key) -> Option<&'static str> {
        match (self, key) {
            // Memory module - Card headers
            (Lang::En, Key::PhysicalRamTitle) => Some("• PHYSICAL MEMORY (RAM)"),
            (Lang::En, Key::SwapCommitTitle) => Some("• SWAP FILE & COMMIT"),
            (Lang::En, Key::MemoryCompositionTitle) => Some("● MEMORY COMPOSITION"),

            // Memory module - Grid labels (Card 1: RAM)
            (Lang::En, Key::UsedTotal) => Some("Used / Total"),
            (Lang::En, Key::AvailableFree) => Some("Available / Free"),
            (Lang::En, Key::CachedStandby) => Some("Cached (System)"),
            (Lang::En, Key::PeakSession) => Some("Peak (Session)"),

            // Memory module - Grid labels (Card 2: Swap/Commit)
            (Lang::En, Key::SwapUsedTotal) => Some("Swap (Pagefile) Used"),
            (Lang::En, Key::CommitChargeLimit) => Some("Commit Charge / Limit"),
            (Lang::En, Key::CommitPercent) => Some("Commit %"),

            // Kernel mini-tiles
            (Lang::En, Key::KernelPool) => Some("Kernel Pool"),
            (Lang::En, Key::CommitCharge) => Some("Committed"),
            (Lang::En, Key::OsActivityTitle) => Some("OS Activity"),

            // Chart
            (Lang::En, Key::MemoryHistoryTitle) => Some("● MEMORY UTILIZATION HISTORY"),
            (Lang::En, Key::RamPercent) => Some("RAM %"),
            (Lang::En, Key::SwapPercent) => Some("Swap %"),

            _ => None,
        }
    }
}
