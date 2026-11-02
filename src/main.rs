//! PULSE — Next-Gen System Telemetry for Windows
//! Entry point: initializes eframe and runs the app.

#![deny(unsafe_code)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::env;
use std::sync::Arc;

use eframe::egui;

mod app;
mod core;
mod i18n;
mod modules;
mod theme;

/// Generates a 32x32 RGBA icon: dark background #0B0C10 with neon pulse symbol #00F0FF.
fn generate_icon() -> Arc<egui::IconData> {
    const SIZE: u32 = 32;
    const BG: [u8; 4] = [0x0B, 0x0C, 0x10, 0xFF]; // #0B0C10
    const FG: [u8; 4] = [0x00, 0xF0, 0xFF, 0xFF]; // #00F0FF (cyan)

    let mut pixels = vec![BG; (SIZE * SIZE) as usize];

    // Draw a simple pulse wave: horizontal line with a peak in the middle
    let center_y = SIZE as i32 / 2;
    let amplitude = SIZE as i32 / 4;

    for x in 0..SIZE as i32 {
        // Pulse wave: sin-like shape
        let phase = (x as f32 / SIZE as f32) * 2.0 * std::f32::consts::PI;
        let y = center_y - (amplitude as f32 * phase.sin()) as i32;

        // Draw a 2-pixel thick line
        for dy in -1..=1 {
            let py = y + dy;
            if py >= 0 && py < SIZE as i32 {
                let idx = (py * SIZE as i32 + x) as usize;
                if idx < pixels.len() {
                    pixels[idx] = FG;
                }
            }
        }

        // Also draw a small diamond at the peak (center)
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
    // Check for --self-test flag
    let args: Vec<String> = env::args().collect();
    let self_test = args.iter().any(|a| a == "--self-test");

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
        // Self-test mode: run headless or with minimal GUI
        run_self_test(options)
    } else {
        eframe::run_native(
            "PULSE",
            options,
            Box::new(|cc| {
                // Setup fonts
                theme::setup_fonts(&cc.egui_ctx);
                Box::new(app::PulseApp::default()) as Box<dyn eframe::App>
            }),
        )
    }
}

/// Runs the self-test: headless in CI, with 5-frame GUI locally.
fn run_self_test(options: eframe::NativeOptions) -> eframe::Result<()> {
    let is_ci = env::var("CI").is_ok();

    if is_ci {
        // Headless CI mode: use offscreen rendering
        eframe::run_native(
            "PULSE Self-Test",
            eframe::NativeOptions {
                viewport: egui::ViewportBuilder::default()
                    .with_inner_size([1.0, 1.0])
                    .with_visible(false),
                ..options
            },
            Box::new(|cc| {
                theme::setup_fonts(&cc.egui_ctx);
                Box::new(app::SelfTestApp::new(true)) as Box<dyn eframe::App>
            }),
        )
    } else {
        // Local GUI mode: show window, render 5 frames, auto-close
        eframe::run_native(
            "PULSE Self-Test",
            options,
            Box::new(|cc| {
                theme::setup_fonts(&cc.egui_ctx);
                Box::new(app::SelfTestApp::new(false)) as Box<dyn eframe::App>
            }),
        )
    }
}
