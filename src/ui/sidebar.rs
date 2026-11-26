//! Informative 2-row telemetry sidebar cards for PULSE.
use crate::app_tabs::ActiveTab;
use crate::core::SystemSnapshot;
use crate::modules::cpu::history::CpuHistory;
use crate::modules::memory::history::MemoryHistory;
use crate::theme::colors;
use crate::ui::primitives::draw_sparkline;
use crate::ui::sidebar_badges::{draw_system_badge, draw_tab_badge};
use eframe::egui;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

static STORAGE_LATCH: Mutex<(u64, bool)> = Mutex::new((0, false));
static NETWORK_LATCH: Mutex<(u64, bool)> = Mutex::new((0, false));
const SPARK_SAMPLES: usize = 60;

pub type SidebarSeries = [Vec<Option<f32>>; 6];

fn tail_opt(points: &VecDeque<f32>) -> Vec<Option<f32>> {
    let mut out: Vec<Option<f32>> =
        points.iter().rev().take(SPARK_SAMPLES).map(|&v| Some(v)).collect();
    out.reverse();
    out
}

fn sum_tail(a: &[[f64; 2]], b: &[[f64; 2]]) -> Vec<Option<f32>> {
    let pairs = a.iter().rev().take(SPARK_SAMPLES).zip(b.iter().rev().take(SPARK_SAMPLES));
    let out: Vec<Option<f32>> = pairs.rev().map(|(x, y)| Some((x[1] + y[1]) as f32)).collect();
    out
}

/// Percentages use 0..100; rate rows anchor at 0 so a small change cannot zoom.
fn series_bounds(samples: &[Option<f32>], percent: bool) -> (f32, f32) {
    if percent {
        return (0.0, 100.0);
    }
    let peak = samples.iter().flatten().copied().filter(|v| v.is_finite()).fold(0.0_f32, f32::max);
    (0.0, peak.max(1.0))
}

/// Reads read-only tails of the real 1 Hz telemetry histories.
/// Nothing is pushed here: rendering never mutates telemetry cadence.
pub fn collect_series(
    cpu: &CpuHistory,
    mem: &MemoryHistory,
    storage: &mut crate::modules::storage::history::StorageHistory,
    network: &mut crate::modules::network::history::NetworkHistory,
    _snap: Option<&SystemSnapshot>,
) -> SidebarSeries {
    let (read, write) = storage.series_points_scoped(None, 60);
    let (rx, tx) = network.traffic_series_scoped(None, 60);
    [
        Vec::new(),
        tail_opt(&mem.ram_points),
        tail_opt(&cpu.total_points),
        sum_tail(&read, &write),
        sum_tail(&rx, &tx),
        crate::ui::system_gpu_tab::system_gpu_chart::recent_gpu_series(SPARK_SAMPLES),
    ]
}

fn next_tab(tab: ActiveTab) -> ActiveTab {
    let tabs = ActiveTab::all_ordered();
    let idx = tabs.iter().position(|t| *t == tab).unwrap_or(0);
    tabs[(idx + 1) % tabs.len()]
}

fn prev_tab(tab: ActiveTab) -> ActiveTab {
    let tabs = ActiveTab::all_ordered();
    let idx = tabs.iter().position(|t| *t == tab).unwrap_or(0);
    tabs[(idx + tabs.len() - 1) % tabs.len()]
}

