//! Live `[04] Network` HUD: grade banner, KPIs, chart-first, adapter cards.

use std::sync::{Arc, Mutex, OnceLock};
use std::time::Instant;

use eframe::egui;

use crate::modules::network::cards::render_adapter_cards;
use crate::modules::network::chart_view::render_network_chart;
use crate::modules::network::history::NetworkHistory;
use crate::modules::network::latency_probe::{
    dns_flush_state, probe_enabled, set_probe_enabled, trigger_dns_flush, DnsFlushState,
};
use crate::modules::network::model::{ConnectionGrade, NetworkSnapshot};
use crate::theme::{card_frame, colors};

const GIB: f64 = 1_073_741_824.0;

#[derive(Clone, Copy, PartialEq)]
pub enum ChartMode {
    Traffic,
    Latency,
    Split,
}

struct NetworkViewState {
    history: NetworkHistory,
    window_secs: usize,
    mode: ChartMode,
    last_push: Option<Instant>,
    selected: Option<Arc<str>>,
    started: Instant,
}

impl Default for NetworkViewState {
    fn default() -> Self {
        Self {
            history: NetworkHistory::new(),
            window_secs: 300,
            mode: ChartMode::Traffic,
            last_push: None,
            selected: None,
            started: Instant::now(),
        }
    }
}

static STATE: OnceLock<Mutex<NetworkViewState>> = OnceLock::new();

fn state() -> &'static Mutex<NetworkViewState> {
    STATE.get_or_init(|| Mutex::new(NetworkViewState::default()))
}

/// Updates the chart window (called from chart window buttons).
pub fn set_window_secs(secs: usize) {
    let mut guard = state().lock().unwrap_or_else(|e| e.into_inner());
    guard.window_secs = secs;
}

pub fn window_secs() -> usize {
    state().lock().unwrap_or_else(|e| e.into_inner()).window_secs
}

/// Sets the chart mode (called from chart mode buttons).
#[allow(dead_code)]
pub fn set_chart_mode(latency: bool) {
    set_mode(if latency { ChartMode::Latency } else { ChartMode::Traffic });
}

/// Sets the chart mode directly (Traffic/Latency/Split).
pub fn set_mode(mode: ChartMode) {
    let mut guard = state().lock().unwrap_or_else(|e| e.into_inner());
    guard.mode = mode;
}

pub fn chart_mode_value() -> u8 {
    let g = state().lock().unwrap_or_else(|e| e.into_inner());
    match g.mode {
        ChartMode::Latency => 1,
        ChartMode::Split => 2,
        ChartMode::Traffic => 0,
    }
}

pub fn set_chart_mode_value(v: u8) {
    let mode = match v {
        1 => ChartMode::Latency,
        2 => ChartMode::Split,
        _ => ChartMode::Traffic,
    };
    set_mode(mode);
}

/// Clears the adapter scope (called from the chart header Show All button).
pub fn set_scope_all() {
    let mut guard = state().lock().unwrap_or_else(|e| e.into_inner());
    guard.selected = None;
}

/// Background ingestion: throttled push (~1s) on every snapshot.
/// Traffic always pushes so the Mbps chart works with probe OFF/probing.
/// Latency keeps None gaps (never fake 0.0); per-adapter RX/TX feed the scope.
pub fn ingest_network_snapshot(snap: &NetworkSnapshot, paused: bool) {
    let mut guard = state().lock().unwrap_or_else(|e| e.into_inner());
    if paused {
        return;
    }
    let now = Instant::now();
    let due = guard.last_push.is_none_or(|t| now.duration_since(t).as_millis() >= 1000);
    if due {
        let elapsed = guard.started.elapsed().as_secs_f64();
        guard.history.push(elapsed, snap.rx_mbps, snap.tx_mbps, snap.ping_ms, snap.jitter_ms);
        for a in snap.adapters.iter() {
            guard.history.push_adapter(a.name.as_ref(), elapsed, a.rx_mbps, a.tx_mbps);
        }
        guard.last_push = Some(now);
    }
}

/// Renders the live Network tab: chart immediately below the KPI strip.
/// Traffic honors the selected adapter scope; latency is scope-free. Split is side-by-side.
pub fn render_network_tab(ui: &mut egui::Ui, snap: &NetworkSnapshot) {
    let (first, second, lat_first, lat_second, window_secs, mode, scope, mut selected) = {
        let mut guard = state().lock().unwrap_or_else(|e| e.into_inner());
        let mode = guard.mode;
        let window = guard.window_secs;
        let sel = guard.selected.clone();
        let is_lat = mode == ChartMode::Latency;
        let scope: Option<String> = if is_lat { None } else { sel.as_ref().map(|s| s.to_string()) };
        let (a, b, la, lb) = if mode == ChartMode::Split {
            let (ta, tb) = guard.history.traffic_series_scoped(scope.as_deref(), window);
            let (la, lb) = guard.history.latency_series(window);
            (ta, tb, la, lb)
        } else if is_lat {
            let (la, lb) = guard.history.latency_series(window);
            (la.clone(), lb.clone(), la, lb)
        } else {
            let (ta, tb) = guard.history.traffic_series_scoped(scope.as_deref(), window);
            (ta.clone(), tb.clone(), ta, tb)
        };
        (a, b, la, lb, window, mode, scope, sel)
    };
    let sel_initial = selected.clone();
    let valid = snap.adapters.iter().any(|a| selected.as_ref() == Some(&a.name));
    if !valid {
        selected = snap.adapters.first().map(|a| Arc::clone(&a.name));
    }
    egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
        render_banner(ui, snap);
        ui.add_space(8.0);
        render_kpis(ui, snap);
        ui.add_space(8.0);
        if mode == ChartMode::Split {
            crate::modules::network::chart_view::render_network_split(
                ui,
                &first,
                &second,
                &lat_first,
                &lat_second,
                window_secs,
                scope.as_deref(),
            );
        } else {
            render_network_chart(
                ui,
                &first,
                &second,
                window_secs,
                mode == ChartMode::Latency,
                scope.as_deref(),
            );
        }
        // Show All may have cleared guard.selected; sync before cards.
        {
            let g = state().lock().unwrap_or_else(|e| e.into_inner());
            if g.selected != sel_initial {
                selected = g.selected.clone();
            }
        }
        ui.add_space(8.0);
        render_adapter_cards(ui, snap, &mut selected);
    });
    let mut guard = state().lock().unwrap_or_else(|e| e.into_inner());
    guard.selected = selected;
}

