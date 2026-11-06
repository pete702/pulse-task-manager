//! Quick-filter pills for [00] Processes toolbar.
use crate::modules::cpu::table_state::{CpuTableState, ProcessQuickFilter};
use eframe::egui;

/// Renders compact [ All | Active | RAM | Eco ] pills; returns true if changed.
pub fn render_quick_filters(ui: &mut egui::Ui, state: &mut CpuTableState) -> bool {
    let mut changed = false;
    for f in [
        ProcessQuickFilter::All,
        ProcessQuickFilter::ActiveLoad,
        ProcessQuickFilter::HighRam,
        ProcessQuickFilter::EcoQos,
    ] {
        let active = state.quick_filter == f;
        let btn = egui::Button::new(egui::RichText::new(f.label()).monospace().size(10.0)).fill(
            if active {
                egui::Color32::from_rgba_unmultiplied(0, 240, 255, 40)
            } else {
                egui::Color32::TRANSPARENT
            },
        );
        if ui.add(btn).clicked() {
            state.set_quick_filter(f);
            changed = true;
        }
    }
    changed
}
/// Returns true when a raw row passes the quick filter.
#[allow(dead_code)]
pub fn passes(f: ProcessQuickFilter, cpu_pct: f32, ram: u64, io: f32, eco: Option<bool>) -> bool {
    f.matches(cpu_pct, ram, io, eco)
}
