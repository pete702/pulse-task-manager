#![allow(clippy::uninlined_format_args)]
//! PULSE -- Next-Gen System Telemetry for Windows
//! Entry point: initializes eframe and runs the app.

#![deny(unsafe_code)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::env;
use std::sync::Arc;

use eframe::egui;

mod app;
mod app_ingest;
mod app_persist;
mod app_render;
mod app_tabs;
mod app_trim;
mod core;
mod i18n;
mod modules;
mod theme;
mod theme_fonts;
mod ui;
mod util;

/// Generates and caches a 32x32 RGBA icon: dark background #0B0C10 with neon pulse symbol #00F0FF.
fn build_pulse_icon() -> Arc<egui::IconData> {
    static ICON: std::sync::OnceLock<egui::IconData> = std::sync::OnceLock::new();
    Arc::new(
        ICON.get_or_init(|| {
            const SIZE: u32 = 32;
            const BG: [u8; 4] = [0x0B, 0x0C, 0x10, 0xFF];
            const FG: [u8; 4] = [0x00, 0xF0, 0xFF, 0xFF];
            let mut pixels = vec![BG; (SIZE * SIZE) as usize];
            let center_y = SIZE as i32 / 2;
            let amplitude = SIZE as i32 / 4;
            for x in 0..SIZE as i32 {
                let phase = (x as f32 / SIZE as f32) * 2.0 * std::f32::consts::PI;
                let y = center_y - (amplitude as f32 * phase.sin()) as i32;
                for dy in -1..=1 {
                    let py = y + dy;
                    if py >= 0 && py < SIZE as i32 {
                        let idx = (py * SIZE as i32 + x) as usize;
                        if idx < pixels.len() {
                            pixels[idx] = FG;
                        }
                    }
                }
                if x == SIZE as i32 / 2 {
                    for dy in -2i32..=2i32 {
                        for dx in -2i32..=2i32 {
                            if dx.abs() + dy.abs() <= 2 {
                                let px = x + dx;
                                let py = center_y + dy;
                                if px >= 0 && px < SIZE as i32 && py >= 0 && py < SIZE as i32 {
                                    let idx = (py * SIZE as i32 + px) as usize;
                                    if idx < pixels.len() {
                                        pixels[idx] = FG;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            let rgba: Vec<u8> = pixels.into_iter().flatten().collect();
            egui::IconData { rgba, width: SIZE, height: SIZE }
        })
        .clone(),
    )
}

fn print_version() {
    println!("PULSE {} (pulse_manager)", env!("CARGO_PKG_VERSION"));
    println!("Copyright (c) 2026 Petro (pete702). All Rights Reserved. Proprietary.");
}

fn print_help() {
    println!("Usage: pulse_manager [--self-test | --version | --help]");
    println!();
    println!("Options:");
    println!("  --self-test    Run self-test (headless in CI, GUI locally)");
    println!("  --version, -V  Print version and exit");
    println!("  --help, -h, /? Print this help and exit");
    println!();
    println!("Copyright (c) 2026 Petro (pete702). All Rights Reserved. Proprietary.");
}

/// Force integrated GPU on NVIDIA Optimus laptops (0 = integrated, 1 = discrete).
#[allow(unsafe_code)]
#[no_mangle]
pub static NvOptimusEnablement: u32 = 0x00000000;

/// Force integrated GPU on AMD PowerXpress laptops (0 = integrated, 1 = discrete).
#[allow(unsafe_code)]
#[no_mangle]
pub static AmdPowerXpressRequestHighPerformance: i32 = 0;

fn main() -> eframe::Result<()> {
    std::panic::set_hook(Box::new(|info| {
        static PANIC_IN_PROGRESS: std::sync::atomic::AtomicBool =
            std::sync::atomic::AtomicBool::new(false);
        if PANIC_IN_PROGRESS.swap(true, std::sync::atomic::Ordering::SeqCst) {
            return;
        }
        crate::core::persist::flush_staged_state();
        let bt_str = if std::env::var_os("PULSE_DEBUG_BACKTRACE").is_some() {
            format!("{}", std::backtrace::Backtrace::force_capture())
        } else {
            "<set PULSE_DEBUG_BACKTRACE=1 for backtrace>".to_string()
        };
        let msg = format!("PANIC: {}\nBacktrace: {}", info, bt_str);
        let safe_msg: String = msg.chars().take(65536).collect();
        eprintln!("{}", safe_msg);
        let log_path = std::env::temp_dir().join(format!("pulse_panic_{}.log", std::process::id()));
        let _ = std::fs::OpenOptions::new().write(true).create_new(true).open(&log_path).and_then(
            |mut f| {
                use std::io::Write;
                f.write_all(safe_msg.as_bytes())
            },
        );
    }));

    let args: Vec<String> =
        std::env::args_os().skip(1).map(|a| a.to_string_lossy().into_owned()).collect();
    let args_skip = args.as_slice();
    if let Some(arg1) = args_skip.first() {
        let recognized = matches!(
            arg1.as_str(),
            "--help" | "-h" | "/?" | "--version" | "-V" | "--selftest" | "--self-test" | "--smoke"
        ) || arg1.starts_with("--lang=");
        if arg1.starts_with('-') && !recognized {
            eprintln!("Unknown option: {arg1}. Use --help for usage.");
        }
    }
    let has_cli_exit_flag = args_skip
        .iter()
        .any(|a| matches!(a.as_str(), "--version" | "-V" | "--help" | "-h" | "/?" | "--self-test"));
    if has_cli_exit_flag {
        #[allow(unsafe_code)]
        #[cfg(target_os = "windows")]
        {
            #[link(name = "kernel32")]
            extern "system" {
                fn AttachConsole(dw_process_id: u32) -> i32;
            }
            unsafe {
                AttachConsole(u32::MAX);
            }
        }
    }

    if args_skip.iter().any(|a| a == "--version" || a == "-V") {
        print_version();
        return Ok(());
    }
    if args_skip.iter().any(|a| a == "--help" || a == "-h" || a == "/?") {
        print_help();
        return Ok(());
    }

    let self_test = args_skip.iter().any(|a| a == "--self-test" || a == "--selftest");

    let lang_override =
        args_skip.iter().find(|a| a.starts_with("--lang=")).and_then(|a| a.strip_prefix("--lang="));

    // Warm up elevation check once at startup (uses OnceLock internally)
    let _ = crate::modules::memory::safe_handle::is_app_elevated();

    // Initialize language from OS detection (or CLI override)
    i18n::init_language(lang_override);

    // Restore last window size from persisted state (clamped in persist module).
    let saved = core::persist::load_state();

    // Setup fonts before creating the app
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([saved.win_w, saved.win_h])
            .with_min_inner_size([app_persist::MIN_WINDOW_WIDTH, 560.0_f32])
            .with_title("▲ PULSE")
            .with_icon(build_pulse_icon()),
        // 2D telemetry UI needs zero 3D depth/stencil/MSAA buffers.
        // Disable them to avoid ~30-50 MB driver Private Memory on fullscreen.
        depth_buffer: 0,
        stencil_buffer: 0,
        multisampling: 0,
        vsync: true,
        hardware_acceleration: eframe::HardwareAcceleration::Preferred,
        ..Default::default()
    };

    if self_test {
        core::selftest::run_self_test(options)
    } else {
        let saved_for_app = saved.clone();
        eframe::run_native(
            "PULSE",
            options,
            Box::new(move |cc| {
                theme::setup_fonts(&cc.egui_ctx);
                theme::apply_obsidian_theme(&cc.egui_ctx);
                Box::new(app::PulseApp::new_with_saved(&cc.egui_ctx, saved_for_app))
                    as Box<dyn eframe::App>
            }),
        )
    }
}
