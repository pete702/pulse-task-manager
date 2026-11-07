//! Compact 2-column adapter cards for `[04] Network`.

use std::sync::Arc;

use eframe::egui;

use crate::modules::network::model::NetworkSnapshot;
use crate::modules::network::view::session_gib;
use crate::theme::{card_frame, colors};

/// Renders adapter cards: single adapter spans 100% width, otherwise
/// a 2-column grid on wide layouts (1 column when narrow).
pub fn render_adapter_cards(
    ui: &mut egui::Ui,
    snap: &NetworkSnapshot,
    selected: &mut Option<Arc<str>>,
) {
    if snap.adapters.is_empty() {
        card_frame().show(ui, |ui| {
            ui.label(
                egui::RichText::new("No adapters detected")
                    .monospace()
                    .size(12.0)
                    .color(colors::TEXT_SECONDARY),
            );
        });
        return;
    }
    if snap.adapters.len() == 1 {
        render_adapter_card(ui, &snap.adapters[0], selected);
        return;
    }
    if ui.available_width() >= 520.0 {
        for pair in snap.adapters.chunks(2) {
            ui.columns(2, |cols| {
                render_adapter_card(&mut cols[0], &pair[0], selected);
                if let Some(second) = pair.get(1) {
                    render_adapter_card(&mut cols[1], second, selected);
                }
            });
            ui.add_space(8.0);
        }
    } else {
        for a in snap.adapters.iter() {
            render_adapter_card(ui, a, selected);
        }
    }
}

fn render_adapter_card(
    ui: &mut egui::Ui,
    adapter: &crate::modules::network::model::NetworkAdapterInfo,
    selected: &mut Option<Arc<str>>,
) {
    let is_selected = selected.as_ref() == Some(&adapter.name);
    let stroke = if is_selected {
        egui::Stroke::new(2.0_f32, colors::ACCENT_CYAN)
    } else {
        egui::Stroke::new(1.0_f32, colors::BORDER)
    };
    let resp_inner = card_frame().stroke(stroke).show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(adapter.name.as_ref())
                    .monospace()
                    .size(12.0)
                    .color(colors::TEXT_PRIMARY),
            );
            if is_selected {
                ui.label(
                    egui::RichText::new("SELECTED")
                        .monospace()
                        .size(11.0)
                        .color(colors::ACCENT_CYAN),
                );
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let txt = format!(
                    "RX {} / TX {}",
                    format_speed(adapter.rx_mbps),
                    format_speed(adapter.tx_mbps)
                );
                ui.label(
                    egui::RichText::new(txt).monospace().size(11.0).color(colors::TEXT_SECONDARY),
                );
            });
        });
        ui.add_space(4.0);
        render_activity_bar(ui, adapter);
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            let line = format!(
                "Session RX {:.2} GiB / TX {:.2} GiB",
                session_gib(adapter.total_rx_bytes),
                session_gib(adapter.total_tx_bytes)
            );
            ui.label(
                egui::RichText::new(line).monospace().size(10.0).color(colors::TEXT_SECONDARY),
            );
        });
    });
    let id = egui::Id::new(format!("net_adapter_{}", adapter.name));
    let resp = ui
        .interact(resp_inner.response.rect, id, egui::Sense::click())
        .on_hover_cursor(egui::CursorIcon::PointingHand);
    if resp.clicked() {
        *selected = Some(Arc::clone(&adapter.name));
    }
}

/// Formats Mbps with Gbps clarity above 1000 Mbps.
pub(crate) fn format_speed(mbps: f32) -> String {
    if mbps >= 1000.0 {
        format!("{:.2} Gbps", mbps / 1000.0)
    } else {
        format!("{mbps:.1} Mbps")
    }
}

/// Adaptive link tier ceiling for the activity bar.
pub(crate) fn link_tier(total_mbps: f32) -> f32 {
    if total_mbps <= 100.0 {
        100.0
    } else if total_mbps <= 1000.0 {
        1000.0
    } else if total_mbps <= 2500.0 {
        2500.0
    } else {
        10000.0
    }
}

/// Thin live activity bar with adaptive tier ceiling.
fn render_activity_bar(
    ui: &mut egui::Ui,
    adapter: &crate::modules::network::model::NetworkAdapterInfo,
) {
    let total = adapter.rx_mbps + adapter.tx_mbps;
    let pct = (total / link_tier(total)).clamp(0.0, 1.0);
    let bar_w = ui.available_width().max(100.0);
    let (rect, _) =
        ui.allocate_exact_size(egui::vec2(bar_w, 6.0), egui::Sense::focusable_noninteractive());
    ui.painter().rect_filled(rect, 2.0, colors::BORDER);
    if pct > 0.0 {
        let fill = rect.width() * pct;
        let filled =
            egui::Rect::from_min_max(rect.min, egui::Pos2::new(rect.min.x + fill, rect.max.y));
        ui.painter().rect_filled(filled, 2.0, colors::ACCENT_CYAN);
    }
}
