//! Translation key definitions for PULSE i18n.

/// Translation key type for type-safe lookups.
#[allow(dead_code)]
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
    SelfTestPassedHeadless,
    SelfTestPassedLive,
    SelfTestFailed,

    // Status bar
    StatusReady,

    // Memory module - Card headers
    PhysicalRamTitle,
    SwapCommitTitle,
    MemoryCompositionTitle,

    // Memory module - Grid labels (Card 1: RAM)
    UsedTotal,
    AvailableFree,
    CachedStandby,
    PeakSession,

    // Memory module - Grid labels (Card 2: Swap/Commit)
    SwapUsedTotal,
    CommitChargeLimit,
    CommitPercent,

    // Kernel mini-tiles
    KernelPool,
    CommitCharge,
    OsActivityTitle,

    // Chart
    MemoryHistoryTitle,
    RamPercent,
    SwapPercent,
}
