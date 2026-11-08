//! Self-test module for PULSE: headless validation, GUI self-test app, and CI/GUI dispatch.

use std::env;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use crate::core::selftest_app::SelfTestApp;
use crate::core::spawn_telemetry_worker;
use crate::i18n::{current_lang, set_lang, Lang};
use crate::modules::memory::actions::{
    open_process_file_location, purge_system_working_sets, terminate_process_group,
    trim_process_group, ReclaimReport, TrimSkipReason,
};
use crate::modules::memory::history::MemoryHistory;
use crate::modules::memory::process_target::ProcessTarget;
use crate::modules::memory::safe_handle::{
    is_app_elevated, is_protected_process, verify_pid_safety_gate, SafeHandle,
};

/// Runs a completely headless self-test without eframe/OpenGL.
fn fail_selftest(e: String) -> ! {
    eprintln!("[SelfTest] FAILED: {}", e);
    std::process::exit(1);
}

pub fn run_headless_self_test() -> Result<(), String> {
    println!("[SelfTest] Starting headless self-test...");
    let (worker, rx) = spawn_telemetry_worker();
    let latest = worker.latest.clone();
    let mut samples = Vec::new();
    let start = std::time::Instant::now();
    while start.elapsed() < Duration::from_secs(6) && samples.len() < 2 {
        if let Ok(snap) = rx.try_recv() {
            samples.push(snap);
        }
        thread::sleep(Duration::from_millis(50));
    }
    if samples.len() < 2 {
        return Err(format!("Expected 2 snapshots, received {}", samples.len()));
    }
    let snap = &samples[0];
    println!(
        "[SelfTest] Received snapshot: uptime={}s, pulse_ram_mb={:.1}",
        snap.uptime_secs, snap.pulse_ram_mb
    );
    if snap.uptime_secs == 0 {
        return Err("uptime_secs is 0".to_string());
    }
    if snap.pulse_ram_mb <= 0.0 {
        return Err(
            "pulse_ram_mb <= 0.0 (K32GetProcessMemoryInfo failed or returned zero)".to_string()
        );
    }
    if snap.cpu.is_none() {
        return Err("Selftest snapshot missing CPU telemetry".to_string());
    }
    // storage read_bytes_per_sec / write_bytes_per_sec and network rx_bytes_per_sec / tx_bytes_per_sec
    if !(snap.storage.read_mb_s.is_finite() && snap.storage.write_mb_s.is_finite()) {
        return Err("Selftest storage rates are not finite".to_string());
    }
    if !(snap.network.rx_mbps.is_finite() && snap.network.tx_mbps.is_finite()) {
        return Err("Selftest network rates are not finite".to_string());
    }
    if !((snap.os_scheduler_jitter_ms as f64).is_finite()
        && (snap.os_scheduler_jitter_ms as f64) >= 0.0)
    {
        return Err("Selftest jitter is not finite".to_string());
    }
    if let Some(ref mem) = snap.memory {
        println!(
            "[SelfTest] MemorySnapshot: ram_total={} bytes, ram_used={} bytes, swap_total={} bytes",
            mem.ram_total_bytes, mem.ram_used_bytes, mem.swap_total_bytes
        );
        if mem.ram_total_bytes == 0 {
            return Err("ram_total_bytes is 0".to_string());
        }
        if mem.ram_usage_percent() <= 0.0 {
            return Err(format!(
                "ram_usage_percent is {:.2} (expected > 0)",
                mem.ram_usage_percent()
            ));
        }
        println!(
            "[SelfTest] FULL scan: scan_ms={:.3}, processes={}, snapshot_id={}, process_scan_id={}, overflows={}",
            mem.scan_duration_ms,
            mem.processes.len(),
            mem.snapshot_id,
            mem.process_scan_id,
            mem.channel_overflows
        );
        if let Some(ref fast) = samples[1].memory {
            let shared = std::sync::Arc::ptr_eq(&mem.processes, &fast.processes);
            println!(
                "[SelfTest] FAST RAM tick: scan_ms={:.3}, processes={}, snapshot_id={}, process_scan_id={}, arc_shared={}",
                fast.scan_duration_ms,
                fast.processes.len(),
                fast.snapshot_id,
                fast.process_scan_id,
                shared
            );
        }
        let mut history = MemoryHistory::new();
        history.push(mem);
        if history.len() != 1 {
            return Err(format!("MemoryHistory len is {} (expected 1)", history.len()));
        }
        if history.peak_ram_percent <= 0.0 {
            return Err(format!(
                "peak_ram_percent is {:.2} (expected > 0)",
                history.peak_ram_percent
            ));
        }
        println!(
            "[SelfTest] MemoryHistory: len={}, peak_ram={:.2}%",
            history.len(),
            history.peak_ram_percent
        );
    } else {
        println!("[SelfTest] WARNING: MemorySnapshot not available (collector not integrated yet)");
    }
    println!(
        "[SelfTest] self_ram_mb={:.1}, channel_overflows={}",
        snap.pulse_ram_mb,
        latest.overflow_count()
    );
    set_lang(Lang::En);
    if current_lang() != Lang::En {
        return Err("i18n set_lang En failed".to_string());
    }
    println!("[SelfTest] i18n set_lang: En OK");
    let _ = is_app_elevated();
    if !is_protected_process(0, "system") {
        return Err("PID 0 must be blocked by the safety gate".to_string());
    }
    if !verify_pid_safety_gate(0, "system").is_err() {
        return Err("PID 0 must be rejected by verify_pid_safety_gate".to_string());
    }
    if SafeHandle::open(0, 0).is_some() {
        return Err("PID 0 must not yield a usable process handle".to_string());
    }
    let _targets: &[ProcessTarget] = &[];
    let _report: ReclaimReport = trim_process_group(_targets);
    let _ = purge_system_working_sets(_targets);
    let _ = terminate_process_group(_targets);
    assert!(
        open_process_file_location(std::path::Path::new("")).is_err(),
        "empty executable path must be rejected before spawning Explorer"
    );
    let _ = TrimSkipReason::Protected;
    println!("[SelfTest] Memory Actions Engine & Safety Gate: API surface OK");
    println!("[SelfTest] SELF-TEST PASSED (HEADLESS CI)");
    Ok(())
}

