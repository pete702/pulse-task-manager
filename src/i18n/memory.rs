//! Memory module translations for PULSE i18n.
//! **100% English Code** -- Ukrainian translations removed.

use super::{Key, Lang};

impl Lang {
    /// Translates a key for this language.
    /// This covers memory module keys and process table keys.
    pub(super) fn translate_memory(self, key: Key) -> Option<&'static str> {
        match (self, key) {
            // Memory module - Card headers
            #[cfg(test)]
            (Lang::En, Key::PhysicalRamTitle) => Some("* PHYSICAL MEMORY (RAM)"),
            #[cfg(test)]
            (Lang::En, Key::SwapCommitTitle) => Some("* SWAP FILE & COMMIT"),

            // Memory module - Grid labels (Card 1: RAM)
            #[cfg(test)]
            (Lang::En, Key::UsedTotal) => Some("Used / Total"),
            #[cfg(test)]
            (Lang::En, Key::AvailableFree) => Some("Available / Free"),
            #[cfg(test)]
            (Lang::En, Key::CachedStandby) => Some("Cached (System)"),
            #[cfg(test)]
            (Lang::En, Key::PeakSession) => Some("Peak (Session)"),

            // Memory module - Grid labels (Card 2: Swap/Commit)
            #[cfg(test)]
            (Lang::En, Key::SwapUsedTotal) => Some("Swap (Pagefile) Used"),
            #[cfg(test)]
            (Lang::En, Key::CommitChargeLimit) => Some("Commit Charge / Limit"),
            #[cfg(test)]
            (Lang::En, Key::CommitPercent) => Some("Commit %"),

            // Kernel mini-tiles
            #[cfg(test)]
            (Lang::En, Key::KernelPool) => Some("Kernel Pool"),
            #[cfg(test)]
            (Lang::En, Key::CommitCharge) => Some("Committed"),
            #[cfg(test)]
            (Lang::En, Key::OsActivityTitle) => Some("OS Activity"),

            // Chart
            (Lang::En, Key::MemoryHistoryTitle) => Some("* MEMORY UTILIZATION HISTORY"),
            (Lang::En, Key::RamPercent) => Some("RAM %"),

            _ => None,
        }
    }
}
