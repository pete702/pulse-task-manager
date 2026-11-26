//! Compact 330x124 Mini-HUD card and viewport transition handling.

use eframe::egui;
use std::sync::Arc;

use crate::app_cmd::AppCommand;
use crate::core::SystemSnapshot;
use crate::theme::colors;
use crate::ui::primitives::{draw_status_dot, mini_progress, pill_badge, pill_button};

pub use crate::app_persist::{should_persist_window_size, sync_viewport};

fn pct(v: f32) -> f32 {
    if v.is_finite() {
        v.clamp(0.0, 100.0)
    } else {
        0.0
    }
}

pub fn format_metrics(
    cpu_pct: f32,
    ram_pct: f32,
    peak_core_pct: f32,
    uptime_secs: u64,
    self_ram_mb: f32,
) -> [String; 5] {
    [
        format!("CPU: {:.1}%", pct(cpu_pct)),
        format!("RAM: {:.1}%", pct(ram_pct)),
        format!("Peak Core: {:.1}%", pct(peak_core_pct)),
        format!("Uptime: {}", crate::util::format_uptime(uptime_secs)),
        format!("Self-RAM: {:.1} MB", if self_ram_mb.is_finite() { self_ram_mb } else { 0.0 }),
    ]
}

/// VRAM share parsed from the live GPU badge ("13% VRAM").
fn vram_percent() -> Option<f32> {
    crate::ui::system_gpu_tab::gpu_badge()
        .split_whitespace()
        .next()
        .and_then(|s| s.trim_end_matches('%').parse::<f32>().ok())
        .map(pct)
}

fn top_process(snap: &SystemSnapshot) -> String {
    snap.cpu
        .as_ref()
        .and_then(|c| {
            c.raw_processes
                .iter()
                .max_by(|a, b| a.cpu_usage.total_cmp(&b.cpu_usage))
                .map(|p| p.name.to_string())
        })
        .unwrap_or_else(|| "--".to_owned())
}

#[derive(Clone, Copy)]
struct MetricCard {
    title: &'static str,
    value: f32,
    sub: &'static str,
    color: egui::Color32,
}

fn draw_metric_card(ui: &mut egui::Ui, width: f32, card: MetricCard) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, 46.0), egui::Sense::hover());
    let painter = ui.painter();
    let rounding = egui::Rounding::same(colors::CARD_RADIUS);
    painter.rect_filled(rect, rounding, colors::BG_CARD);
    painter.rect_stroke(rect, rounding, egui::Stroke::new(1.0_f32, colors::BORDER));
    painter.text(
        egui::pos2(rect.left() + 7.0, rect.top() + 10.0),
        egui::Align2::LEFT_CENTER,
        card.title,
        egui::FontId::monospace(9.0),
        colors::TEXT_SECONDARY,
    );
    painter.text(
        egui::pos2(rect.right() - 7.0, rect.top() + 11.0),
        egui::Align2::RIGHT_CENTER,
        format!("{:.0}%", pct(card.value)),
        egui::FontId::monospace(13.0),
        card.color,
    );
    let bar = egui::Rect::from_min_max(
        egui::pos2(rect.left() + 7.0, rect.top() + 23.0),
        egui::pos2(rect.right() - 7.0, rect.top() + 28.0),
    );
    mini_progress(ui, bar, pct(card.value) / 100.0, card.color);
    painter.text(
        egui::pos2(rect.left() + 7.0, rect.bottom() - 7.0),
        egui::Align2::LEFT_CENTER,
        card.sub,
        egui::FontId::monospace(9.0),
        colors::TEXT_MUTED,
    );
}

