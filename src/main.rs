//! PULSE — Next-Gen System Telemetry for Windows
//! Entry point: initializes eframe and runs the app.

use eframe::egui;

mod app;
mod core;
mod theme;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("⚡ PULSE"),
        ..Default::default()
    };

    eframe::run_native(
        "PULSE",
        options,
        Box::new(|_cc| Box::new(app::PulseApp::default()) as Box<dyn eframe::App>),
    )
}
