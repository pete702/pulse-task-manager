//! Live `[03] Storage` HUD: status strip, throughput chart, volumes, consumers.

use std::sync::{Arc, Mutex, OnceLock};

use eframe::egui;

use crate::core::SystemSnapshot;
use crate::modules::cpu::model::RawProcessCpuRow;
use crate::modules::storage::chart_view::render_storage_chart;
use crate::modules::storage::inspector::render_selection_split;
use crate::modules::storage::model::{StorageHistory, StorageSnapshot};
use crate::modules::storage::view_helpers::{col_kpi, render_blackbox, truncate};
use crate::modules::storage::volume_cards::render_volume_cards;
use crate::theme::{card_frame, colors};

struct StorageViewState {
    history: StorageHistory,
    window_secs: usize,
    last_sample_ts: Option<f64>,
    selected_mount: Option<Arc<str>>,
}

impl Default for StorageViewState {
    fn default() -> Self {
        Self {
            history: StorageHistory::new(),
            window_secs: 300,
            last_sample_ts: None,
            selected_mount: None,
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

/// Clears selection back to All Disks (called from chart scope button).
pub fn clear_selected() {
    let mut guard = state().lock().unwrap_or_else(|e| e.into_inner());
    guard.selected_mount = None;
}

/// Current selected mount (None = All Disks).
#[allow(dead_code)]
pub fn selected_mount() -> Option<Arc<str>> {
    state().lock().unwrap_or_else(|e| e.into_inner()).selected_mount.clone()
}

#[cfg(test)]
pub(crate) fn set_selected_for_test(value: Option<Arc<str>>) {
    let mut guard = state().lock().unwrap_or_else(|e| e.into_inner());
    guard.selected_mount = value;
}

/// Background ingestion on worker pause-adjusted timestamps.
/// Pushes even at 0.0 MB/s so the chart scrolls smoothly when idle.
pub fn ingest_storage_snapshot(
    snap: &SystemSnapshot,
    _raw_processes: &[RawProcessCpuRow],
    paused: bool,
) {
    let mut guard = state().lock().unwrap_or_else(|e| e.into_inner());
    if paused {
        return;
    }
    let elapsed = snap.memory.as_ref().map(|m| m.sample_elapsed_secs).unwrap_or(0.0);
    let due = guard.last_sample_ts.is_none_or(|last| elapsed > last);
    if !due {
        return;
    }
    guard.history.push(elapsed, snap.storage.read_mb_s, snap.storage.write_mb_s);
    // Per-disk series: one sample per physical disk (volumes share rates).
    let mut seen: Vec<u32> = Vec::new();
    for vol in snap.storage.volumes.iter() {
        if let Some(d) = vol.physical_disk {
            if seen.contains(&d) {
                continue;
            }
            seen.push(d);
            guard.history.push_disk(d, elapsed, vol.read_mb_s, vol.write_mb_s);
        }
    }
    guard.last_sample_ts = Some(elapsed);
}

/// Renders the live Storage tab: chart first, then volume grid.
pub fn render_storage_tab(
    ui: &mut egui::Ui,
    snap: &StorageSnapshot,
    raw_processes: &[RawProcessCpuRow],
) {
    // Resolve selected volume -> Disk N scope (None = All Disks).
    let mut selected = {
        let guard = state().lock().unwrap_or_else(|e| e.into_inner());
        guard.selected_mount.clone()
    };
    let sel_initial = selected.clone();
    // Clear stale selection if the mount vanished; keep None as "All".
    if selected.is_some() {
        let valid = snap.volumes.iter().any(|v| selected.as_ref() == Some(&v.mount_point));
        if !valid {
            selected = None;
        }
    }
    let scope_disk: Option<u32> = selected.as_ref().and_then(|sel| {
        snap.volumes.iter().find(|v| &v.mount_point == sel).and_then(|v| v.physical_disk)
    });
    let (kpi_read, kpi_write) = match scope_disk {
        Some(d) => snap
            .volumes
            .iter()
            .find(|v| v.physical_disk == Some(d))
            .map(|v| (v.read_mb_s, v.write_mb_s))
            .unwrap_or((snap.read_mb_s, snap.write_mb_s)),
        None => (snap.read_mb_s, snap.write_mb_s),
    };
    let scope_label = scope_disk.map(|d| {
        let mounts: Vec<String> = snap
            .volumes
            .iter()
            .filter(|v| v.physical_disk == Some(d))
            .map(|v| v.mount_point.to_string())
            .collect();
        format!("Disk {d} ({})", mounts.join(", "))
    });
    let (read_series, write_series, window_secs) = {
        let mut guard = state().lock().unwrap_or_else(|e| e.into_inner());
        let window = guard.window_secs;
        let (r, w) = guard.history.series_points_scoped(scope_disk, window);
        (r, w, window)
    };
    egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
        render_status_and_kpis(ui, snap, kpi_read, kpi_write, scope_disk);
        ui.add_space(8.0);
        render_storage_chart(ui, &read_series, &write_series, window_secs, scope_label.as_deref());
        // Show All may have cleared guard.selected_mount; sync before cards.
        {
            let g = state().lock().unwrap_or_else(|e| e.into_inner());
            if g.selected_mount != sel_initial {
                selected = g.selected_mount.clone();
            }
        }
        ui.add_space(8.0);
        render_volume_cards(ui, &snap.volumes, &mut selected);
        ui.add_space(8.0);
        render_selection_split(ui, snap, raw_processes, &selected);
    });
    let mut guard = state().lock().unwrap_or_else(|e| e.into_inner());
    guard.selected_mount = selected;
}

/// Fixed 4-column strip spanning 100% width (zero-jitter, no wrapping).
/// Col 0: Blackbox state, Col 1: READ, Col 2: WRITE, Col 3: TOP I/O.
fn render_status_and_kpis(
    ui: &mut egui::Ui,
    snap: &StorageSnapshot,
    kpi_read: f32,
    kpi_write: f32,
    scope_disk: Option<u32>,
) {
    let read_label = match scope_disk {
        Some(d) => format!("READ Disk {d}"),
        None => "READ MB/s".to_owned(),
    };
    let write_label = match scope_disk {
        Some(d) => format!("WRITE Disk {d}"),
        None => "WRITE MB/s".to_owned(),
    };
    let top = snap
        .top_io_process
        .as_ref()
        .map(|t| truncate(&format!("{} {:.1}", t.1, t.2), 22))
        .unwrap_or_else(|| "-".to_owned());
    card_frame().show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.columns(4, |cols| {
            render_blackbox(&mut cols[0], snap);
            col_kpi(&mut cols[1], &read_label, format!("{:.1}", kpi_read), colors::ACCENT_CYAN);
            col_kpi(&mut cols[2], &write_label, format!("{:.1}", kpi_write), colors::ACCENT_AMBER);
            col_kpi(&mut cols[3], "TOP I/O", top, colors::TEXT_PRIMARY);
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn clear_selected_persists_after_show_all() {
        set_selected_for_test(Some(Arc::from("C:\\")));
        assert_eq!(selected_mount(), Some(Arc::from("C:\\")));
        clear_selected();
        assert_eq!(selected_mount(), None);
        let sel_initial: Option<Arc<str>> = Some(Arc::from("C:\\"));
        let g = state().lock().unwrap_or_else(|e| e.into_inner());
        assert!(g.selected_mount != sel_initial);
        assert_eq!(g.selected_mount, None);
        drop(g);
        set_selected_for_test(None);
    }
}
