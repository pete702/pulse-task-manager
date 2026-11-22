//! Live GPU & VRAM utilization history chart for the [05] dashboard.
use std::collections::VecDeque;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Instant;

use crate::modules::memory::model::ChartWindow;
use crate::platform::dxgi_gpu::DxgiGpuAdapter;
use crate::platform::pdh_gpu::GpuTelemetrySnapshot;
use crate::theme::colors;
use crate::ui::chart_scale::{lock_time_chart_bounds, x_grid_spacer, ONE_SEC_MIN, PERCENT_Y_GRID};
use eframe::egui;
use egui_plot::{Legend, Line, Plot, PlotPoints};

const CAP: usize = 512;
const WINDOWS: [ChartWindow; 3] =
    [ChartWindow::OneMin, ChartWindow::FiveMin, ChartWindow::FifteenMin];

#[derive(Clone, Copy)]
struct GpuPoint {
    t: Instant,
    vram_pct: f32,
    gpu_pct: f32,
}

struct ChartState {
    points: VecDeque<GpuPoint>,
    last_epoch: u64,
    badge: Arc<str>,
}

static CHART: OnceLock<Mutex<ChartState>> = OnceLock::new();

fn chart_state() -> &'static Mutex<ChartState> {
    CHART.get_or_init(|| {
        Mutex::new(ChartState {
            points: VecDeque::with_capacity(CAP),
            last_epoch: u64::MAX,
            badge: Arc::from("GPU"),
        })
    })
}

/// Compact badge text for sidebar tab [05]; "GPU" until the first real sample.
pub fn gpu_badge() -> Arc<str> {
    Arc::clone(&chart_state().lock().unwrap_or_else(|e| e.into_inner()).badge)
}

/// System-wide dedicated VRAM percentage from DXGI limits and PDH usage.
pub fn vram_usage_percent(adapters: &[DxgiGpuAdapter]) -> f32 {
    let mut used = 0u64;
    let mut limit = 0u64;
    for a in adapters {
        used = used.saturating_add(a.dedicated_used_bytes);
        limit = limit.saturating_add(a.dedicated_limit_bytes);
    }
    if limit == 0 {
        return 0.0;
    }
    (used as f64 / limit as f64 * 100.0).clamp(0.0, 100.0) as f32
}

/// Maps (age in seconds, VRAM %, GPU %) rows to plot XY points for a window.
fn map_series(rows: &[(f64, f32, f32)], span_min: f64) -> (Vec<[f64; 2]>, Vec<[f64; 2]>) {
    let max_age = span_min * 60.0;
    let mut vram = Vec::new();
    let mut gpu = Vec::new();
    for &(age, v, g) in rows {
        if age > max_age {
            continue;
        }
        let x = -(age / 60.0).min(span_min);
        vram.push([x, v.clamp(0.0, 100.0) as f64]);
        gpu.push([x, g.clamp(0.0, 100.0) as f64]);
    }
    if vram.len() <= 1 {
        let v = vram.first().map_or(0.0, |p| p[1]);
        let g = gpu.first().map_or(0.0, |p| p[1]);
        return (vec![[-ONE_SEC_MIN, v], [0.0, v]], vec![[-ONE_SEC_MIN, g], [0.0, g]]);
    }
    (vram, gpu)
}

fn window_id() -> egui::Id {
    egui::Id::new("pulse_gpu_chart_window")
}

/// Bounded, non-zero plot area that always fits the 265 px upper cockpit row.
fn plot_extent(w: f32, h: f32) -> (f32, f32) {
    (w.max(160.0), h.clamp(170.0, 220.0))
}

fn selected_window(ctx: &egui::Context) -> ChartWindow {
    let mut w = ChartWindow::FiveMin;
    ctx.data_mut(|d| w = *d.get_temp_mut_or_insert_with(window_id(), || ChartWindow::FiveMin));
    w
}

fn window_button(ui: &mut egui::Ui, w: ChartWindow, active: bool) -> bool {
    let col = if active { colors::ACCENT_CYAN } else { colors::TEXT_SECONDARY };
    let txt = egui::RichText::new(w.label_en()).monospace().size(11.0).strong().color(col);
    let fill = if active { colors::ACCENT_CYAN.gamma_multiply(0.15) } else { colors::BORDER };
    ui.add(egui::Button::new(txt).frame(true).min_size(egui::vec2(38.0, 18.0)).fill(fill)).clicked()
}

fn draw_y_labels(ui: &egui::Ui, rect: egui::Rect) {
    let painter = ui.painter();
    for v in [0.0_f32, 25.0, 50.0, 75.0, 100.0] {
        let y = rect.bottom() - rect.height() * (v / 100.0);
        painter.text(
            egui::pos2(rect.left() + 5.0, y),
            egui::Align2::LEFT_CENTER,
            format!("{v:.0}%"),
            egui::FontId::monospace(9.0),
            colors::TEXT_SECONDARY,
        );
    }
}

