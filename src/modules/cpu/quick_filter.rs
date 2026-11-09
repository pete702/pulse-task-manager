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
        if ui.add(btn).clicked() && !active {
            state.set_quick_filter(f);
            changed = true;
        }
    }
    changed
}