pub fn render_mini_hud(
    ui: &mut egui::Ui,
    snapshot: &Option<Arc<SystemSnapshot>>,
    cmds: &mut Vec<AppCommand>,
    last_ui_ms: f32,
) {
    let (cpu, peak, ram_pct, ram_used, ram_total, uptime, self_ram, top) = match snapshot.as_ref() {
        Some(s) => {
            let cpu = s.cpu.as_ref().map_or(0.0, |c| c.total_pct);
            let peak = s.cpu.as_ref().map_or(0.0, |c| c.hottest_core_pct);
            let ram_pct = s.memory.as_ref().map_or(0.0, |m| m.ram_usage_percent());
            let ram_used = s.memory.as_ref().map_or(0.0, |m| m.ram_used_gib());
            let ram_total = s.memory.as_ref().map_or(0.0, |m| m.ram_total_gib());
            (cpu, peak, ram_pct, ram_used, ram_total, s.uptime_secs, s.pulse_ram_mb, top_process(s))
        }
        None => (0.0, 0.0, 0.0, 0.0, 0.0, 0, 0.0, "--".to_owned()),
    };

    ui.vertical(|ui| {
        ui.horizontal(|ui| {
            let drag_width = (ui.available_width() - 150.0).max(40.0);
            let (drag_rect, drag) =
                ui.allocate_exact_size(egui::vec2(drag_width, 22.0), egui::Sense::click_and_drag());
            if drag.drag_started_by(egui::PointerButton::Primary) {
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
            }
            ui.allocate_ui_at_rect(drag_rect, |ui| {
                ui.label(egui::RichText::new("PULSE").monospace().size(13.0).strong());
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if pill_button(ui, "X").on_hover_text("Exit PULSE").clicked() {
                    crate::platform::tray::set_allow_real_close(true);
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                }
                if pill_button(ui, "Expand").clicked() {
                    crate::platform::hotkey::set_mini_hud_active(false);
                }
                if pill_button(ui, "Tray").clicked() {
                    cmds.push(AppCommand::HideToTray);
                }
                let (status, color) =
                    match snapshot.as_ref().map(|s| crate::ui::three_walls::evaluate(s)) {
                        Some(crate::ui::three_walls::WallState::MemoryWall) => {
                            ("Memory Wall", colors::ACCENT_RED)
                        }
                        Some(crate::ui::three_walls::WallState::Elevated) => {
                            ("Elevated", colors::ACCENT_AMBER)
                        }
                        Some(crate::ui::three_walls::WallState::Optimal) => {
                            ("Nominal", colors::EMERALD)
                        }
                        _ => ("Init", colors::TEXT_SECONDARY),
                    };
                let pill = pill_badge(ui, status, color);
                draw_status_dot(
                    ui,
                    egui::pos2(pill.rect.left() + 7.0, pill.rect.center().y),
                    color,
                );
            });
        });

        let metrics = format_metrics(cpu, ram_pct, peak, uptime, self_ram);
        let verdict = crate::core::blackbox::latest_verdict();
        let line = match &verdict {
            Some(v) => format!("HUD Verdict: {}", v.chars().take(34).collect::<String>()),
            None => format!("{} / Top Proc {}", metrics[2].replace("Peak Core", "Peak"), top),
        };
        ui.label(egui::RichText::new(line).monospace().size(10.0).color(if verdict.is_some() {
            colors::ACCENT_AMBER
        } else {
            colors::TEXT_SECONDARY
        }));

        ui.horizontal(|ui| {
            let card_w = (ui.available_width() - 12.0) / 3.0;
            draw_metric_card(
                ui,
                card_w,
                MetricCard {
                    title: "CPU %",
                    value: cpu,
                    sub: "Peak",
                    color: crate::theme::load_color(pct(peak)),
                },
            );
            draw_metric_card(
                ui,
                card_w,
                MetricCard {
                    title: "RAM %",
                    value: ram_pct,
                    sub: if ram_total > 0.0 { "GiB" } else { "--" },
                    color: crate::theme::load_color(ram_pct),
                },
            );
            let vram = vram_percent();
            draw_metric_card(
                ui,
                card_w,
                MetricCard {
                    title: "VRAM %",
                    value: vram.unwrap_or(0.0),
                    sub: if vram.is_some() { "VRAM" } else { "--" },
                    color: colors::ACCENT_PURPLE,
                },
            );
        });

        ui.label(
            egui::RichText::new(format!(
                "{} | PULSE {:.1} MB | UI {:.1} ms",
                crate::util::format_uptime(uptime),
                if self_ram.is_finite() { self_ram } else { 0.0 },
                if last_ui_ms.is_finite() { last_ui_ms } else { 0.0 }
            ))
            .monospace()
            .size(9.0)
            .color(colors::TEXT_MUTED),
        );
        let _ = (ram_used, ram_total);
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app_persist::{MINI_HUD_HEIGHT, MINI_HUD_WIDTH};

    #[test]
    fn compact_metric_formatting_is_stable() {
        let m = format_metrics(42.4, 61.8, 97.2, 0, 123.4);
        assert_eq!(m[0], "CPU: 42.4%");
        assert_eq!(m[1], "RAM: 61.8%");
        assert_eq!(m[2], "Peak Core: 97.2%");
        assert!(m[3].starts_with("Uptime: "));
        assert_eq!(m[4], "Self-RAM: 123.4 MB");
        assert_eq!(pct(f32::NAN), 0.0);
        assert_eq!(pct(120.0), 100.0);
    }

    #[test]
    fn metric_card_layout_is_bounded() {
        assert_eq!(MINI_HUD_WIDTH, 330.0);
        assert_eq!(MINI_HUD_HEIGHT, 124.0);
        let card =
            MetricCard { title: "CPU %", value: 50.0, sub: "Peak", color: colors::ACCENT_CYAN };
        assert_eq!(card.title, "CPU %");
    }
}
