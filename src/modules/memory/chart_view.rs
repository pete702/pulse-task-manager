//! Multi-horizon neon chart for PULSE memory telemetry (1m / 5m / 15m).
use crate::i18n::Key;
use crate::modules::memory::chart_axis::{axis_label, minutes_ago, window_minutes};
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

/// Always-framed window selector button so the label stays visible on screen.
fn window_button(ui: &mut egui::Ui, w: ChartWindow, active: bool) -> bool {
    let color = if active { colors::ACCENT_CYAN } else { colors::TEXT_SECONDARY };
    let text = egui::RichText::new(w.label_en()).monospace().size(11.0).strong().color(color);
    let mut button = egui::Button::new(text).frame(true).min_size(egui::vec2(40.0, 20.0));
    button = if active {
        button.fill(colors::ACCENT_CYAN.gamma_multiply(0.15))
    } else {
        button.fill(colors::BORDER)
    };
    ui.add(button).clicked()
}

/// Returns cached plot points, rebuilding them whenever the history revision
/// or the selected window changes (works after the ring buffer is full).
fn cached_points(
    ui: &egui::Ui,
    key: &str,
    revision: u64,
    window: usize,
    build: impl FnOnce() -> Vec<[f64; 2]>,
) -> Vec<[f64; 2]> {
    ui.ctx().data_mut(|d| {
        let cell: &mut (u64, usize, Vec<[f64; 2]>) =
            d.get_temp_mut_or_insert_with(egui::Id::new(key), || (u64::MAX, 0, Vec::new()));
        if cell.0 != revision || cell.1 != window {
            cell.2 = build();
            cell.0 = revision;
            cell.1 = window;
        }
        cell.2.clone()
    })
}

pub fn render_memory_chart(ui: &mut egui::Ui, h: &mut MemoryHistory) {
    let mut selected = selected_window(ui.ctx());

    crate::theme::card_frame().show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(Key::MemoryHistoryTitle.t())
                        .monospace()
                        .size(13.0)
                        .color(colors::ACCENT_CYAN),
                );
                ui.add_space(16.0);
                for w in WINDOWS {
                    if window_button(ui, w, w == selected) {
                        selected = w;
                        set_window(ui.ctx(), w);
                    }
                }
            });
            ui.add_space(6.0);
            h.chart_window = selected;

            let ws = selected.seconds() as usize;
            let now = h.latest_timestamp();
            let revision = h.revision();
            let ram = cached_points(ui, "pulse_chart_ram", revision, ws, || {
                h.windowed_ram_timed().map(|(t, v)| [minutes_ago(t, now), v as f64]).collect()
            });
            let swap = cached_points(ui, "pulse_chart_swap", revision, ws, || {
                h.windowed_swap_timed().map(|(t, v)| [minutes_ago(t, now), v as f64]).collect()
            });
            let (ram, swap) = (PlotPoints::new(ram), PlotPoints::new(swap));

            let width = ui.available_width().max(80.0);
            let height = (ui.available_height() - 24.0).clamp(75.0, 165.0);
            let window_mins = window_minutes(selected);
            let window_secs = selected.seconds();
            let legend = egui_plot::Legend::default()
                .position(egui_plot::Corner::LeftTop)
                .text_style(egui::TextStyle::Monospace);
            Plot::new(("memory_pressure_plot", selected.seconds()))
                .width(width)
                .height(height)
                .show_grid(true)
                .auto_bounds(egui::Vec2b::new(false, false))
                .x_axis_formatter(move |m, _, _| axis_label(window_secs, m.value))
                .y_axis_formatter(|m, _, _| format!("{}%", m.value.round() as i32))
                .include_x(-window_mins)
                .include_x(0.0)
                .include_x(0.02)
                .include_y(0.0)
                .include_y(100.0)
                .allow_scroll(false)
                .allow_zoom(false)
                .allow_drag(false)
                .allow_boxed_zoom(false)
                .label_formatter(|n, v| {
                    let ago = -v.x;
                    let when = if ago.abs() < 0.05 {
                        "now".to_owned()
                    } else {
                        format!("{:.1}m ago", ago)
                    };
                    if n.is_empty() {
                        format!("{}: {:.1}%", when, v.y)
                    } else {
                        format!("{}\n{}: {:.1}%", n, when, v.y)
                    }
                })
                .legend(legend)
                .show(ui, |pu| {
                    pu.hline(
                        HLine::new(85.0)
                            .color(egui::Color32::from_rgba_unmultiplied(255, 42, 84, 110))
                            .style(LineStyle::Dashed { length: 6.0 }),
                    );
                    pu.line(
                        Line::new(ram)
                            .name(Key::RamPercent.t())
                            .color(colors::ACCENT_CYAN)
                            .width(2.2_f32)
                            .fill(0.0_f32),
                    );
                    pu.line(
                        Line::new(swap)
                            .name(Key::SwapPercent.t())
                            .color(colors::ACCENT_PURPLE)
                            .width(1.8_f32),
                    );
                });
        });
    });
}
