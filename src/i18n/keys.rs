//! Translation key definitions for PULSE i18n.

/// Translation key type for type-safe lookups.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    // Top bar / window chrome
    AppTitle,
    VersionPrefix,
    UptimeLabel,

    // Sidebar tabs
    TabMemory,
    TabCpu,
    TabStorage,
    TabNetwork,
    TabSystem,

    // Central panel
    AwaitingMemoryModel,

    // Self-test
    #[cfg(test)]
    SelfTestPassedHeadless,
    SelfTestPassedLive,
    SelfTestFailed,

    // Status bar
    StatusReady,

    // Memory module - Card headers
    #[cfg(test)]
    PhysicalRamTitle,
    #[cfg(test)]
    SwapCommitTitle,

    // Memory module - Grid labels (Card 1: RAM)
    #[cfg(test)]
    UsedTotal,
    #[cfg(test)]
    AvailableFree,
    #[cfg(test)]
    CachedStandby,
    #[cfg(test)]
    PeakSession,

    // Memory module - Grid labels (Card 2: Swap/Commit)
    #[cfg(test)]
    SwapUsedTotal,
    #[cfg(test)]
    CommitChargeLimit,
    #[cfg(test)]
    CommitPercent,

    // Kernel mini-tiles
    #[cfg(test)]
    KernelPool,
    #[cfg(test)]
    CommitCharge,
    #[cfg(test)]
    OsActivityTitle,

    // Chart
    MemoryHistoryTitle,
    RamPercent,
}
