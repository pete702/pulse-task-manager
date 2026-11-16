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
use std::sync::Arc;

/// Minimum width (px) at which the storage tab switches to the side-by-side cockpit layout.
const STORAGE_SPLIT_MIN_WIDTH: f32 = 680.0;

#[derive(Default)]
pub struct StorageViewState {
    pub history: StorageHistory,
    pub window_secs: usize,
    pub last_sample_ts: Option<f64>,
    pub selected_mount: Option<Arc<str>>,
    pub show_read: bool,
    pub show_write: bool,
}

impl StorageViewState {
    pub fn new() -> Self {
        Self { window_secs: 300, show_read: true, show_write: true, ..Default::default() }
    }
    pub fn set_window_secs(&mut self, secs: usize) {
        self.window_secs = secs;
    }
    pub fn window_secs(&self) -> usize {
        self.window_secs
    }
    pub fn set_show_read(&mut self, v: bool) {
        self.show_read = v || (self.show_read && !self.show_write);
    }
    pub fn set_show_write(&mut self, v: bool) {
        self.show_write = v || (self.show_write && !self.show_read);
    }
    pub fn clear_selected(&mut self) {
        self.selected_mount = None;
    }
    #[cfg(test)]
    pub fn set_selected_for_test(&mut self, value: Option<Arc<str>>) {
        self.selected_mount = value;
    }
}

pub fn ingest_storage_snapshot(
    state: &mut StorageViewState,
    snap: &std::sync::Arc<SystemSnapshot>,
) {
    let guard = state;
    let elapsed = snap.memory.as_ref().map(|m| m.sample_elapsed_secs).unwrap_or(0.0);
    let due = guard.last_sample_ts.is_none_or(|last| elapsed > last);
    if !due {
        return;
    }
    guard.history.push(elapsed, snap.storage.read_mb_s, snap.storage.write_mb_s);
    let mut seen: std::collections::HashSet<u32> = std::collections::HashSet::new();
    for vol in snap.storage.volumes.iter() {
        if let Some(d) = history_key_for_volume(vol) {
            if !seen.insert(d) {
                continue;
            }
            guard.history.push_disk(d, elapsed, vol.read_mb_s, vol.write_mb_s);
        }
    }
    guard.last_sample_ts = Some(elapsed);
}

fn history_key_for_volume(v: &DiskVolumeInfo) -> Option<u32> {
    v.physical_disk
        .or_else(|| drive_letter(v.mount_point.as_ref()).map(|ch| 0x8000_0000 | (ch as u32)))
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
    state: &mut StorageViewState,
) {
    render_status_and_kpis(ui, snap, kr, kw, disk);
    ui.add_space(8.0);
    render_storage_chart(ui, r, w, window, scope.as_deref(), sr, sw, state);
    let _ = (selected, init);
}

pub fn render_storage_tab(
    ui: &mut egui::Ui,
    snap: &StorageSnapshot,
    raw_processes: &[RawProcessCpuRow],
    state: &mut StorageViewState,
) {
    let mut selected = state.selected_mount.clone();
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
        let win = state.window_secs;
        let (r, w) = state.history.series_points_scoped(scope_disk, win);
        (r, w, win, state.show_read, state.show_write)
    };
    if ui.available_width() >= STORAGE_SPLIT_MIN_WIDTH {
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
                        state,
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
                state,
            );
            ui.add_space(8.0);
            render_all_selector(ui, snap.read_mb_s, snap.write_mb_s, &mut selected);
            ui.add_space(8.0);
            render_volume_cards(ui, &snap.volumes, &mut selected);
            ui.add_space(8.0);
            render_selection_split(ui, snap, raw_processes, &selected);
        });
    }
    state.selected_mount = selected;
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn clear_selected_persists_after_show_all() {
        let mut s = StorageViewState::new();
        s.set_selected_for_test(Some(Arc::from("C:\\")));
        s.clear_selected();
        assert_eq!(s.selected_mount, None);
        s.set_selected_for_test(None);
    }
}
