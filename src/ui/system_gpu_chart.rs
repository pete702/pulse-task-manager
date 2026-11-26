//! Live GPU & VRAM utilization history driven by the 1 Hz worker.
use std::collections::VecDeque;
use std::sync::{Arc, Mutex, OnceLock};

use crate::modules::memory::model::ChartWindow;
use crate::platform::dxgi_gpu::DxgiGpuAdapter;
use crate::platform::pdh_gpu::GpuTelemetrySnapshot;
use crate::theme::colors;
use crate::ui::primitives::pill_button_colored;
use crate::ui::system_gpu_tab::system_gpu_chart_hud::{draw_gpu_plot, legend_ids, segments};
use eframe::egui;

const CAP: usize = 512;
const WINDOWS: [ChartWindow; 3] =
    [ChartWindow::OneMin, ChartWindow::FiveMin, ChartWindow::FifteenMin];

#[derive(Clone, Copy)]
struct GpuPoint {
    t: f64,
    vram_pct: Option<f32>,
    gpu_pct: Option<f32>,
}

struct ChartState {
    points: VecDeque<GpuPoint>,
    last_key: Option<(u64, u64)>,
    badge: Arc<str>,
    context: Arc<str>,
}

static CHART: OnceLock<Mutex<ChartState>> = OnceLock::new();

fn chart_state() -> &'static Mutex<ChartState> {
    CHART.get_or_init(|| {
        Mutex::new(ChartState {
            points: VecDeque::with_capacity(CAP),
            last_key: None,
            badge: Arc::from("GPU"),
            context: Arc::from("VRAM: awaiting | GPU Engine: --"),
        })
    })
}

pub fn gpu_badge() -> Arc<str> {
    Arc::clone(&chart_state().lock().unwrap_or_else(|e| e.into_inner()).badge)
}

pub fn gpu_context() -> Arc<str> {
    Arc::clone(&chart_state().lock().unwrap_or_else(|e| e.into_inner()).context)
}

/// Total GPU engine utilization for the active adapter; None when PDH has no engine rows.
pub fn total_gpu_pct(snap: &GpuTelemetrySnapshot, adapters: &[DxgiGpuAdapter]) -> Option<f32> {
    if snap.engine_pct_by_luid.is_empty() {
        return None;
    }
    let active = adapters.first().map(|a| a.luid);
    active
        .and_then(|luid| snap.engine_pct_by_luid.get(&luid).copied())
        .or_else(|| snap.engine_pct_by_luid.values().copied().max_by(f32::total_cmp))
        .map(|v| v.clamp(0.0, 100.0))
}

