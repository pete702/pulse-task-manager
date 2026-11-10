//! Quick-filter pills for [00] Processes toolbar.
use crate::modules::cpu::table_state::CpuTableState;
use eframe::egui;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ProcessQuickFilter {
    #[default]
    All,
    ActiveLoad,
    HighRam,
    EcoQos,
}

impl ProcessQuickFilter {
    pub fn from_u8(v: u8) -> Self {
        match v {
            0 => Self::All,
            1 => Self::ActiveLoad,
            2 => Self::HighRam,
            3 => Self::EcoQos,
            _ => Self::All,
        }
    }
    pub fn to_u8(self) -> u8 {
        match self {
            Self::All => 0,
            Self::ActiveLoad => 1,
            Self::HighRam => 2,
            Self::EcoQos => 3,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            ProcessQuickFilter::All => "[ All ]",
            ProcessQuickFilter::ActiveLoad => "[ Active CPU/Disk ]",
            ProcessQuickFilter::HighRam => "[ RAM > 200 MB ]",
            ProcessQuickFilter::EcoQos => "[ EcoQoS ]",
        }
    }
    pub fn matches(self, cpu: f32, ram: u64, io: f32, eco: Option<bool>) -> bool {
        match self {
            Self::All => true,
            Self::ActiveLoad => cpu >= 1.0 || io > 0.0,
            Self::HighRam => ram >= 200 * 1024 * 1024,
            Self::EcoQos => eco == Some(true),
        }
    }
}

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

#[cfg(test)]
mod tests {
    use super::ProcessQuickFilter;

    #[test]
    fn quick_filter_roundtrip_fallback_and_labels() {
        for f in [
            ProcessQuickFilter::All,
            ProcessQuickFilter::ActiveLoad,
            ProcessQuickFilter::HighRam,
            ProcessQuickFilter::EcoQos,
        ] {
            assert_eq!(ProcessQuickFilter::from_u8(f.to_u8()), f);
            assert!(!f.label().is_empty());
        }
        assert_eq!(ProcessQuickFilter::from_u8(99), ProcessQuickFilter::All);
    }
}
