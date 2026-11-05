use crate::modules::memory::model::RawProcessRow;
use crate::modules::memory::process_item::ProcessMemoryItem;
use crate::modules::memory::process_target::ProcessTarget;
use crate::modules::memory::table_state::MemoryTableState;
use crate::modules::memory::win32_api::query_process_exe_path;
use eframe::egui;
use std::path::Path;

pub(super) fn render_context_menu(
    ui: &mut egui::Ui,
    item: &ProcessMemoryItem,
    _state: &mut MemoryTableState,
) {
    // Get exe_path from item, or query on-demand if not available (collector uses without_exe())
    let exe_path = item
        .exe_path
        .as_deref()
        .map(|s| s.to_string())
        .or_else(|| query_process_exe_path(item.pid));
    if let Some(ref path) = exe_path {
        if ui.button("◧ Open File Location").clicked() {
            let _ = crate::modules::memory::actions::open_process_file_location(Path::new(path));
            ui.close_menu();
        }
    }
    if ui.button("× Terminate Process").clicked() {
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
        let _ = crate::modules::memory::actions::terminate_process_group(&items);
        ui.close_menu();
    }
    if ui.button("◻ Copy Name & PID").clicked() {
        ui.output_mut(|o| o.copied_text = format!("{} (PID {})", item.name, item.pid));
        ui.close_menu();
    }
}

pub(super) fn handle_double_click(item: &ProcessMemoryItem) {
    let exe_path = item
        .exe_path
        .as_deref()
        .map(|s| s.to_string())
        .or_else(|| query_process_exe_path(item.pid));
    if let Some(ref path) = exe_path {
        let _ = crate::modules::memory::actions::open_process_file_location(Path::new(path));
    }
}
