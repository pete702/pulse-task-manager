use crate::modules::memory::action_bar::{format_trim_status, MemoryActionBarState};
use crate::modules::memory::actions;
use crate::modules::memory::model::RawProcessRow;
use crate::modules::memory::process_item::ProcessMemoryItem;
use crate::modules::memory::process_target::ProcessTarget;
use crate::modules::memory::table_state::MemoryTableState;
use eframe::egui;
use std::path::Path;

pub(super) fn render_context_menu(
    ui: &mut egui::Ui,
    item: &ProcessMemoryItem,
    _state: &mut MemoryTableState,
    action_state: &mut MemoryActionBarState,
) {
    if ui.button("Open File Location").clicked() {
        let pid = item.pid;
        let cached = item.exe_path.as_ref().map(|s| s.to_string());
        let name = item.name.clone();
        action_state.spawn(move || {
            let resolved =
                cached.or_else(|| crate::modules::memory::win32_api::query_process_exe_path(pid));
            match resolved {
                Some(p) => match actions::open_process_file_location(Path::new(&p)) {
                    Ok(()) => format!("Opened location for {}", name),
                    Err(e) => format!("Open location failed: {e}"),
                },
                None => format!("Open location failed: no path for {}", name),
            }
        });
        ui.close_menu();
    }
    if ui.button("Trim Working Set").clicked() {
        let items = vec![ProcessTarget::from_raw(&RawProcessRow {
            pid: item.pid,
            name: item.name.clone(),
            ram_bytes: item.ram_bytes,
            virtual_bytes: item.virtual_bytes,
            delta_bytes_per_sec: item.delta_bytes_per_sec,
            exe_path: item.exe_path.clone(),
            creation_time: item.creation_time,
        })];
        action_state.spawn(move || format_trim_status(&actions::trim_process_group(&items)));
        ui.close_menu();
    }
    if ui.button("Terminate Process").clicked() {
        crate::core::request_process_scan();
        let items = vec![ProcessTarget::from_raw(&RawProcessRow {
            pid: item.pid,
            name: item.name.clone(),
            ram_bytes: item.ram_bytes,
            virtual_bytes: item.virtual_bytes,
            delta_bytes_per_sec: item.delta_bytes_per_sec,
            exe_path: item.exe_path.clone(),
            creation_time: item.creation_time,
        })];
        action_state.spawn(move || match actions::terminate_process_group(&items) {
            Ok(n) => format!("End task: {n}/1 processes terminated"),
            Err(e) => format!("End task blocked: {e}"),
        });
        ui.close_menu();
    }
    if ui.button("Copy Name & PID").clicked() {
        ui.output_mut(|o| o.copied_text = format!("{} (PID {})", item.name, item.pid));
        ui.close_menu();
    }
}

pub(super) fn handle_double_click(
    item: &ProcessMemoryItem,
    action_state: &mut MemoryActionBarState,
) {
    let pid = item.pid;
    let cached = item.exe_path.as_ref().map(|s| s.to_string());
    let name = item.name.clone();
    action_state.spawn(move || {
        let resolved =
            cached.or_else(|| crate::modules::memory::win32_api::query_process_exe_path(pid));
        match resolved {
            Some(p) => match actions::open_process_file_location(Path::new(&p)) {
                Ok(()) => format!("Opened location for {}", name),
                Err(e) => format!("Open location failed: {e}"),
            },
            None => format!("Open location failed: no path for {}", name),
        }
    });
}
