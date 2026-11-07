//! Unit tests for PulseApp: ActiveTab, format_uptime, i18n keys, MemoryHistory integration.

use crate::app_tabs::ActiveTab;
use crate::i18n::{set_lang, Key, Lang};
use crate::modules::memory::history::MemoryHistory;
use crate::modules::memory::model::MemorySnapshot;
use crate::util::{format_uptime, sanitize_f32, sanitize_f64};

#[test]
fn test_format_uptime_table() {
    let cases = [
        (0, "0s"),
        (30, "30s"),
        (59, "59s"),
        (60, "1m 0s"),
        (90, "1m 30s"),
        (3599, "59m 59s"),
        (3600, "1h 0m 0s"),
        (7200, "2h 0m 0s"),
        (86399, "23h 59m 59s"),
        (86400, "1d 0h 0m 0s"),
        (90061, "1d 1h 1m 1s"),
        (172800, "2d 0h 0m 0s"),
        (90000, "1d 1h 0m 0s"),
    ];
    for (input, expected) in cases {
        assert_eq!(format_uptime(input), expected, "input {input}");
    }
}

#[test]
fn test_active_tab_all() {
    let tabs = ActiveTab::all();
    assert_eq!(tabs.len(), 5);
    assert_eq!(tabs[0], ActiveTab::Memory);
    assert_eq!(tabs[1], ActiveTab::Cpu);
    assert_eq!(tabs[2], ActiveTab::Storage);
    assert_eq!(tabs[3], ActiveTab::Network);
    assert_eq!(tabs[4], ActiveTab::System);
}

#[test]
fn test_active_tab_label_en() {
    set_lang(Lang::En);
    assert_eq!(ActiveTab::Memory.label(), "[01] Memory (RAM)");
    assert_eq!(ActiveTab::Cpu.label(), "[02] Processor (CPU)");
    assert_eq!(ActiveTab::Storage.label(), "[03] Storage");
    assert_eq!(ActiveTab::Network.label(), "[04] Network");
    assert_eq!(ActiveTab::System.label(), "[05] System & GPU");
}

#[test]
fn test_i18n_keys() {
    set_lang(Lang::En);
    assert_eq!(Key::AppTitle.t(), "▲ PULSE // TELEMETRY");
    assert_eq!(Key::VersionPrefix.t(), "v");
    assert_eq!(Key::UptimeLabel.t(), "Uptime");
    assert_eq!(Key::AwaitingMemoryModel.t(), "Telemetry core active. Awaiting memory model...");
    assert_eq!(Key::SelfTestPassedHeadless.t(), "SELF-TEST PASSED (HEADLESS CI)");
    assert_eq!(Key::SelfTestPassedLive.t(), "SELF-TEST PASSED (LIVE GUI)");
    assert_eq!(Key::SelfTestFailed.t(), "SELF-TEST FAILED");
}

#[test]
fn test_memory_history_integration() {
    let mut history = MemoryHistory::new();
    let snap = MemorySnapshot {
        ram_total_bytes: 16_000_000_000,
        ram_used_bytes: 8_000_000_000,
        ram_available_bytes: 8_000_000_000,
        ram_free_bytes: 2_000_000_000,
        swap_total_bytes: 0,
        swap_used_bytes: 0,
        swap_free_bytes: 0,
        commit_total_bytes: 0,
        commit_limit_bytes: 0,
        kernel_paged_bytes: 0,
        kernel_nonpaged_bytes: 0,
        system_handles: 0,
        system_threads: 0,
        system_processes: 0,
        ..Default::default()
    };

    history.push(&snap);
    assert_eq!(history.len(), 1);
    assert!((history.peak_ram_percent - 50.0).abs() < f32::EPSILON);
}

#[test]
fn test_all_tabs_present_in_sidebar() {
    let tabs = ActiveTab::all();
    assert_eq!(tabs.len(), 5, "Should have 5 tabs: Memory, CPU, Storage, Network, System");
}

#[test]
fn test_sanitize_f32() {
    assert_eq!(sanitize_f32(1.5, 0.0), 1.5);
    assert_eq!(sanitize_f32(f32::NAN, 42.0), 42.0);
    assert_eq!(sanitize_f32(f32::INFINITY, 42.0), 42.0);
    assert_eq!(sanitize_f32(f32::NEG_INFINITY, -1.0), -1.0);
}

#[test]
fn test_sanitize_f64() {
    assert_eq!(sanitize_f64(2.5, 0.0), 2.5);
    assert_eq!(sanitize_f64(f64::NAN, 7.0), 7.0);
    assert_eq!(sanitize_f64(f64::INFINITY, 7.0), 7.0);
}
