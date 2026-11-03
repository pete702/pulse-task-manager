//! English translation tests for PULSE i18n keys.

use crate::i18n::{set_lang, Key, Lang};

#[test]
fn test_key_translation_en() {
    set_lang(Lang::En);
    // Top bar / window chrome
    assert_eq!(Key::AppTitle.t(), "⚡ PULSE // TELEMETRY");
    assert_eq!(Key::VersionPrefix.t(), "v");
    assert_eq!(Key::UptimeLabel.t(), "Uptime");
    assert_eq!(Key::SelfDiagLabel.t(), "Self-Diag");
    assert_eq!(Key::LocaleIndicator.t(), "Locale");

    // Sidebar tabs
    assert_eq!(Key::TabMemory.t(), "◈ Memory (RAM)");
    assert_eq!(Key::TabCpu.t(), "▣ Processor (CPU)");
    assert_eq!(Key::TabStorage.t(), "◧ Storage (Disks)");
    assert_eq!(Key::TabNetwork.t(), "⇅ Network & Ping");
    assert_eq!(Key::TabSystem.t(), "⚙ System & GPU");

    // Central panel
    assert_eq!(
        Key::AwaitingMemoryModel.t(),
        "Telemetry core active. Awaiting memory model..."
    );

    // Self-test
    assert_eq!(
        Key::SelfTestPassedHeadless.t(),
        "SELF-TEST PASSED (HEADLESS CI)"
    );
    assert_eq!(Key::SelfTestPassedLive.t(), "SELF-TEST PASSED (LIVE GUI)");
    assert_eq!(Key::SelfTestFailed.t(), "SELF-TEST FAILED");

    // Status bar
    assert_eq!(Key::StatusReady.t(), "● READY");
    assert_eq!(
        Key::StatusInfo.t(),
        "PULSE v0.0.11  |  Windows Kernel Monitor  |  Active Processes: {}"
    );

    // Memory module - Card headers
    assert_eq!(Key::PhysicalRamTitle.t(), "◈ PHYSICAL MEMORY (RAM)");
    assert_eq!(Key::SwapCommitTitle.t(), "▣ SWAP FILE & COMMIT");

    // Memory module - Grid labels (Card 1: RAM)
    assert_eq!(Key::UsedTotal.t(), "Used / Total");
    assert_eq!(Key::AvailableFree.t(), "Available / Free");
    assert_eq!(Key::CachedStandby.t(), "Cached (Standby)");
    assert_eq!(Key::PeakSession.t(), "Peak (Session)");

    // Memory module - Grid labels (Card 2: Swap/Commit)
    assert_eq!(Key::SwapUsedTotal.t(), "Swap Used / Total");
    assert_eq!(Key::CommitChargeLimit.t(), "Commit Charge / Limit");
    assert_eq!(Key::CommitPercent.t(), "Commit %");

    // Kernel mini-tiles
    assert_eq!(Key::SystemCache.t(), "System Cache");
    assert_eq!(Key::KernelPool.t(), "Kernel Pool");
    assert_eq!(Key::CommitCharge.t(), "Committed");
    assert_eq!(Key::OsActivityTitle.t(), "OS Activity");

    // Chart
    assert_eq!(Key::MemoryHistoryTitle.t(), "● MEMORY UTILIZATION HISTORY");
    assert_eq!(Key::RamPercent.t(), "RAM %");
    assert_eq!(Key::SwapPercent.t(), "Swap %");

    // Legacy keys
    assert_eq!(Key::MemoryTotal.t(), "Total RAM");
    assert_eq!(Key::MemoryAvailable.t(), "Available");
    assert_eq!(Key::MemoryUsed.t(), "Used");
    assert_eq!(Key::SwapTotal.t(), "Total Swap");
    assert_eq!(Key::SwapUsed.t(), "Swap Used");
    assert_eq!(Key::SwapFree.t(), "Swap Free");
    assert_eq!(Key::ClearCacheButton.t(), "Clear Cache");
    assert_eq!(Key::CacheCleared.t(), "Cache cleared");
    assert_eq!(Key::CacheClearFailed.t(), "Cache clear failed");

    // Process module
    assert_eq!(Key::ProcessTitle.t(), "◈ RAM PROCESSES");
    assert_eq!(Key::ProcessSearch.t(), "Search by name or PID...");
    assert_eq!(Key::ProcessGrouped.t(), " (grouped)");
    assert_eq!(Key::ProcessName.t(), "Name");
    assert_eq!(Key::ProcessPid.t(), "PID");
    assert_eq!(Key::ProcessRam.t(), "RAM");
    assert_eq!(Key::ProcessVirtual.t(), "Virtual");
    assert_eq!(Key::ProcessPct.t(), "% RAM");
    assert_eq!(Key::ProcessCount.t(), "Count");
    assert_eq!(Key::ProcessHog.t(), "⚠ HOG");
    assert_eq!(
        Key::ProcessSpikeEvent.t(),
        "MEMORY SPIKE: {} ({}%) — {} (PID {})"
    );
}