fn context_line(tab: ActiveTab, snap: Option<&SystemSnapshot>) -> String {
    let Some(snap) = snap else { return "Awaiting telemetry".to_owned() };
    match tab {
        ActiveTab::Processes => {
            let top = snap.cpu.as_ref().and_then(|c| {
                c.raw_processes.iter().max_by(|a, b| a.cpu_usage.total_cmp(&b.cpu_usage))
            });
            let name: String =
                top.map_or_else(|| "--".to_owned(), |p| p.name.chars().take(18).collect());
            format!("Top: {name}")
        }
        ActiveTab::Memory => match &snap.memory {
            Some(m) => format!("{:.1} / {:.1} GiB", m.ram_used_gib(), m.ram_total_gib()),
            None => "-- GiB".to_owned(),
        },
        ActiveTab::Cpu => {
            let peak = snap.cpu.as_ref().map_or(0.0, |c| c.hottest_core_pct);
            format!("Peak: {:.0}%", if peak.is_finite() { peak } else { 0.0 })
        }
        ActiveTab::Storage => {
            format!("R/W: {:.1} MB/s", snap.storage.read_mb_s + snap.storage.write_mb_s)
        }
        ActiveTab::Network => match snap.network.ping_ms {
            Some(ms) => format!("Ping: {:.0} ms", ms),
            None => "Ping: -- ms".to_owned(),
        },
        ActiveTab::System => crate::ui::system_gpu_tab::system_gpu_chart::gpu_context().to_string(),
    }
}

