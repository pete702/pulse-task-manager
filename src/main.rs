//! PULSE — Next-Gen System Telemetry for Windows
//! Entry point: initializes eframe and runs the app.

#![deny(unsafe_code)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::env;
use std::sync::Arc;

use eframe::egui;

mod app;
mod app_render;
mod app_tabs;
mod core;
mod i18n;
mod modules;
mod theme;
mod theme_fonts;
mod ui;
mod util;

/// Generates a 32x32 RGBA icon: dark background #0B0C10 with neon pulse symbol #00F0FF.
fn generate_icon() -> Arc<egui::IconData> {
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
    Arc::new(egui::IconData { rgba, width: SIZE, height: SIZE })
}

fn print_version() {
    println!("PULSE {} (pulse_manager)", env!("CARGO_PKG_VERSION"));
    println!("Copyright (c) 2026 pete702 (PULSE Project). Licensed under MIT.");
}

fn print_help() {
    println!("Usage: pulse_manager [--self-test | --version | --help]");
    println!();
    println!("Options:");
    println!("  --self-test    Run headless self-test (CI mode)");
    println!("  --version, -V  Print version and exit");
    println!("  --help, -h, /? Print this help and exit");
    println!();
    println!("Copyright (c) 2026 pete702 (PULSE Project). Licensed under MIT.");
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
    // Initialize panic hook
    std::panic::set_hook(Box::new(|info| {
        let msg = format!("PANIC: {}", info);
        eprintln!("{}", msg);
        let path = std::env::temp_dir().join("pulse_panic.log");
        let _ = std::fs::write(path, msg);
    }));

    // Parse CLI arguments
    let args: Vec<String> = env::args().collect();
    let args_skip = &args[1..];

    if args_skip.iter().any(|a| a == "--version" || a == "-V") {
        print_version();
        return Ok(());
    }
    if args_skip.iter().any(|a| a == "--help" || a == "-h" || a == "/?") {
        print_help();
        return Ok(());
    }

    let self_test = args_skip.iter().any(|a| a == "--self-test");

    let lang_override =
        args_skip.iter().find(|a| a.starts_with("--lang=")).and_then(|a| a.strip_prefix("--lang="));

    // Warm up elevation check once at startup (uses OnceLock internally)
    let _ = crate::modules::memory::safe_handle::is_app_elevated();

    // Initialize language from OS detection (or CLI override)
    i18n::init_language(lang_override);

    // Setup fonts before creating the app
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1180.0_f32, 760.0_f32])
            .with_min_inner_size([950.0_f32, 620.0_f32])
            .with_title("⚡ PULSE")
            .with_icon(generate_icon()),
        // 2D telemetry UI needs zero 3D depth/stencil/MSAA buffers
        depth_buffer: 0,
        stencil_buffer: 0,
        multisampling: 0,
        ..Default::default()
    };

    if self_test {
        core::selftest::run_self_test(options)
    } else {
        eframe::run_native(
            "PULSE",
            options,
            Box::new(|cc| {
                theme::setup_fonts(&cc.egui_ctx);
                theme::apply_obsidian_theme(&cc.egui_ctx);
                Box::new(app::PulseApp::new(&cc.egui_ctx)) as Box<dyn eframe::App>
            }),
        )
    }
}
