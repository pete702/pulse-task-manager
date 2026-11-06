//! Live `[03] Storage` HUD: status strip, throughput chart, volumes, consumers.

use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

use eframe::egui;

use crate::modules::cpu::model::RawProcessCpuRow;
use crate::modules::storage::chart_view::render_storage_chart;
use crate::modules::storage::inspector::render_selection_split;
use crate::modules::storage::model::{StorageHistory, StorageSnapshot};
use crate::modules::storage::volume_cards::render_volume_cards;
use crate::theme::{card_frame, colors};

struct StorageViewState {
    history: StorageHistory,
    window_secs: usize,
    last_push: Option<Instant>,
    selected_mount: Option<Arc<str>>,
    started: Instant,
}

impl Default for StorageViewState {
    fn default() -> Self {
        Self {
            history: StorageHistory::new(),
            window_secs: 300,
            last_push: None,
            selected_mount: None,
            started: Instant::now(),
        }
    }
}

static STATE: OnceLock<Mutex<StorageViewState>> = OnceLock::new();

fn state() -> &'static Mutex<StorageViewState> {
    STATE.get_or_init(|| Mutex::new(StorageViewState::default()))
}

/// Updates the chart window (called from chart window buttons).
pub fn set_window_secs(secs: usize) {
    let mut guard = state().lock().unwrap_or_else(|e| e.into_inner());
    guard.window_secs = secs;
}

pub fn window_secs() -> usize {
    state().lock().unwrap_or_else(|e| e.into_inner()).window_secs
}

/// Background ingestion: pure 1-second time throttle (like network).
/// Pushes even at 0.0 MB/s so the chart scrolls smoothly when idle.
/// Lock is held only briefly.
pub fn ingest_storage_snapshot(
    snap: &StorageSnapshot,
    raw_processes: &[RawProcessCpuRow],
    paused: bool,
) {
    let mut guard = state().lock().unwrap_or_else(|e| e.into_inner());
    if paused || raw_processes.is_empty() {
        return;
    }
    let now = Instant::now();
    let due = guard.last_push.is_none_or(|t| now.duration_since(t) >= Duration::from_millis(950));
    if !due {
        return;
    }
    let elapsed = guard.started.elapsed().as_secs_f64();
    guard.history.push(elapsed, snap.read_mb_s, snap.write_mb_s);
    guard.last_push = Some(now);
}

/// Renders the live Storage tab: chart first, then volume grid.
pub fn render_storage_tab(
    ui: &mut egui::Ui,
    snap: &StorageSnapshot,
    raw_processes: &[RawProcessCpuRow],
) {
    let (read_series, write_series, window_secs, mut selected) = {
        let mut guard = state().lock().unwrap_or_else(|e| e.into_inner());
        let window = guard.window_secs;
        let (r, w) = guard.history.series_points(window);
        (r, w, window, guard.selected_mount.clone())
    };
    let valid = snap.volumes.iter().any(|v| selected.as_ref() == Some(&v.mount_point));
    if !valid {
        selected = snap.volumes.first().map(|v| Arc::clone(&v.mount_point));
    }
    egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
        render_status_and_kpis(ui, snap);
        ui.add_space(8.0);
        render_storage_chart(ui, &read_series, &write_series, window_secs);
        ui.add_space(8.0);
        render_volume_cards(ui, &snap.volumes, &mut selected);
        ui.add_space(8.0);
        render_selection_split(ui, snap, raw_processes, &selected);
    });
    let mut guard = state().lock().unwrap_or_else(|e| e.into_inner());
    guard.selected_mount = selected;
}