/// Last GPU engine percentages as an Option tail: missing samples stay gaps.
pub fn recent_gpu_series(limit: usize) -> Vec<Option<f32>> {
    let st = chart_state().lock().unwrap_or_else(|e| e.into_inner());
    let mut out: Vec<Option<f32>> = st.points.iter().rev().take(limit).map(|p| p.gpu_pct).collect();
    out.reverse();
    out
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

fn sample_key(uptime_secs: u64, snapshot_id: u64) -> (u64, u64) {
    (uptime_secs, snapshot_id)
}

/// Records one point per worker tick; dedups by snapshot sequence, never by PDH epoch.
pub fn push_sample(
    uptime_secs: u64,
    snapshot_id: u64,
    gpu_pct: Option<f32>,
    vram_pct: Option<f32>,
) {
    let key = sample_key(uptime_secs, snapshot_id);
    let mut st = chart_state().lock().unwrap_or_else(|e| e.into_inner());
    if st.last_key == Some(key) {
        return;
    }
    st.last_key = Some(key);
    if st.points.len() == CAP {
        st.points.pop_front();
    }
    st.points.push_back(GpuPoint { t: uptime_secs as f64, vram_pct, gpu_pct });
    st.badge = vram_pct
        .map_or_else(|| Arc::from("GPU"), |v| Arc::from(format!("{}% VRAM", v.round() as u32)));
    st.context = match (vram_pct, gpu_pct) {
        (Some(v), Some(g)) => Arc::from(format!("VRAM: {v:.0}% | GPU Engine: {g:.0}%")),
        (Some(v), None) => Arc::from(format!("VRAM: {v:.0}% | GPU Engine: --")),
        (None, Some(g)) => Arc::from(format!("VRAM: -- | GPU Engine: {g:.0}%")),
        (None, None) => Arc::from("VRAM: awaiting | GPU Engine: --"),
    };
}

fn window_id() -> egui::Id {
    egui::Id::new("pulse_gpu_chart_window")
}

fn selected_window(ctx: &egui::Context) -> ChartWindow {
    let mut w = ChartWindow::OneMin;
    ctx.data_mut(|d| w = *d.get_temp_mut_or_insert_with(window_id(), || ChartWindow::OneMin));
    w
}

pub fn render_gpu_chart(
    ui: &mut egui::Ui,
    adapters: &[DxgiGpuAdapter],
    snap: &GpuTelemetrySnapshot,
    uptime_secs: u64,
) {
    let has_vram = adapters.iter().any(|a| a.dedicated_limit_bytes > 0);
    let vram_now = vram_usage_percent(adapters);
    let gpu_now = total_gpu_pct(snap, adapters);
    let rows: Vec<(f64, Option<f32>, Option<f32>)> = {
        let st = chart_state().lock().unwrap_or_else(|e| e.into_inner());
        st.points
            .iter()
            .map(|p| ((uptime_secs as f64 - p.t).max(0.0), p.vram_pct, p.gpu_pct))
            .collect()
    };
    let mut window = selected_window(ui.ctx());
    let span_min = window.seconds() as f64 / 60.0;
    let engine_segments = segments(&rows, span_min, true);
    let vram_segments = segments(&rows, span_min, false);
    let (engine_id, vram_id) = legend_ids();
    let show_engine: bool =
        ui.ctx().data_mut(|d| *d.get_temp_mut_or_insert_with(engine_id, || true));
    let show_vram: bool = ui.ctx().data_mut(|d| *d.get_temp_mut_or_insert_with(vram_id, || true));
    crate::ui::chart_kit::chart_frame().show(ui, |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.label(
                egui::RichText::new("GPU & VRAM UTILIZATION HISTORY")
                    .font(egui::FontId::proportional(13.0))
                    .color(colors::ACCENT_CYAN),
            );
            ui.add_space(6.0);
            for w in WINDOWS {
                let col = if w == window { colors::ACCENT_CYAN } else { colors::TEXT_SECONDARY };
                if pill_button_colored(ui, w.label_en(), col).clicked() {
                    window = w;
                    ui.ctx().data_mut(|d| d.insert_temp(window_id(), w));
                }
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let v =
                    if has_vram { format!("VRAM {vram_now:.1}%") } else { "VRAM --".to_owned() };
                let g = gpu_now
                    .map_or_else(|| "GPU Engine --".to_owned(), |g| format!("GPU Engine {g:.1}%"));
                ui.label(
                    egui::RichText::new(format!("{v} | {g}"))
                        .font(egui::FontId::monospace(10.0))
                        .color(colors::TEXT_SECONDARY),
                );
            });
        });
        ui.add_space(4.0);
        draw_gpu_plot(
            ui,
            span_min,
            window,
            show_vram,
            show_engine,
            &vram_segments,
            &engine_segments,
        );
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_window_is_one_minute() {
        assert_eq!(ChartWindow::default(), ChartWindow::OneMin);
        assert_eq!(WINDOWS[0], ChartWindow::OneMin);
    }

    #[test]
    fn sample_key_dedups_repeated_epoch_but_keeps_progressing() {
        let key = sample_key(10, 7);
        assert_eq!(key, sample_key(10, 7));
        assert_ne!(key, sample_key(10, 8));
        assert_ne!(key, sample_key(11, 7));
    }

    #[test]
    fn total_gpu_pct_is_none_without_engine_rows() {
        assert_eq!(total_gpu_pct(&GpuTelemetrySnapshot::default(), &[]), None);
    }
}
