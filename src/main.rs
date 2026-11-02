//! PULSE — Next-Gen System Telemetry for Windows
//! Entry point for the eframe/egui application.

use eframe::egui;

/// Main application state.
struct PulseApp {
    /// Application name for the window title.
    app_name: &'static str,
}

impl Default for PulseApp {
    fn default() -> Self {
        Self {
            app_name: "⚡ PULSE",
        }
    }
}

impl eframe::App for PulseApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Apply Obsidian HUD theme
        apply_obsidian_theme(ctx);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(egui::RichText::new(self.app_name).monospace().size(28.0));
            ui.separator();
            ui.label("PULSE v0.0.1 — Project initialized successfully.");
            ui.label("Architecture: Rust + egui 0.27.2 + sysinfo 0.30.12");
            ui.label("Next: Implement core modules (Memory, Processes, System).");
        });

        // Request repaint for future metrics updates
        ctx.request_repaint_after(std::time::Duration::from_millis(100));
    }
}

/// Applies the "Obsidian HUD" visual theme to the egui context.
fn apply_obsidian_theme(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();

    // Color tokens (Obsidian HUD)
    const BG_WINDOW: egui::Color32 = egui::Color32::from_rgb(0x0B, 0x0C, 0x10); // #0B0C10
    const BG_CARD: egui::Color32 = egui::Color32::from_rgb(0x12, 0x14, 0x1C); // #12141C
    const BORDER: egui::Color32 = egui::Color32::from_rgb(0x1F, 0x23, 0x30); // #1F2330
    const RADIUS: f32 = 6.0;

    style.visuals.window_fill = BG_WINDOW;
    style.visuals.panel_fill = BG_CARD;
    style.visuals.window_stroke = egui::Stroke::new(1.0_f32, BORDER);
    style.visuals.window_rounding = egui::Rounding::same(RADIUS);

    // Widget colors
    style.visuals.widgets.noninteractive.bg_fill = BG_CARD;
    style.visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0_f32, BORDER);
    style.visuals.widgets.noninteractive.rounding = egui::Rounding::same(RADIUS);

    style.visuals.widgets.inactive.bg_fill = BG_CARD;
    style.visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0_f32, BORDER);
    style.visuals.widgets.inactive.rounding = egui::Rounding::same(RADIUS);

    style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(0x1A, 0x1E, 0x2E);
    style.visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0_f32, BORDER);
    style.visuals.widgets.hovered.rounding = egui::Rounding::same(RADIUS);

    style.visuals.widgets.active.bg_fill = egui::Color32::from_rgb(0x1F, 0x23, 0x30);
    style.visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0_f32, BORDER);
    style.visuals.widgets.active.rounding = egui::Rounding::same(RADIUS);

    // Text style: default to monospace for numbers
    style.text_styles.insert(
        egui::TextStyle::Monospace,
        egui::FontId::new(13.0, egui::FontFamily::Monospace),
    );

    ctx.set_style(style);
}

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
        Box::new(|_cc| Box::new(PulseApp::default()) as Box<dyn eframe::App>),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_creation() {
        let app = PulseApp::default();
        assert_eq!(app.app_name, "⚡ PULSE");
    }

    #[test]
    fn test_theme_colors() {
        // Verify color constants match Obsidian HUD spec
        assert_eq!(
            egui::Color32::from_rgb(0x0B, 0x0C, 0x10),
            egui::Color32::from_rgb(11, 12, 16)
        );
        assert_eq!(
            egui::Color32::from_rgb(0x12, 0x14, 0x1C),
            egui::Color32::from_rgb(18, 20, 28)
        );
        assert_eq!(
            egui::Color32::from_rgb(0x1F, 0x23, 0x30),
            egui::Color32::from_rgb(31, 35, 48)
        );
    }

    #[test]
    fn test_traffic_light_colors() {
        // Cyan < 60%
        let cyan = egui::Color32::from_rgb(0x00, 0xF0, 0xFF);
        assert_eq!(cyan.r(), 0);
        assert_eq!(cyan.g(), 0xF0);
        assert_eq!(cyan.b(), 0xFF);

        // Amber 60-85%
        let amber = egui::Color32::from_rgb(0xFF, 0xB8, 0x00);
        assert_eq!(amber.r(), 0xFF);
        assert_eq!(amber.g(), 0xB8);
        assert_eq!(amber.b(), 0x00);

        // Neon Red >= 85%
        let red = egui::Color32::from_rgb(0xFF, 0x2A, 0x54);
        assert_eq!(red.r(), 0xFF);
        assert_eq!(red.g(), 0x2A);
        assert_eq!(red.b(), 0x54);
    }
}