/// Renders the live chart and records one sample per telemetry epoch.
pub fn render_gpu_chart(
    ui: &mut egui::Ui,
    adapters: &[DxgiGpuAdapter],
    snap: &GpuTelemetrySnapshot,
) {
    let vram_pct = vram_usage_percent(adapters);
    let gpu_pct =
        snap.processes.values().filter_map(|p| p.gpu_pct).fold(0.0f32, f32::max).clamp(0.0, 100.0);
    let now = Instant::now();
    {
        let mut st = chart_state().lock().unwrap_or_else(|e| e.into_inner());
        if st.last_epoch != snap.epoch || st.points.is_empty() {
            st.last_epoch = snap.epoch;
            if st.points.len() == CAP {
                st.points.pop_front();
            }
            st.points.push_back(GpuPoint { t: now, vram_pct, gpu_pct });
        }
        st.badge = if st.points.is_empty() {
            Arc::from("GPU")
        } else {
            Arc::from(format!("{}% VRAM", vram_pct.round() as u32))
        };
    }
    let rows: Vec<(f64, f32, f32)> = {
        let st = chart_state().lock().unwrap_or_else(|e| e.into_inner());
        st.points.iter().map(|p| ((now - p.t).as_secs_f64(), p.vram_pct, p.gpu_pct)).collect()
    };
    let mut window = selected_window(ui.ctx());
    let span_min = window.seconds() as f64 / 60.0;
    let (vram_pts, gpu_pts) = map_series(&rows, span_min);
    crate::ui::chart_kit::chart_frame().show(ui, |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.label(
                egui::RichText::new("GPU & VRAM UTILIZATION HISTORY")
                    .monospace()
                    .size(12.0)
                    .color(colors::ACCENT_CYAN),
            );
            ui.add_space(6.0);
            for w in WINDOWS {
                if window_button(ui, w, w == window) {
                    window = w;
                    ui.ctx().data_mut(|d| d.insert_temp(window_id(), w));
                }
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    egui::RichText::new(format!("VRAM {vram_pct:.1}% | GPU {gpu_pct:.1}%"))
                        .monospace()
                        .size(10.0)
                        .color(colors::TEXT_SECONDARY),
                );
            });
        });
        ui.add_space(4.0);
        let (plot_w, plot_h) = plot_extent(ui.available_width(), ui.available_height());
        let response = Plot::new("gpu_vram_history_plot")
            .width(plot_w)
            .height(plot_h)
            .show_axes([false, false])
            .show_grid([true, true])
            .auto_bounds(egui::Vec2b::new(false, false))
            .y_grid_spacer(|_| PERCENT_Y_GRID.to_vec())
            .x_grid_spacer(move |input| x_grid_spacer(span_min, &input))
            .include_x(-span_min)
            .include_x(0.0)
            .include_y(0.0)
            .include_y(100.0)
            .allow_scroll(false)
            .allow_zoom(false)
            .allow_drag(false)
            .allow_boxed_zoom(false)
            .legend(Legend::default().text_style(egui::TextStyle::Monospace))
            .label_formatter(|name, value| {
                if name.is_empty() {
                    format!("{:.1}%", value.y)
                } else {
                    format!("{}\n{:.1}%", name, value.y)
                }
            })
            .show(ui, |pu| {
                lock_time_chart_bounds(pu, window.seconds() as usize, 100.0);
                pu.line(
                    Line::new(PlotPoints::new(vram_pts))
                        .name("VRAM %")
                        .color(colors::ACCENT_CYAN)
                        .width(2.2_f32)
                        .fill(0.0_f32),
                );
                pu.line(
                    Line::new(PlotPoints::new(gpu_pts))
                        .name("GPU %")
                        .color(colors::ACCENT_PURPLE)
                        .width(1.8_f32)
                        .fill(0.0_f32),
                );
            });
        draw_y_labels(ui, *response.transform.frame());
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_series_filters_and_anchors() {
        let rows = [(30.0, 12.0, 40.0), (10.0, 13.0, 55.0), (0.0, 14.0, 60.0)];
        let (v, g) = map_series(&rows, 1.0);
        assert_eq!(v.len(), 3);
        assert!(v[0][0] <= v[1][0] && v[1][0] <= v[2][0]);
        assert_eq!(g[2][1], 60.0);
    }

    #[test]
    fn map_series_single_point_is_flattened() {
        let (v, g) = map_series(&[(5.0, 25.0, 75.0)], 5.0);
        assert_eq!(v.len(), 2);
        assert_eq!(v[0][1], 25.0);
        assert_eq!(g[1][1], 75.0);
    }

    #[test]
    fn plot_extent_never_collapses() {
        let (w, h) = plot_extent(0.0, 0.0);
        assert!(w >= 160.0 && h >= 170.0);
        let (w, h) = plot_extent(900.0, 2000.0);
        assert!(w >= 160.0 && h <= 220.0);
    }
}
