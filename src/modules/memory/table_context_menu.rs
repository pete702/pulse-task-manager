use crate::modules::memory::model::RawProcessRow;
use crate::modules::memory::process_item::ProcessMemoryItem;
use crate::modules::memory::process_target::ProcessTarget;
use crate::modules::memory::table_state::MemoryTableState;
use eframe::egui;

pub(super) fn render_context_menu(
    ui: &mut egui::Ui,
    item: &ProcessMemoryItem,
    _state: &mut MemoryTableState,
) {
    if let Some(path) = item.exe_path.as_deref() {
        if ui.button("📂 Open File Location").clicked() {
            let _ = crate::modules::memory::actions::open_process_file_location(
                std::path::Path::new(path),
            );
            ui.close_menu();
        }
    }
    if ui.button("✂ Terminate Process").clicked() {
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
        crate::modules::memory::actions::terminate_process_group(&items);
        ui.close_menu();
    }
    if ui.button("📋 Copy Name & PID").clicked() {
        ui.output_mut(|o| o.copied_text = format!("{} (PID {})", item.name, item.pid));
        ui.close_menu();
    }
}

pub(super) fn handle_double_click(item: &ProcessMemoryItem) {
    if let Some(path) = item.exe_path.as_deref() {
        let _ =
            crate::modules::memory::actions::open_process_file_location(std::path::Path::new(path));
    }
}
