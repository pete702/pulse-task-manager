//! Storage HUD chart: anchored minutes-ago dual series (read/write MB/s).

use std::sync::Arc;

use eframe::egui;
use egui_plot::{GridMark, HLine, Line, LineStyle, Plot, PlotPoints};

use crate::modules::cpu::chart_axis::x_grid_spacer;
use crate::modules::memory::downsampling::lttb_downsample;
use crate::modules::storage::collector::IO_SPIKE_THRESHOLD_MB_S;
use crate::theme::{card_frame, colors};
use crate::ui::primitives::{cyber_accent, draw_cyber_corner_brackets};

const CEILINGS: &[f64] = &[100.0, 250.0, 500.0, 1000.0, 2500.0, 5000.0, 10000.0];

/// Smallest stepped ceiling at or above the peak (at least 100 MB/s).
fn stepped_ceiling(read: &Arc<Vec<[f64; 2]>>, write: &Arc<Vec<[f64; 2]>>) -> f64 {
    let peak = read.iter().chain(write.iter()).map(|p| p[1]).fold(0.0f64, f64::max);
    let need = peak.max(100.0);
    CEILINGS.iter().copied().find(|c| *c >= need).unwrap_or(10000.0)
}

/// Renders the full storage chart card with window picker and HUD labels.
pub fn render_storage_chart(
    ui: &mut egui::Ui,
    read: &Arc<Vec<[f64; 2]>>,
    write: &Arc<Vec<[f64; 2]>>,
    window_secs: usize,
    scope: Option<&str>,
) {
    let total_now = latest_total(read, write);
    let resp = card_frame().show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("DISK THROUGHPUT")
                    .monospace()
                    .size(12.0)
                    .color(colors::TEXT_PRIMARY),
            );
            if let Some(label) = scope {
                ui.label(
                    egui::RichText::new(format!("Scope: {label}"))
                        .monospace()
                        .size(11.0)
                        .color(colors::ACCENT_CYAN),
                );
                let all = egui::Button::new(
                    egui::RichText::new("[ Show All Disks ]")
                        .monospace()
                        .size(11.0)
                        .color(colors::TEXT_SECONDARY),
                );
                if ui.add(all).clicked() {
                    crate::modules::storage::view::clear_selected();
                }
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                pick_window(ui, "1m", 60, window_secs);
                pick_window(ui, "5m", 300, window_secs);
                pick_window(ui, "15m", 900, window_secs);
            });
        });
        ui.add_space(4.0);
        let span_min = window_secs as f64 / 60.0;
        let y_ceiling = stepped_ceiling(read, write);
        let y_step = y_ceiling / 4.0;
        let y_grid = [
            GridMark { value: f64::from(0), step_size: y_step },
            GridMark { value: y_step, step_size: y_step },
            GridMark { value: y_step * 2.0, step_size: y_step },
            GridMark { value: y_step * 3.0, step_size: y_step },
            GridMark { value: y_ceiling, step_size: y_step },
        ];
        let read_pts = to_points(read);
        let write_pts = to_points(write);
        let plot_resp = Plot::new(("storage_io", window_secs))
            .width(ui.available_width().max(200.0))
            .height(220.0)
            .show_axes([false, false])
            .show_grid([true, true])
            .auto_bounds(egui::Vec2b::new(false, false))
            .y_grid_spacer(move |_| y_grid.to_vec())
            .x_grid_spacer(move |input| x_grid_spacer(span_min, &input))
            .include_x(-span_min)
            .include_x(0.0)
            .include_y(0.0)
            .include_y(y_ceiling)
            .allow_scroll(false)
            .allow_zoom(false)
            .allow_drag(false)
            .allow_boxed_zoom(false)
            .label_formatter(|name, value| {
                let ago = -value.x;
                let when =
                    if ago.abs() < 0.05 { "now".to_owned() } else { format!("{:.1}m ago", ago) };
                format!("{}: {:.1} MB/s ({})", name, value.y, when)
            })
            .show(ui, |pu| {
                pu.hline(
                    HLine::new(IO_SPIKE_THRESHOLD_MB_S as f64)
                        .color(egui::Color32::from_rgba_unmultiplied(255, 42, 84, 110))
                        .style(LineStyle::Dashed { length: 6.0 }),
                );
                pu.line(
                    Line::new(read_pts)
                        .name("Read MB/s")
                        .color(colors::ACCENT_CYAN)
                        .width(2.2_f32)
                        .fill(0.0_f32),
                );
                pu.line(
                    Line::new(write_pts)
                        .name("Write MB/s")
                        .color(colors::ACCENT_AMBER)
                        .width(2.2_f32),
                );
            });
        let rect = *plot_resp.transform.frame();
        draw_hud(ui, rect, span_min, y_ceiling);
        draw_legend(ui, rect);
    });
    draw_cyber_corner_brackets(ui, resp.response.rect, cyber_accent(total_now));
}