fn grade_color(grade: ConnectionGrade) -> egui::Color32 {
    match grade {
        ConnectionGrade::TournamentReady | ConnectionGrade::Stable => colors::ACCENT_CYAN,
        ConnectionGrade::Probing | ConnectionGrade::ProbeDisabled => colors::TEXT_SECONDARY,
        ConnectionGrade::BufferbloatJitter => colors::ACCENT_AMBER,
        ConnectionGrade::PacketLoss | ConnectionGrade::Offline => colors::ACCENT_RED,
    }
}

fn render_banner(ui: &mut egui::Ui, snap: &NetworkSnapshot) {
    let effective = if snap.probe_enabled { snap.grade } else { ConnectionGrade::ProbeDisabled };
    card_frame().show(ui, |ui| {
        ui.horizontal_wrapped(|ui| {
            let text = format!("[ GRADE: {} ]", effective.badge());
            ui.label(
                egui::RichText::new(text).monospace().size(13.0).color(grade_color(effective)),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let flush_label = match dns_flush_state() {
                    DnsFlushState::Running => "[ Flushing... ]",
                    DnsFlushState::Success => "[ DNS Flushed OK ]",
                    DnsFlushState::Failed => "[ DNS Flush Failed ]",
                    DnsFlushState::Idle => "[ Flush DNS Cache ]",
                };
                if ui.button(egui::RichText::new(flush_label).monospace().size(11.0)).clicked() {
                    trigger_dns_flush();
                }
                let probe_label = if probe_enabled() { "ON" } else { "OFF" };
                let btn = egui::Button::new(
                    egui::RichText::new(format!("[ Ping Probe: {probe_label} ]"))
                        .monospace()
                        .size(11.0),
                );
                if ui.add(btn).clicked() {
                    set_probe_enabled(!probe_enabled());
                }
            });
        });
    });
}

fn render_kpis(ui: &mut egui::Ui, snap: &NetworkSnapshot) {
    card_frame().show(ui, |ui| {
        ui.set_width(ui.available_width());
        let ping = snap.ping_ms.map(|p| format!("{p:.0}")).unwrap_or_else(|| "--".to_owned());
        let jitter = snap.jitter_ms.map(|j| format!("{j:.1}")).unwrap_or_else(|| "--".to_owned());
        let session = format!("{:.2}", snap.total_rx_gib + snap.total_tx_gib);
        let cells: [(&str, String, egui::Color32); 6] = [
            ("DOWNLOAD (Mbps)", format!("{:.1}", snap.rx_mbps), colors::ACCENT_CYAN),
            ("UPLOAD (Mbps)", format!("{:.1}", snap.tx_mbps), colors::ACCENT_AMBER),
            ("PING (ms)", ping, colors::TEXT_PRIMARY),
            ("JITTER (ms)", jitter, colors::TEXT_PRIMARY),
            ("PACKET LOSS (%)", format!("{:.1}", snap.packet_loss_pct), colors::ACCENT_RED),
            ("SESSION TRAFFIC (GiB)", session, colors::TEXT_PRIMARY),
        ];
        if ui.available_width() >= 720.0 {
            ui.columns(6, |cols| {
                for (i, (label, value, color)) in cells.iter().enumerate() {
                    kpi_cell(&mut cols[i], label, value.clone(), *color);
                }
            });
        } else {
            // Narrow: 2 rows of 3 so labels never clip at 860 px window width.
            ui.columns(3, |cols| {
                for (i, (label, value, color)) in cells.iter().take(3).enumerate() {
                    kpi_cell(&mut cols[i], label, value.clone(), *color);
                }
            });
            ui.add_space(4.0);
            ui.columns(3, |cols| {
                for (i, (label, value, color)) in cells.iter().skip(3).enumerate() {
                    kpi_cell(&mut cols[i], label, value.clone(), *color);
                }
            });
        }
    });
}

fn kpi_cell(ui: &mut egui::Ui, label: &str, value: String, color: egui::Color32) {
    ui.vertical(|ui| {
        ui.label(egui::RichText::new(label).monospace().size(9.0).color(colors::TEXT_SECONDARY));
        ui.label(egui::RichText::new(value).monospace().size(13.0).color(color));
    });
}

/// Session GiB of one adapter since collector start.
pub fn session_gib(bytes: u64) -> f64 {
    bytes as f64 / GIB
}

#[cfg(test)]
#[path = "view_tests.rs"]
mod tests;
