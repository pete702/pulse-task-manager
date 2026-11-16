//! Compact 2-column adapter cards for `[04] Network`.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

use eframe::egui;

use crate::modules::network::model::NetworkSnapshot;
use crate::modules::network::view::session_gib;
use crate::theme::{card_frame, colors};

#[derive(Clone, Default)]
struct AdapterCardCache {
    max_total_mbps: f32,
    rx_str: String,
    tx_str: String,
    rx_val: f32,
    tx_val: f32,
    session_str: String,
    session_rx: u64,
    session_tx: u64,
}

fn adapter_card_cache() -> &'static Mutex<HashMap<Arc<str>, AdapterCardCache>> {
    static CACHE: OnceLock<Mutex<HashMap<Arc<str>, AdapterCardCache>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

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
    let total = adapter.rx_mbps + adapter.tx_mbps;
    let (tier, rx_txt, tx_txt, session_txt) = {
        let mut cache = adapter_card_cache().lock().unwrap_or_else(|e| e.into_inner());
        let entry = cache.entry(Arc::clone(&adapter.name)).or_default();
        if total > entry.max_total_mbps {
            entry.max_total_mbps = total;
        }
        let tier = link_tier(entry.max_total_mbps.max(total));
        if (adapter.rx_mbps - entry.rx_val).abs() > f32::EPSILON {
            entry.rx_val = adapter.rx_mbps;
            entry.rx_str = format_speed(adapter.rx_mbps);
        }
        if (adapter.tx_mbps - entry.tx_val).abs() > f32::EPSILON {
            entry.tx_val = adapter.tx_mbps;
            entry.tx_str = format_speed(adapter.tx_mbps);
        }
        if adapter.total_rx_bytes != entry.session_rx || adapter.total_tx_bytes != entry.session_tx
        {
            entry.session_rx = adapter.total_rx_bytes;
            entry.session_tx = adapter.total_tx_bytes;
            entry.session_str = format!(
                "Session RX {:.2} GiB / TX {:.2} GiB",
                session_gib(adapter.total_rx_bytes),
                session_gib(adapter.total_tx_bytes)
            );
        }
        (tier, entry.rx_str.clone(), entry.tx_str.clone(), entry.session_str.clone())
    };
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
                let txt = format!("RX {} / TX {}", rx_txt, tx_txt);
                ui.label(
                    egui::RichText::new(txt).monospace().size(11.0).color(colors::TEXT_SECONDARY),
                );
            });
        });
        ui.add_space(4.0);
        render_activity_bar(ui, adapter, tier);
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(session_txt.clone())
                    .monospace()
                    .size(10.0)
                    .color(colors::TEXT_SECONDARY),
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
    tier: f32,
) {
    let total = adapter.rx_mbps + adapter.tx_mbps;
    let pct = (total / tier).clamp(0.0, 1.0);
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
