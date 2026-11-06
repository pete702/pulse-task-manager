//! CPU Peak Thread Bottleneck card.

use std::sync::{Arc, Mutex, OnceLock};

use crate::modules::cpu::model::CpuSnapshot;
use crate::theme::{card_frame, colors};
use crate::ui::primitives::{cyber_accent, draw_cyber_corner_brackets};
use eframe::egui;

fn cached_hottest(pct: f32) -> Arc<str> {
    let key = (pct * 10.0).round() as u16;
    static CACHE: OnceLock<Mutex<(u16, Arc<str>)>> = OnceLock::new();
    let mut cache = CACHE
        .get_or_init(|| Mutex::new((0, Arc::from(""))))
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    if cache.0 != key {
        cache.0 = key;
        cache.1 = Arc::from(format!("{pct:.1}%"));
    }
    Arc::clone(&cache.1)
}

fn cached_thread(idx: usize) -> Arc<str> {
    static CACHE: OnceLock<Mutex<(usize, Arc<str>)>> = OnceLock::new();
    let mut cache = CACHE
        .get_or_init(|| Mutex::new((0, Arc::from(""))))
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    if cache.0 != idx {
        cache.0 = idx;
        cache.1 = Arc::from(format!("Thread #{idx}"));
    }
    Arc::clone(&cache.1)
}

fn cached_delta(delta: f32) -> Arc<str> {
    let key = (delta * 10.0).round() as u16;
    static CACHE: OnceLock<Mutex<(u16, Arc<str>)>> = OnceLock::new();
    let mut cache = CACHE
        .get_or_init(|| Mutex::new((0, Arc::from(""))))
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    if cache.0 != key {
        cache.0 = key;
        cache.1 = Arc::from(format!("+{delta:.1}% vs Total"));
    }
    Arc::clone(&cache.1)
}

/// Renders the Peak Thread Bottleneck card.
pub fn render_bottleneck_card(ui: &mut egui::Ui, cpu: &CpuSnapshot) {
    let hottest_pct = cpu.hottest_core_pct;
    let is_bottleneck = hottest_pct >= 85.0;
    let resp = card_frame().show(ui, |ui| {
        ui.vertical(|ui| {
            // Header: Title + subtitle + Dynamic status badge
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(
                        egui::RichText::new("PEAK THREAD LOAD")
                            .monospace()
                            .size(13.0)
                            .color(colors::TEXT_PRIMARY),
                    );
                    ui.add(
                        egui::Label::new(
                            egui::RichText::new("Logical thread bottleneck detector")
                                .monospace()
                                .size(10.0)
                                .color(colors::TEXT_SECONDARY),
                        )
                        .wrap(false),
                    );
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let (badge_text, badge_color) = if is_bottleneck {
                        ("[!] SINGLE-THREAD BOUND", egui::Color32::from_rgb(239, 68, 68))
                    } else {
                        ("[OK] BALANCED", egui::Color32::from_rgb(0, 229, 255))
                    };
                    ui.label(
                        egui::RichText::new(badge_text)
                            .monospace()
                            .size(10.0)
                            .color(badge_color),
                    );
                });
            });
            ui.add_space(8.0);

            // Primary metric: Hottest thread % + precision bar
            let color = if is_bottleneck {
                egui::Color32::from_rgb(239, 68, 68)
            } else if hottest_pct >= 60.0 {
                egui::Color32::from_rgb(245, 158, 11)
            } else {
                colors::ACCENT_CYAN
            };
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(cached_hottest(hottest_pct).to_string())
                        .monospace()
                        .size(28.0)
                        .color(color),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new(cached_thread(cpu.hottest_core_idx).to_string())
                            .monospace()
                            .size(12.0)
                            .color(colors::TEXT_SECONDARY),
                    );
                });
            });
            ui.add_space(4.0);

            // Precision bar with 85% threshold marker
            let bar_w = ui.available_width().max(80.0);
            let (rect, resp) = ui.allocate_exact_size(egui::vec2(bar_w, 16.0), egui::Sense::hover());
            resp.on_hover_text(
                "Highest single logical thread utilization. >= 85% indicates a single-thread bottleneck limiting overall throughput.",
            );
            super::top_cards::draw_precision_bar_with_threshold(ui, rect, hottest_pct / 100.0, color, 0.85);
            ui.add_space(12.0);

            // Bottom row: 3 single-line metric cells via Grid (no wrapping).
            let delta = hottest_pct - cpu.total_pct;
            let diag_text = if is_bottleneck {
                "1 thread >= 85% load"
            } else if hottest_pct >= 60.0 {
                "Elevated single-thread load"
            } else {
                "Balanced"
            };

            egui::Grid::new("cpu_peak_grid")
                .num_columns(3)
                .spacing([20.0, 6.0])
                .show(ui, |ui| {
                    metric_cell(
                        ui,
                        "HOTTEST THREAD",
                        cached_thread(cpu.hottest_core_idx).as_ref(),
                        color,
                    );
                    metric_cell(
                        ui,
                        "LOAD SPREAD",
                        cached_delta(delta).as_ref(),
                        if delta > 20.0 { colors::ACCENT_RED } else { colors::TEXT_PRIMARY },
                    );
                    metric_cell(
                        ui,
                        "DIAGNOSTIC",
                        diag_text,
                        if is_bottleneck { colors::ACCENT_RED } else { colors::TEXT_PRIMARY },
                    );
                    ui.end_row();
                });
        });
    });
    draw_cyber_corner_brackets(ui, resp.response.rect, cyber_accent(hottest_pct));
}

/// Single metric cell: caption (secondary) on top, value (primary) below; both no-wrap.
fn metric_cell(ui: &mut egui::Ui, caption: &str, value: &str, value_color: egui::Color32) {
    ui.vertical(|ui| {
        ui.add(
            egui::Label::new(
                egui::RichText::new(caption).monospace().size(10.0).color(colors::TEXT_SECONDARY),
            )
            .wrap(false),
        );
        ui.add(
            egui::Label::new(egui::RichText::new(value).monospace().size(12.0).color(value_color))
                .wrap(false),
        );
    });
}
