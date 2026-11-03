//! PULSE — Next-Gen System Telemetry for Windows
//! Entry point: initializes eframe and runs the app.

#![deny(unsafe_code)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::env;
use std::sync::Arc;

use eframe::egui;

mod app;
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
    Arc::new(egui::IconData {
        rgba,
        width: SIZE,
        height: SIZE,
    })
}

fn main() -> eframe::Result<()> {
    // Initialize panic hook
    std::panic::set_hook(Box::new(|info| {
        eprintln!("PANIC: {}", info);
    }));

    // Parse CLI arguments
    let args: Vec<String> = env::args().collect();
    let self_test = args.iter().any(|a| a == "--self-test");

    let lang_override = args
        .iter()
        .find(|a| a.starts_with("--lang="))
        .and_then(|a| a.strip_prefix("--lang="));

    // Initialize language from OS detection (or CLI override)
    i18n::init_language(lang_override);

    // Setup fonts before creating the app
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1180.0_f32, 760.0_f32])
            .with_min_inner_size([950.0_f32, 620.0_f32])
            .with_title("⚡ PULSE")
            .with_icon(generate_icon()),
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
                Box::new(app::PulseApp::default()) as Box<dyn eframe::App>
            }),
        )
    }
}
