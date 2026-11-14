//! CPU chart window state helpers.

use crate::modules::cpu::history::CpuChartWindow;
use eframe::egui;

fn window_id() -> egui::Id {
    egui::Id::new("pulse_cpu_chart_window")
}

pub fn selected_window(ctx: &egui::Context) -> CpuChartWindow {
    let mut window = CpuChartWindow::FiveMin;
    ctx.data_mut(|d| {
        window = *d.get_temp_mut_or_insert_with(window_id(), || CpuChartWindow::FiveMin)
    });
    window
}

pub fn set_window(ctx: &egui::Context, window: CpuChartWindow) {
    ctx.data_mut(|d| d.insert_temp(window_id(), window));
}