/// Fixed 5-column strip spanning 100% width (zero-jitter, no wrapping).
/// Col 0: Blackbox state, Col 1-3: rates/count, Col 4: truncated TOP I/O.
fn render_status_and_kpis(ui: &mut egui::Ui, snap: &StorageSnapshot) {
    card_frame().show(ui, |ui| {
        ui.set_width(ui.available_width());
        if ui.available_width() >= 720.0 {
            ui.columns(5, |cols| {
                render_blackbox(&mut cols[0], snap);
                col_kpi(
                    &mut cols[1],
                    "READ MB/s",
                    format!("{:.1}", snap.read_mb_s),
                    colors::ACCENT_CYAN,
                );
                col_kpi(
                    &mut cols[2],
                    "WRITE MB/s",
                    format!("{:.1}", snap.write_mb_s),
                    colors::ACCENT_AMBER,
                );
                col_kpi(
                    &mut cols[3],
                    "VOLUMES",
                    format!("{}", snap.volumes.len()),
                    colors::TEXT_PRIMARY,
                );
                let top = snap
                    .top_io_process
                    .as_ref()
                    .map(|t| truncate(&format!("{} {:.1}", t.1, t.2), 22))
                    .unwrap_or_else(|| "-".to_owned());
                col_kpi(&mut cols[4], "TOP I/O", top, colors::TEXT_PRIMARY);
            });
        } else {
            ui.columns(3, |cols| {
                render_blackbox(&mut cols[0], snap);
                col_kpi(
                    &mut cols[1],
                    "READ MB/s",
                    format!("{:.1}", snap.read_mb_s),
                    colors::ACCENT_CYAN,
                );
                col_kpi(
                    &mut cols[2],
                    "WRITE MB/s",
                    format!("{:.1}", snap.write_mb_s),
                    colors::ACCENT_AMBER,
                );
            });
            ui.add_space(4.0);
            ui.columns(3, |cols| {
                col_kpi(
                    &mut cols[0],
                    "VOLUMES",
                    format!("{}", snap.volumes.len()),
                    colors::TEXT_PRIMARY,
                );
                let top = snap
                    .top_io_process
                    .as_ref()
                    .map(|t| truncate(&format!("{} {:.1}", t.1, t.2), 22))
                    .unwrap_or_else(|| "-".to_owned());
                col_kpi(&mut cols[1], "TOP I/O", top, colors::TEXT_PRIMARY);
            });
        }
    });
}

fn render_blackbox(ui: &mut egui::Ui, snap: &StorageSnapshot) {
    ui.vertical(|ui| {
        ui.label(
            egui::RichText::new("I/O BLACKBOX").monospace().size(9.0).color(colors::TEXT_SECONDARY),
        );
        if let Some(spike) = snap.last_io_spike.as_ref() {
            let sub = format!(
                "{} {:.1} MB/s ({}s ago)",
                spike.process_name, spike.peak_mb_s, spike.ticks_ago
            );
            let tip = format!(
                "PID {} at {} - R {:.1} / W {:.1} MB/s - peak {:.1} MB/s",
                spike.pid, spike.timestamp_hms, spike.read_mb_s, spike.write_mb_s, spike.peak_mb_s
            );
            let resp = ui.label(
                egui::RichText::new("SPIKE ALERT").monospace().size(13.0).color(colors::ACCENT_RED),
            );
            resp.on_hover_text(tip);
            ui.label(
                egui::RichText::new(truncate(&sub, 28))
                    .monospace()
                    .size(9.0)
                    .color(colors::TEXT_SECONDARY),
            );
        } else {
            ui.label(
                egui::RichText::new("ARMED")
                    .monospace()
                    .size(13.0)
                    .color(crate::theme::OPTIMAL_EMERALD),
            );
        }
    });
}

fn col_kpi(ui: &mut egui::Ui, label: &str, value: String, color: egui::Color32) {
    ui.vertical(|ui| {
        ui.label(egui::RichText::new(label).monospace().size(9.0).color(colors::TEXT_SECONDARY));
        ui.label(egui::RichText::new(value).monospace().size(13.0).color(color));
    });
}

fn truncate(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_owned();
    }
    let mut out: String = text.chars().take(max_chars.saturating_sub(1)).collect();
    out.push('~');
    out
}
