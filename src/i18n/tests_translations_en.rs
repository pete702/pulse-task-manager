//! English translation tests for PULSE i18n keys.

use crate::i18n::{set_lang, Key, Lang};

#[test]
fn test_key_translation_en() {
    set_lang(Lang::En);
    // Top bar / window chrome
    assert_eq!(Key::AppTitle.t(), "▲ PULSE // TELEMETRY");
    assert_eq!(Key::VersionPrefix.t(), "v");
    assert_eq!(Key::UptimeLabel.t(), "Uptime");

    // Sidebar tabs
    assert_eq!(Key::TabMemory.t(), "[01] Memory (RAM)");
    assert_eq!(Key::TabCpu.t(), "[02] Processor (CPU)");
    assert_eq!(Key::TabStorage.t(), "[03] Storage (Disks)");
    assert_eq!(Key::TabNetwork.t(), "[04] Network & Ping");
    assert_eq!(Key::TabSystem.t(), "[05] System & GPU");

    // Central panel
    assert_eq!(Key::AwaitingMemoryModel.t(), "Telemetry core active. Awaiting memory model...");

    // Self-test
    assert_eq!(Key::SelfTestPassedHeadless.t(), "SELF-TEST PASSED (HEADLESS CI)");
    assert_eq!(Key::SelfTestPassedLive.t(), "SELF-TEST PASSED (LIVE GUI)");
    assert_eq!(Key::SelfTestFailed.t(), "SELF-TEST FAILED");

    // Status bar
    assert_eq!(Key::StatusReady.t(), "● READY");

    // Memory module - Card headers
    assert_eq!(Key::PhysicalRamTitle.t(), "• PHYSICAL MEMORY (RAM)");
    assert_eq!(Key::SwapCommitTitle.t(), "• SWAP FILE & COMMIT");

    // Memory module - Grid labels (Card 1: RAM)
    assert_eq!(Key::UsedTotal.t(), "Used / Total");
    assert_eq!(Key::AvailableFree.t(), "Available / Free");
    assert_eq!(Key::CachedStandby.t(), "Cached (System)");
    assert_eq!(Key::PeakSession.t(), "Peak (Session)");

    // Memory module - Grid labels (Card 2: Swap/Commit)
    assert_eq!(Key::SwapUsedTotal.t(), "Swap (Pagefile) Used");
    assert_eq!(Key::CommitChargeLimit.t(), "Commit Charge / Limit");
    assert_eq!(Key::CommitPercent.t(), "Commit %");

    // Kernel mini-tiles
    assert_eq!(Key::KernelPool.t(), "Kernel Pool");
    assert_eq!(Key::CommitCharge.t(), "Committed");
    assert_eq!(Key::OsActivityTitle.t(), "OS Activity");

    // Chart
    assert_eq!(Key::MemoryHistoryTitle.t(), "● MEMORY UTILIZATION HISTORY");
    assert_eq!(Key::RamPercent.t(), "RAM %");
    assert_eq!(Key::SwapPercent.t(), "Swap %");
}