/// Runs the self-test: headless in CI (without OpenGL), with 5-frame GUI locally.
pub fn run_self_test(options: eframe::NativeOptions) -> eframe::Result<()> {
    let is_ci = env::var("CI").is_ok() || env::var("GITHUB_ACTIONS").is_ok();
    if is_ci {
        println!("[SelfTest] CI environment detected -- running headless self-test without OpenGL");
        let _ = run_headless_self_test().map_err(fail_selftest);
        return Ok(());
    }
    let passed = Arc::new(AtomicBool::new(false));
    let passed_clone = Arc::clone(&passed);
    let app_creator = Box::new(move |cc: &eframe::CreationContext<'_>| {
        crate::theme::setup_fonts(&cc.egui_ctx);
        crate::theme::apply_obsidian_theme(&cc.egui_ctx);
        let (worker, rx) = spawn_telemetry_worker();
        Box::new(SelfTestApp::new(worker, rx, Arc::clone(&passed_clone))) as Box<dyn eframe::App>
    });
    match eframe::run_native("PULSE Self-Test", options, app_creator) {
        Ok(()) => {
            if !passed.load(Ordering::SeqCst) {
                fail_selftest("GUI self-test did not pass validation".to_string());
            }
            Ok(())
        }
        Err(e) => {
            eprintln!("[SelfTest] eframe failed: {}. Falling back to headless mode.", e);
            println!("[SelfTest] Fallback to headless self-test");
            let _ = run_headless_self_test().map_err(fail_selftest);
            Ok(())
        }
    }
}

#[cfg(test)]
#[path = "selftest_tests.rs"]
mod selftest_tests;
