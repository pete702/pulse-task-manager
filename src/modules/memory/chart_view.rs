//! Multi-horizon neon chart for PULSE memory telemetry (1m / 5m / 15m).
use crate::i18n::Key;
use crate::modules::memory::history::MemoryHistory;
use crate::modules::memory::model::ChartWindow;
use crate::theme::colors;
use eframe::egui;
use egui_plot::{HLine, Line, LineStyle, Plot, PlotPoints};

const WINDOWS: [ChartWindow; 3] =
    [ChartWindow::OneMin, ChartWindow::FiveMin, ChartWindow::FifteenMin];
fn window_id() -> egui::Id {
    egui::Id::new("pulse_memory_chart_window")
}
pub fn selected_window(ctx: &egui::Context) -> ChartWindow {
    let mut w = ChartWindow::FiveMin;
    ctx.data_mut(|d| w = *d.get_temp_mut_or_insert_with(window_id(), || ChartWindow::FiveMin));
    w
}
pub fn set_window(ctx: &egui::Context, window: ChartWindow) {
    ctx.data_mut(|d| d.insert_temp(window_id(), window));
}
fn window_button(ui: &mut egui::Ui, w: ChartWindow, a: bool) -> bool {
    let c = if a { colors::ACCENT_CYAN } else { colors::TEXT_SECONDARY };
    let t = egui::RichText::new(w.label_en()).monospace().size(11.0).strong().color(c);
    let b = egui::Button::new(t).frame(a);
    if a {
        ui.add(b.fill(colors::ACCENT_CYAN.gamma_multiply(0.15))).clicked()
    } else {
        ui.add(b).clicked()
    }
}
fn plot_pt(i: usize, v: f32, ws: usize) -> [f64; 2] {
    [-(ws.saturating_sub(i + 1) as f64), v as f64]
}
pub fn render_memory_chart(ui: &mut egui::Ui, h: &mut MemoryHistory) {
    let mut s = selected_window(ui.ctx());
    h.chart_window = s;
    let fr = crate::theme::card_frame().inner_margin(egui::Margin::symmetric(14.0, 10.0));
    fr.outer_margin(egui::Margin { left: 0.0, right: 4.0, top: 0.0, bottom: 0.0 }).show(ui, |ui| {
        let iw = (ui.available_width() - 16.0).max(100.0);
        ui.set_max_width(iw);
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(Key::MemoryHistoryTitle.t())
                        .monospace()
                        .size(13.0)
                        .color(colors::ACCENT_CYAN),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    for w in WINDOWS {
                        if window_button(ui, w, w == s) {
                            s = w;
                            set_window(ui.ctx(), w);
                            h.chart_window = w;
                        }
                    }
                });
            });
            ui.add_space(6.0);
            let ws = s.seconds() as usize;
            let wf = ws as f64;
            let ck = (h.len(), ws);
            let (rp, sp) = ui.ctx().data_mut(|d| {
                let rk = egui::Id::new(("pulse_chart_ram", ck));
                let sk = egui::Id::new(("pulse_chart_swap", ck));
                let rp: Vec<[f64; 2]> = d
                    .get_temp_mut_or_insert_with(rk, || {
                        h.windowed_ram_points()
                            .enumerate()
                            .map(|(i, v)| plot_pt(i, v, ws))
                            .collect::<Vec<_>>()
                    })
                    .clone();
                let sp: Vec<[f64; 2]> = d
                    .get_temp_mut_or_insert_with(sk, || {
                        h.windowed_swap_points()
                            .enumerate()
                            .map(|(i, v)| plot_pt(i, v, ws))
                            .collect::<Vec<_>>()
                    })
                    .clone();
                (PlotPoints::new(rp), PlotPoints::new(sp))
            });
            let ph = (ui.available_height() - 24.0).clamp(75.0, 165.0);
            let lg = egui_plot::Legend::default()
                .position(egui_plot::Corner::LeftTop)
                .text_style(egui::TextStyle::Monospace);
            Plot::new("memory_history_plot")
                .width(iw)
                .height(ph)
                .show_grid(true)
                .x_axis_formatter(|m, _, _| format!("{}s", (-m.value).round() as i32))
                .y_axis_formatter(|m, _, _| format!("{}%", m.value.round() as i32))
                .include_x(-wf)
                .include_x(0.0)
                .include_y(0.0)
                .include_y(100.0)
                .allow_scroll(false)
                .allow_zoom(false)
                .allow_drag(false)
                .allow_boxed_zoom(false)
                .label_formatter(|n, v| {
                    let sa = v.x.abs().round() as u32;
                    if n.is_empty() {
                        format!("{sa}s ago: {:.1}%", v.y)
                    } else {
                        format!("{n}\n{sa}s ago: {:.1}%", v.y)
                    }
                })
                .legend(lg)
                .show(ui, |pu| {
                    pu.hline(
                        HLine::new(85.0)
                            .color(egui::Color32::from_rgba_unmultiplied(255, 42, 84, 110))
                            .style(LineStyle::Dashed { length: 6.0 }),
                    );
                    pu.line(
                        Line::new(rp)
                            .name(Key::RamPercent.t())
                            .color(colors::ACCENT_CYAN)
                            .width(2.2_f32)
                            .fill(0.0_f32),
                    );
                    pu.line(
                        Line::new(sp)
                            .name(Key::SwapPercent.t())
                            .color(colors::ACCENT_PURPLE)
                            .width(1.8_f32),
                    );
                });
        });
    });
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::memory::model::MemorySnapshot;
    fn snap(rt: u64, ru: u64) -> MemorySnapshot {
        MemorySnapshot { ram_total_bytes: rt, ram_used_bytes: ru, ..Default::default() }
    }
    #[test]
    fn test_default_window_is_five_minutes() {
        let ctx = egui::Context::default();
        assert_eq!(selected_window(&ctx), ChartWindow::FiveMin);
        set_window(&ctx, ChartWindow::FifteenMin);
        assert_eq!(selected_window(&ctx), ChartWindow::FifteenMin);
    }
    #[test]
    fn test_render_memory_chart_no_panic() {
        let mut h = MemoryHistory::new();
        for i in 0..120 {
            h.push(&snap(16_000_000_000, 8_000_000_000 + (i * 10_000_000) as u64));
        }
        egui::__run_test_ui(|ui| render_memory_chart(ui, &mut h));
        assert_eq!(h.len(), 120);
    }
    #[test]
    fn test_chart_window_slices() {
        let mut h = MemoryHistory::new();
        for i in 0..400 {
            h.push(&snap(100, i % 100));
        }
        h.chart_window = ChartWindow::OneMin;
        assert_eq!(h.windowed_ram_points().count(), 60);
        h.chart_window = ChartWindow::FifteenMin;
        assert_eq!(h.windowed_ram_points().count(), 400);
    }
}
