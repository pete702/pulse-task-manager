//! Self-test module for PULSE: headless validation, GUI self-test app, and CI/GUI dispatch.

use std::env;
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
use crate::modules::memory::safe_handle::{
    is_app_elevated, is_protected_process, verify_pid_safety_gate, SafeHandle,
};

/// Runs a completely headless self-test without eframe/OpenGL.
/// Validates: live snapshot (pulse_ram_mb > 0), MemorySnapshot fields, MemoryHistory, i18n toggle.
pub fn run_headless_self_test() -> Result<(), String> {
    println!("[SelfTest] Starting headless self-test...");

    // Spawn the telemetry worker
    let rx = spawn_telemetry_worker();

    // Wait for first snapshot (timeout 5 seconds)
    let mut snapshot = None;
    let start = std::time::Instant::now();
    while start.elapsed() < Duration::from_secs(5) {
        if let Ok(snap) = rx.try_recv() {
            snapshot = Some(snap);
            break;
        }
        thread::sleep(Duration::from_millis(50));
    }

    let snap = match snapshot {
        Some(s) => s,
        None => {
            return Err("No snapshot received within 5 seconds".to_string());
        }
    };

    println!(
        "[SelfTest] Received snapshot: uptime={}s, pulse_ram_mb={:.1}",
        snap.uptime_secs, snap.pulse_ram_mb
    );

    // Validate core telemetry
    if snap.uptime_secs == 0 {
        return Err("uptime_secs is 0".to_string());
    }
    if snap.pulse_ram_mb <= 0.0 {
        return Err(format!(
            "pulse_ram_mb is {:.1} (expected > 0)",
            snap.pulse_ram_mb
        ));
    }

    // Validate MemorySnapshot if present
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

        // Test MemoryHistory ring buffer
        let mut history = MemoryHistory::new();
        history.push(mem);
        if history.len() != 1 {
            return Err(format!(
                "MemoryHistory len is {} (expected 1)",
                history.len()
            ));
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

    // Test i18n set_lang (only En exists now)
    set_lang(Lang::En);
    if current_lang() != Lang::En {
        return Err("i18n set_lang En failed".to_string());
    }
    println!("[SelfTest] i18n set_lang: En OK");

    // Sanity check: Memory Actions Engine & Safety Gate (compile-time / dead-code guard)
    let _ = is_app_elevated();
    let _ = is_protected_process(0, "system");
    let _ = verify_pid_safety_gate(0, "system");
    let _ = SafeHandle::open(0, 0);
    let _targets: &[(u32, String)] = &[];
    let _report: ReclaimReport = trim_process_group(_targets);
    let _ = purge_system_working_sets(_targets);
    let _ = terminate_process_group(_targets);
    let _ = open_process_file_location(std::path::Path::new(""));
    let _ = TrimSkipReason::Protected;
    println!("[SelfTest] Memory Actions Engine & Safety Gate: API surface OK");

    println!("[SelfTest] SELF-TEST PASSED (HEADLESS CI)");
    Ok(())
}

/// Runs the self-test: headless in CI (without OpenGL), with 5-frame GUI locally.
/// If CI environment detected or eframe fails (e.g., no OpenGL), falls back to headless mode.
pub fn run_self_test(options: eframe::NativeOptions) -> eframe::Result<()> {
    let is_ci = env::var("CI").is_ok() || env::var("GITHUB_ACTIONS").is_ok();

    if is_ci {
        // CI mode: run headless WITHOUT eframe::run_native (no OpenGL required)
        println!("[SelfTest] CI environment detected — running headless self-test without OpenGL");
        let _ = run_headless_self_test().map_err(|e| {
            eprintln!("[SelfTest] FAILED: {}", e);
            std::process::exit(1);
        });
        return Ok(());
    }

    // Local GUI mode: try to run with eframe, fallback to headless on error (e.g., no OpenGL)
    match eframe::run_native(
        "PULSE Self-Test",
        options,
        Box::new(|cc| {
            crate::theme::setup_fonts(&cc.egui_ctx);
            crate::theme::apply_obsidian_theme(&cc.egui_ctx);
            Box::new(SelfTestApp::new(false)) as Box<dyn eframe::App>
        }),
    ) {
        Ok(()) => Ok(()),
        Err(e) => {
            // Fallback: if eframe fails (e.g., no OpenGL on CI), run headless
            eprintln!(
                "[SelfTest] eframe failed: {}. Falling back to headless mode.",
                e
            );
            println!("[SelfTest] Fallback to headless self-test");
            let _ = run_headless_self_test().map_err(|e| {
                eprintln!("[SelfTest] FAILED: {}", e);
                std::process::exit(1);
            });
            Ok(())
        }
    }
}

#[cfg(test)]
#[path = "selftest_tests.rs"]
mod selftest_tests;
