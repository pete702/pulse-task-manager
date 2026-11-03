//! Translation key definitions for PULSE i18n.

/// Translation key type for type-safe lookups.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    // Top bar / window chrome
    AppTitle,
    VersionPrefix,
    UptimeLabel,
    SelfDiagLabel,
    LocaleIndicator,

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
    StatusInfo,

    // Memory module - Card headers
    PhysicalRamTitle,
    SwapCommitTitle,

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
    SystemCache,
    KernelPool,
    CommitCharge,
    OsActivityTitle,

    // Chart
    MemoryHistoryTitle,
    RamPercent,
    SwapPercent,

    // Memory module - legacy keys (keep for backward compat)
    MemoryTotal,
    MemoryAvailable,
    MemoryUsed,
    SwapTotal,
    SwapUsed,
    SwapFree,
    ClearCacheButton,
    CacheCleared,
    CacheClearFailed,

    // Process module (v0.0.8+)
    ProcessTitle,
    ProcessSearch,
    ProcessGrouped,
    ProcessName,
    ProcessPid,
    ProcessRam,
    ProcessVirtual,
    ProcessPct,
    ProcessCount,
    ProcessHog,
    ProcessSpikeEvent,
}