fn pick_window(ui: &mut egui::Ui, label: &str, secs: usize, current: usize) {
    let selected = secs == current;
    let btn =
        egui::Button::new(egui::RichText::new(label).monospace().size(11.0).color(if selected {
            colors::ACCENT_CYAN
        } else {
            colors::TEXT_SECONDARY
        }));
    if ui.add(btn).clicked() {
        crate::modules::storage::view::set_window_secs(secs);
    }
}

fn to_points(series: &Arc<Vec<[f64; 2]>>) -> PlotPoints {
    if series.is_empty() {
        PlotPoints::new(vec![[-1.0 / 60.0, 0.0], [0.0, 0.0]])
    } else {
        PlotPoints::new(lttb_downsample(series, 300))
    }
}

fn latest_total(read: &Arc<Vec<[f64; 2]>>, write: &Arc<Vec<[f64; 2]>>) -> f32 {
    let r = read.last().map(|p| p[1]).unwrap_or(0.0);
    let w = write.last().map(|p| p[1]).unwrap_or(0.0);
    (r + w).clamp(0.0, 100.0) as f32
}

fn hud_color() -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(148, 163, 184, 135)
}

fn draw_hud(ui: &egui::Ui, rect: egui::Rect, span_min: f64, y_ceiling: f64) {
    let painter = ui.painter();
    let col = hud_color();
    let top = format!("{y_ceiling:.0} MB/s");
    painter.text(
        egui::pos2(rect.right() - 6.0, rect.top() + 3.0),
        egui::Align2::RIGHT_TOP,
        top,
        egui::FontId::monospace(10.0),
        col,
    );
    for v in [75.0, 50.0, 25.0] {
        let y = rect.bottom() - (v / 100.0) * rect.height();
        let label = format!("{:.0}", v as f64 / 100.0 * y_ceiling);
        painter.text(
            egui::pos2(rect.right() - 6.0, y - 2.0),
            egui::Align2::RIGHT_CENTER,
            label,
            egui::FontId::monospace(10.0),
            col,
        );
    }
    let left = format!("{:.0}m ago", span_min);
    let y = rect.bottom() - 5.0;
    painter.text(
        egui::pos2(rect.left() + 8.0, y),
        egui::Align2::LEFT_BOTTOM,
        left,
        egui::FontId::monospace(11.0),
        col,
    );
    painter.text(
        egui::pos2(rect.right() - 8.0, y),
        egui::Align2::RIGHT_BOTTOM,
        "now",
        egui::FontId::monospace(11.0),
        col,
    );
}

fn draw_legend(ui: &egui::Ui, rect: egui::Rect) {
    let painter = ui.painter();
    painter.circle_filled(
        egui::pos2(rect.left() + 14.0, rect.top() + 10.0),
        3.5,
        colors::ACCENT_CYAN,
    );
    painter.text(
        egui::pos2(rect.left() + 24.0, rect.top() + 10.0),
        egui::Align2::LEFT_CENTER,
        "Read MB/s",
        egui::FontId::monospace(10.5),
        hud_color(),
    );
    painter.circle_filled(
        egui::pos2(rect.left() + 110.0, rect.top() + 10.0),
        3.5,
        colors::ACCENT_AMBER,
    );
    painter.text(
        egui::pos2(rect.left() + 120.0, rect.top() + 10.0),
        egui::Align2::LEFT_CENTER,
        "Write MB/s",
        egui::FontId::monospace(10.5),
        hud_color(),
    );
}
