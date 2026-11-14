//! Network chart: traffic (Mbps), ping/jitter (ms), and split side by side.
use crate::core::downsampling::lttb_segments;
use crate::modules::network::chart_hud::*;
use crate::modules::network::view::{ChartMode, NetworkViewState};
use crate::ui::primitives::{cyber_accent, draw_cyber_corner_brackets};
use eframe::egui;
use egui_plot::PlotPoints;
use std::sync::Arc;

pub(crate) fn to_points(series: &Arc<Vec<[f64; 2]>>) -> PlotPoints {
    if series.is_empty() {
        PlotPoints::new(vec![[-1.0 / 60.0, 0.0], [0.0, 0.0]])
    } else if series.len() <= 920 {
        PlotPoints::new(series.to_vec())
    } else {
        PlotPoints::new(lttb_segments(series, 920))
    }
}

pub fn render_network_chart(
    ui: &mut egui::Ui,
    first: &Arc<Vec<[f64; 2]>>,
    second: &Arc<Vec<[f64; 2]>>,
    window_secs: usize,
    latency: bool,
    scope: Option<&str>,
    state: &mut NetworkViewState,
) {
    let mode = if latency { ChartMode::Latency } else { ChartMode::Traffic };
    let peak_now = second.last().map(|p| p[1]).unwrap_or(0.0);
    let resp = crate::ui::chart_kit::chart_frame().show(ui, |ui| {
        header_modes(ui, mode, window_secs, scope, state);
        ui.add_space(4.0);
        draw_plot(ui, first, second, window_secs, latency, "");
    });
    draw_cyber_corner_brackets(
        ui,
        resp.response.rect,
        cyber_accent(peak_now.clamp(0.0, 100.0) as f32),
    );
}

#[allow(clippy::too_many_arguments)]
pub fn render_network_split(
    ui: &mut egui::Ui,
    t_first: &Arc<Vec<[f64; 2]>>,
    t_second: &Arc<Vec<[f64; 2]>>,
    l_first: &Arc<Vec<[f64; 2]>>,
    l_second: &Arc<Vec<[f64; 2]>>,
    window_secs: usize,
    scope: Option<&str>,
    state: &mut NetworkViewState,
) {
    let resp = crate::ui::chart_kit::chart_frame().show(ui, |ui| {
        header_modes(ui, ChartMode::Split, window_secs, scope, state);
        ui.add_space(4.0);
        if ui.available_width() >= 720.0 {
            ui.columns(2, |cols| {
                draw_plot(&mut cols[0], t_first, t_second, window_secs, false, "RX / TX");
                draw_plot(&mut cols[1], l_first, l_second, window_secs, true, "Ping / Jitter");
            });
        } else {
            draw_plot(ui, t_first, t_second, window_secs, false, "RX / TX");
            ui.add_space(4.0);
            draw_plot(ui, l_first, l_second, window_secs, true, "Ping / Jitter");
        }
    });
    draw_cyber_corner_brackets(ui, resp.response.rect, cyber_accent(100.0));
}