pub fn render_sidebar(
    ui: &mut egui::Ui,
    active_tab: &mut ActiveTab,
    latest_snapshot: &Option<Arc<SystemSnapshot>>,
    series: &SidebarSeries,
) {
    ui.vertical(|ui| {
        ui.add_space(8.0);
        let (up, down, confirm) = ui.input(|i| {
            (
                i.key_pressed(egui::Key::ArrowUp),
                i.key_pressed(egui::Key::ArrowDown),
                i.key_pressed(egui::Key::Enter),
            )
        });
        let tabs = ActiveTab::all_ordered();
        let mut row_ids: Vec<(ActiveTab, egui::Id)> = Vec::new();
        let mut pending_focus: Option<ActiveTab> = None;
        for (idx, &tab) in tabs.iter().enumerate() {
            let is_active = *active_tab == tab;
            let text_col = if is_active { colors::TEXT_PRIMARY } else { colors::TEXT_SECONDARY };
            let (rect, response) = ui.allocate_exact_size(
                egui::vec2(ui.available_width().max(0.0), 54.0),
                egui::Sense::click(),
            );
            let response =
                response.on_hover_cursor(egui::CursorIcon::PointingHand).on_hover_text(tab.label());
            let hovered = !is_active && response.hovered();
            let fill = if is_active {
                colors::ELEVATED
            } else if hovered {
                colors::HOVER
            } else {
                colors::BG_CARD
            };
            let accent = if is_active {
                colors::ACCENT_CYAN
            } else if hovered {
                colors::ACCENT_CYAN.gamma_multiply(0.60)
            } else {
                colors::BORDER
            };
            let rounding = egui::Rounding::same(colors::CARD_RADIUS);
            let painter = ui.painter();
            painter.rect_filled(rect, rounding, fill);
            painter.rect_stroke(rect, rounding, egui::Stroke::new(1.0_f32, accent));
            if is_active {
                let bar = egui::Rect::from_min_max(
                    rect.left_top(),
                    egui::pos2(rect.left() + 3.0, rect.bottom()),
                );
                painter.rect_filled(bar, rounding, colors::ACCENT_CYAN);
            }
            let row1 = egui::Rect::from_min_max(
                rect.left_top(),
                egui::pos2(rect.right(), rect.top() + 26.0),
            );
            let row2 = egui::Rect::from_min_max(
                egui::pos2(rect.left(), rect.top() + 26.0),
                rect.right_bottom(),
            );
            painter.text(
                egui::pos2(rect.left() + 12.0, row1.center().y),
                egui::Align2::LEFT_CENTER,
                format!("{:02} {}", idx, tab.short_label()),
                egui::FontId::proportional(13.0),
                text_col,
            );
            if tab == ActiveTab::System {
                draw_system_badge(ui, row1);
            } else if let Some(ref snap) = latest_snapshot {
                draw_tab_badge(ui, row1, tab, snap, &STORAGE_LATCH, &NETWORK_LATCH);
            }
            // Row 2: clipped text ends 8 px before the spark box at right - 70.
            let text_rect = egui::Rect::from_min_max(
                egui::pos2(rect.left() + 12.0, row2.top()),
                egui::pos2(rect.right() - 78.0, row2.bottom()),
            );
            let clipped = painter.with_clip_rect(text_rect);
            clipped.text(
                egui::pos2(rect.left() + 12.0, row2.center().y),
                egui::Align2::LEFT_CENTER,
                context_line(tab, latest_snapshot.as_deref()),
                egui::FontId::proportional(11.0),
                colors::TEXT_MUTED,
            );
            let spark = egui::Rect::from_min_max(
                egui::pos2(rect.right() - 70.0, row2.center().y - 7.0),
                egui::pos2(rect.right() - 10.0, row2.center().y + 7.0),
            );
            let spark_color = if is_active { colors::ACCENT_CYAN } else { colors::TEXT_SECONDARY };
            if tab != ActiveTab::Processes {
                let samples: &[Option<f32>] = series.get(idx).map_or(&[][..], |v| v.as_slice());
                let percent = matches!(tab, ActiveTab::Memory | ActiveTab::Cpu | ActiveTab::System);
                let (min, max) = series_bounds(samples, percent);
                draw_sparkline(ui, spark, samples, spark_color, min, max);
                if tab == ActiveTab::System && !samples.iter().flatten().any(|v| v.is_finite()) {
                    ui.painter().text(
                        spark.center(),
                        egui::Align2::CENTER_CENTER,
                        "--",
                        egui::FontId::monospace(10.0),
                        spark_color,
                    );
                }
            }
            row_ids.push((tab, response.id));
            if response.clicked() {
                *active_tab = tab;
            }
            if response.has_focus() {
                if confirm {
                    *active_tab = tab;
                } else if down {
                    let nt = next_tab(tab);
                    *active_tab = nt;
                    pending_focus = Some(nt);
                } else if up {
                    let pt = prev_tab(tab);
                    *active_tab = pt;
                    pending_focus = Some(pt);
                }
            }
            if pending_focus == Some(tab) {
                response.request_focus();
            }
            ui.add_space(5.0);
        }
        if let Some(target) = pending_focus {
            if let Some((_, id)) = row_ids.iter().find(|(t, _)| *t == target) {
                ui.memory_mut(|m| m.request_focus(*id));
            }
        }
        ui.add_space(6.0);
        crate::ui::primitives::pill_badge(
            ui,
            "Pure User-Mode / Ring-3 Win32 API",
            colors::TEXT_MUTED,
        );
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tail_helpers_are_bounded_and_ordered() {
        assert_eq!(SPARK_SAMPLES, 60);
        let mut d = VecDeque::new();
        for i in 0..80 {
            d.push_back(i as f32);
        }
        let tail = tail_opt(&d);
        assert_eq!(tail.len(), SPARK_SAMPLES);
        assert_eq!(tail[0], Some(20.0));
        assert_eq!(tail[SPARK_SAMPLES - 1], Some(79.0));
        assert!(tail.windows(2).all(|w| w[0] < w[1]));
        let xy: Vec<[f64; 2]> = (0..80).map(|i| [0.0, i as f64]).collect();
        let summed = sum_tail(&xy, &xy);
        assert_eq!(summed.len(), SPARK_SAMPLES);
        assert_eq!(summed[0], Some(40.0));
    }

    #[test]
    fn spark_bounds_anchor_at_zero_and_preserve_gaps() {
        assert_eq!(series_bounds(&[None, Some(1.0), None, Some(3.0)], false), (0.0, 3.0));
        assert_eq!(series_bounds(&[None, Some(1.0)], false), (0.0, 1.0));
        assert_eq!(series_bounds(&[], false), (0.0, 1.0));
        assert_eq!(series_bounds(&[Some(1.0), Some(99.0)], true), (0.0, 100.0));
    }
}
