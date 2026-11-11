//! Live `[03] Storage` HUD: master-detail cockpit split.
use crate::core::SystemSnapshot;
use crate::modules::cpu::model::RawProcessCpuRow;
use crate::modules::storage::chart_view::render_storage_chart;
use crate::modules::storage::inspector::render_selection_split;
use crate::modules::storage::model::{DiskVolumeInfo, StorageHistory, StorageSnapshot};
use crate::modules::storage::physical_disk::drive_letter;
use crate::modules::storage::view_helpers::{render_status_and_kpis, scope_label_for};
use crate::modules::storage::volume_cards::{render_all_selector, render_volume_cards};
use eframe::egui;
use std::sync::{Arc, Mutex, OnceLock};
struct StorageViewState {
    history: StorageHistory,
    window_secs: usize,
    last_sample_ts: Option<f64>,
    selected_mount: Option<Arc<str>>,
    show_read: bool,
    show_write: bool,
}
impl Default for StorageViewState {
    fn default() -> Self {
        Self {
            history: StorageHistory::new(),
            window_secs: 300,
            last_sample_ts: None,
            selected_mount: None,
            show_read: true,
            show_write: true,
        }
    }
}
static STATE: OnceLock<Mutex<StorageViewState>> = OnceLock::new();
fn state() -> &'static Mutex<StorageViewState> {
    STATE.get_or_init(|| Mutex::new(StorageViewState::default()))
}
/// Updates the chart window (called from chart window buttons).
pub fn set_window_secs(secs: usize) {
    let mut g = state().lock().unwrap_or_else(|e| e.into_inner());
    g.window_secs = secs;
}
pub fn set_show_read(v: bool) {
    let mut g = state().lock().unwrap_or_else(|e| e.into_inner());
    g.show_read = v || (g.show_read && !g.show_write);
}
pub fn set_show_write(v: bool) {
    let mut g = state().lock().unwrap_or_else(|e| e.into_inner());
    g.show_write = v || (g.show_write && !g.show_read);
}
pub fn window_secs() -> usize {
    state().lock().unwrap_or_else(|e| e.into_inner()).window_secs
}
/// Clears selection back to All Disks (called from chart scope button).
pub fn clear_selected() {
    let mut g = state().lock().unwrap_or_else(|e| e.into_inner());
    g.selected_mount = None;
}
/// Current selected mount (None = All Disks).
#[allow(dead_code)]
pub fn selected_mount() -> Option<Arc<str>> {
    state().lock().unwrap_or_else(|e| e.into_inner()).selected_mount.clone()
}
#[cfg(test)]
pub(crate) fn set_selected_for_test(value: Option<Arc<str>>) {
    let mut g = state().lock().unwrap_or_else(|e| e.into_inner());
    g.selected_mount = value;
}
fn history_key_for_volume(v: &DiskVolumeInfo) -> Option<u32> {
    v.physical_disk
        .or_else(|| drive_letter(v.mount_point.as_ref()).map(|ch| 0x8000_0000 | (ch as u32)))
}
/// Background ingestion on worker pause-adjusted timestamps.
pub fn ingest_storage_snapshot(snap: &std::sync::Arc<SystemSnapshot>) {
    let mut guard = state().lock().unwrap_or_else(|e| e.into_inner());
    let elapsed = snap.memory.as_ref().map(|m| m.sample_elapsed_secs).unwrap_or(0.0);
    let due = guard.last_sample_ts.is_none_or(|last| elapsed > last);
    if !due {
        return;
    }
    guard.history.push(elapsed, snap.storage.read_mb_s, snap.storage.write_mb_s);
    let mut seen: Vec<u32> = Vec::new();
    for vol in snap.storage.volumes.iter() {
        if let Some(d) = history_key_for_volume(vol) {
            if seen.contains(&d) {
                continue;
            }
            seen.push(d);
            guard.history.push_disk(d, elapsed, vol.read_mb_s, vol.write_mb_s);
        }
    }
    guard.last_sample_ts = Some(elapsed);
}
fn sync_sel(selected: &mut Option<Arc<str>>, init: &Option<Arc<str>>) {
    let g = state().lock().unwrap_or_else(|e| e.into_inner());
    if g.selected_mount != *init {
        *selected = g.selected_mount.clone();
    }
}
#[allow(clippy::too_many_arguments)]
fn status_chart(
    ui: &mut egui::Ui,
    snap: &StorageSnapshot,
    r: &Arc<Vec<[f64; 2]>>,
    w: &Arc<Vec<[f64; 2]>>,
    window: usize,
    scope: Option<String>,
    sr: bool,
    sw: bool,
    kr: f32,
    kw: f32,
    disk: Option<u32>,
    selected: &mut Option<Arc<str>>,
    init: &Option<Arc<str>>,
) {
    render_status_and_kpis(ui, snap, kr, kw, disk);
    ui.add_space(8.0);
    render_storage_chart(ui, r, w, window, scope.as_deref(), sr, sw);
    sync_sel(selected, init);
}
/// Renders the Storage tab as master-detail cockpit or narrow stack.
pub fn render_storage_tab(
    ui: &mut egui::Ui,
    snap: &StorageSnapshot,
    raw_processes: &[RawProcessCpuRow],
) {
    let mut selected = {
        let guard = state().lock().unwrap_or_else(|e| e.into_inner());
        guard.selected_mount.clone()
    };
    let sel_initial = selected.clone();
    if selected.is_some() {
        let valid = snap.volumes.iter().any(|v| selected.as_ref() == Some(&v.mount_point));
        if !valid {
            selected = None;
        }
    }
    let scope_disk: Option<u32> = selected.as_ref().and_then(|sel| {
        snap.volumes.iter().find(|v| &v.mount_point == sel).and_then(history_key_for_volume)
    });
    let kpi = match scope_disk {
        Some(d) => snap
            .volumes
            .iter()
            .find(|v| history_key_for_volume(v) == Some(d))
            .map(|v| (v.read_mb_s, v.write_mb_s))
            .unwrap_or((snap.read_mb_s, snap.write_mb_s)),
        None => (snap.read_mb_s, snap.write_mb_s),
    };
    let scope_label = scope_label_for(snap, scope_disk, history_key_for_volume);
    let (rs, ws, window, sr, sw) = {
        let mut guard = state().lock().unwrap_or_else(|e| e.into_inner());
        let win = guard.window_secs;
        let (r, w) = guard.history.series_points_scoped(scope_disk, win);
        (r, w, win, guard.show_read, guard.show_write)
    };
    if ui.available_width() >= 680.0 {
        let full_h = ui.available_height().max(320.0);
        let full_w = ui.available_width();
        ui.horizontal_top(|ui| {
            ui.allocate_ui_with_layout(
                egui::vec2(280.0, full_h),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    ui.set_min_size(egui::vec2(280.0, full_h));
                    render_all_selector(ui, snap.read_mb_s, snap.write_mb_s, &mut selected);
                    ui.add_space(6.0);
                    egui::ScrollArea::vertical()
                        .max_height(ui.available_height().max(200.0))
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            render_volume_cards(ui, &snap.volumes, &mut selected);
                        });
                },
            );
            let right_w = (full_w - 290.0).max(360.0);
            ui.allocate_ui_with_layout(
                egui::vec2(right_w, full_h),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    status_chart(
                        ui,
                        snap,
                        &rs,
                        &ws,
                        window,
                        scope_label.clone(),
                        sr,
                        sw,
                        kpi.0,
                        kpi.1,
                        scope_disk,
                        &mut selected,
                        &sel_initial,
                    );
                    ui.add_space(8.0);
                    render_selection_split(ui, snap, raw_processes, &selected);
                },
            );
        });
    } else {
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            status_chart(
                ui,
                snap,
                &rs,
                &ws,
                window,
                scope_label,
                sr,
                sw,
                kpi.0,
                kpi.1,
                scope_disk,
                &mut selected,
                &sel_initial,
            );
            ui.add_space(8.0);
            render_all_selector(ui, snap.read_mb_s, snap.write_mb_s, &mut selected);
            ui.add_space(8.0);
            render_volume_cards(ui, &snap.volumes, &mut selected);
            ui.add_space(8.0);
            render_selection_split(ui, snap, raw_processes, &selected);
        });
    }
    let mut guard = state().lock().unwrap_or_else(|e| e.into_inner());
    guard.selected_mount = selected;
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn clear_selected_persists_after_show_all() {
        set_selected_for_test(Some(Arc::from("C:\\")));
        clear_selected();
        assert_eq!(selected_mount(), None);
        set_selected_for_test(None);
    }
}
