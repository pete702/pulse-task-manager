//! Live `[04] Network` HUD: grade banner, KPIs, chart-first, adapter cards.
use std::sync::Arc;

use eframe::egui;

use crate::core::SystemSnapshot;
use crate::modules::network::cards::render_adapter_cards;
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

pub struct NetworkViewState {
    pub history: crate::modules::network::history::NetworkHistory,
    pub window_secs: usize,
    pub mode: ChartMode,
    pub last_sample_ts: Option<f64>,
    pub selected: Option<Arc<str>>,
}

impl Default for NetworkViewState {
    fn default() -> Self {
        Self {
            history: crate::modules::network::history::NetworkHistory::new(),
            window_secs: 300,
            mode: ChartMode::Traffic,
            last_sample_ts: None,
            selected: None,
        }
    }
}

impl NetworkViewState {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn set_window_secs(&mut self, secs: usize) {
        self.window_secs = secs;
    }
    pub fn window_secs(&self) -> usize {
        self.window_secs
    }
    pub fn set_mode(&mut self, mode: ChartMode) {
        self.mode = mode;
    }
    pub fn chart_mode_value(&self) -> u8 {
        match self.mode {
            ChartMode::Latency => 1,
            ChartMode::Split => 2,
            ChartMode::Traffic => 0,
        }
    }
    pub fn set_chart_mode_value(&mut self, v: u8) {
        self.mode = match v {
            1 => ChartMode::Latency,
            2 => ChartMode::Split,
            _ => ChartMode::Traffic,
        };
    }
    pub fn set_scope_all(&mut self) {
        self.selected = None;
    }
}

pub fn ingest_network_snapshot(
    state: &mut NetworkViewState,
    snap: &std::sync::Arc<SystemSnapshot>,
) {
    let guard = state;
    let elapsed = snap.memory.as_ref().map(|m| m.sample_elapsed_secs).unwrap_or(0.0);
    let due = guard.last_sample_ts.is_none_or(|last| elapsed > last);
    if due {
        let net = &snap.network;
        guard.history.push(elapsed, net.rx_mbps, net.tx_mbps, net.ping_ms, net.jitter_ms);
        for a in net.adapters.iter() {
            guard.history.push_adapter(a.name.as_ref(), elapsed, a.rx_mbps, a.tx_mbps);
        }
        guard.last_sample_ts = Some(elapsed);
    }
}

pub fn render_network_tab(ui: &mut egui::Ui, snap: &NetworkSnapshot, state: &mut NetworkViewState) {
    let (first, second, lat_first, lat_second, window_secs, mode, scope, mut selected) = {
        let mode = state.mode;
        let window = state.window_secs;
        let sel = state.selected.clone();
        let is_lat = mode == ChartMode::Latency;
        let scope: Option<String> = if is_lat { None } else { sel.as_ref().map(|s| s.to_string()) };
        let (a, b, la, lb) = if mode == ChartMode::Split {
            let (ta, tb) = state.history.traffic_series_scoped(scope.as_deref(), window);
            let (la, lb) = state.history.latency_series(window);
            (ta, tb, la, lb)
        } else if is_lat {
            let (la, lb) = state.history.latency_series(window);
            (la.clone(), lb.clone(), la, lb)
        } else {
            let (ta, tb) = state.history.traffic_series_scoped(scope.as_deref(), window);
            (ta.clone(), tb.clone(), ta, tb)
        };
        (a, b, la, lb, window, mode, scope, sel)
    };
    let sel_initial = selected.clone();
    let valid = snap.adapters.iter().any(|a| selected.as_ref() == Some(&a.name));
    if selected.is_some() && !valid {
        selected = None;
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
                state,
            );
        } else {
            crate::modules::network::chart_view::render_network_chart(
                ui,
                &first,
                &second,
                window_secs,
                mode == ChartMode::Latency,
                scope.as_deref(),
                state,
            );
        }
        {
            if state.selected != sel_initial {
                selected = state.selected.clone();
            }
        }
        ui.add_space(8.0);
        render_adapter_cards(ui, snap, &mut selected);
    });
    if state.selected != selected {
        state.selected = selected;
    }
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
    card_frame().show(ui, |ui| {
        ui.horizontal_wrapped(|ui| {
            let effective =
                if snap.probe_enabled { snap.grade } else { ConnectionGrade::ProbeDisabled };
            let text = format!("GRADE: {}", effective.badge());
            ui.label(
                egui::RichText::new(text).monospace().size(13.0).color(grade_color(effective)),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let flush_label = match dns_flush_state() {
                    DnsFlushState::Running => "Flushing...",
                    DnsFlushState::Success => "DNS Flushed OK",
                    DnsFlushState::Failed => "DNS Flush Failed",
                    DnsFlushState::Idle => "Flush DNS Cache",
                };
                if ui.button(egui::RichText::new(flush_label).monospace().size(11.0)).clicked() {
                    trigger_dns_flush();
                }
                let probe_label = if probe_enabled() { "ON" } else { "OFF" };
                let btn = egui::Button::new(
                    egui::RichText::new(format!("Ping Probe: {probe_label}"))
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
        let cells: [(&str, String, egui::Color32); 4] = [
            ("DOWNLOAD (Mbps)", format!("{:.1}", snap.rx_mbps), colors::ACCENT_CYAN),
            ("UPLOAD (Mbps)", format!("{:.1}", snap.tx_mbps), colors::ACCENT_AMBER),
            ("PING (ms)", ping, colors::TEXT_PRIMARY),
            ("JITTER (ms)", jitter, colors::TEXT_PRIMARY),
        ];
        ui.columns(4, |cols| {
            for (i, (label, value, color)) in cells.iter().enumerate() {
                kpi_cell(&mut cols[i], label, value.clone(), *color);
            }
        });
    });
}

fn kpi_cell(ui: &mut egui::Ui, label: &str, value: String, color: egui::Color32) {
    ui.vertical(|ui| {
        ui.label(egui::RichText::new(label).monospace().size(9.0).color(colors::TEXT_SECONDARY));
        ui.label(egui::RichText::new(value).monospace().size(13.0).color(color));
    });
}

pub fn session_gib(bytes: u64) -> f64 {
    bytes as f64 / GIB
}
