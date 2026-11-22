//! Pure helpers and item model for the command palette.
use crate::app_tabs::ActiveTab;
use crate::modules::cpu::model::RawProcessCpuRow;

const MAX_PROCESS_ROWS: usize = 12;

/// Case-insensitive token match: every whitespace token must appear in the label.
pub fn matches_palette_query(query: &str, label: &str) -> bool {
    let q = query.trim().to_ascii_lowercase();
    if q.is_empty() {
        return true;
    }
    let lower = label.to_ascii_lowercase();
    q.split_whitespace().all(|token| lower.contains(token))
}

pub fn clamp_palette_selection(selected: usize, count: usize) -> usize {
    if count == 0 {
        0
    } else {
        selected.min(count - 1)
    }
}

pub fn move_palette_selection(selected: usize, count: usize, delta: isize) -> usize {
    if count == 0 {
        return 0;
    }
    let current = clamp_palette_selection(selected, count) as isize;
    (current + delta).rem_euclid(count as isize) as usize
}

#[derive(Clone, Copy)]
pub enum PaletteAction {
    Tab(ActiveTab),
    ToggleMiniHud,
    TrimRam,
    ExportReceipt,
    PickWindow,
    HideToTray,
    Exit,
    SelectProcess(u32),
}

pub struct PaletteItem {
    pub label: String,
    pub action: PaletteAction,
}

const COMMANDS: &[(&str, PaletteAction)] = &[
    ("Tab: [01] CPU & Processes", PaletteAction::Tab(ActiveTab::Cpu)),
    ("Tab: [02] Memory", PaletteAction::Tab(ActiveTab::Memory)),
    ("Tab: [03] Storage", PaletteAction::Tab(ActiveTab::Storage)),
    ("Tab: [04] Network", PaletteAction::Tab(ActiveTab::Network)),
    ("Tab: [05] System & GPU", PaletteAction::Tab(ActiveTab::System)),
    ("Action: Toggle Mini-HUD", PaletteAction::ToggleMiniHud),
    ("Action: Trim RAM Working Sets", PaletteAction::TrimRam),
    ("Action: Export Lag-Receipt (CSV/JSON)", PaletteAction::ExportReceipt),
    ("Action: Pick Window Target", PaletteAction::PickWindow),
    ("Action: Hide to System Tray", PaletteAction::HideToTray),
    ("Action: Exit PULSE", PaletteAction::Exit),
];

pub fn build_items(processes: &[RawProcessCpuRow], query: &str) -> Vec<PaletteItem> {
    let mut items: Vec<PaletteItem> = COMMANDS
        .iter()
        .map(|(label, action)| PaletteItem { label: (*label).to_owned(), action: *action })
        .collect();
    let q = query.trim().to_ascii_lowercase();
    // Filter every live process first; only an empty query defaults to top-12 CPU.
    let mut top: Vec<&RawProcessCpuRow> = processes
        .iter()
        .filter(|p| p.pid > 4 && p.pid != std::process::id())
        .filter(|p| {
            if q.is_empty() {
                return true;
            }
            matches_palette_query(query, &format!("Process: {} (PID {})", p.name, p.pid))
        })
        .collect();
    top.sort_by(|a, b| b.cpu_usage.total_cmp(&a.cpu_usage));
    for process in top.into_iter().take(MAX_PROCESS_ROWS) {
        items.push(PaletteItem {
            label: format!("Process: {} (PID {})", process.name, process.pid),
            action: PaletteAction::SelectProcess(process.pid),
        });
    }
    items
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn palette_query_is_case_insensitive_token_match() {
        assert!(matches_palette_query("", "anything"));
        assert!(matches_palette_query("MEM", "Tab: [02] Memory"));
        assert!(matches_palette_query("tab memory", "Tab: [02] Memory"));
        assert!(matches_palette_query("exit pulse", "Action: Exit PULSE"));
        assert!(!matches_palette_query("storage", "Tab: [02] Memory"));
        assert!(!matches_palette_query("mem storage", "Tab: [02] Memory"));
    }

    #[test]
    fn selection_clamps_to_available_rows() {
        assert_eq!(clamp_palette_selection(5, 0), 0);
        assert_eq!(clamp_palette_selection(9, 3), 2);
        assert_eq!(clamp_palette_selection(1, 3), 1);
    }

    #[test]
    fn selection_wraps_in_both_directions() {
        assert_eq!(move_palette_selection(0, 3, -1), 2);
        assert_eq!(move_palette_selection(2, 3, 1), 0);
        assert_eq!(move_palette_selection(1, 3, 1), 2);
        assert_eq!(move_palette_selection(0, 0, 1), 0);
    }

    #[test]
    fn build_items_caps_process_rows_and_labels_them() {
        let processes: Vec<RawProcessCpuRow> = (0..20)
            .map(|i| RawProcessCpuRow {
                pid: 100 + i,
                name: std::sync::Arc::from("app"),
                ..Default::default()
            })
            .collect();
        let items = build_items(&processes, "");
        assert_eq!(items.len(), COMMANDS.len() + MAX_PROCESS_ROWS);
        assert!(items[COMMANDS.len()].label.starts_with("Process: app (PID "));
    }

    #[test]
    fn search_reaches_processes_past_top_twelve_cpu() {
        let mut processes: Vec<RawProcessCpuRow> = (0..20)
            .map(|i| RawProcessCpuRow {
                pid: 100 + i,
                name: std::sync::Arc::from("hot"),
                cpu_usage: 90.0 - i as f32,
                ..Default::default()
            })
            .collect();
        processes[19].name = std::sync::Arc::from("needle");
        processes[19].cpu_usage = 0.0;
        let items = build_items(&processes, "needle");
        assert!(items.iter().any(|i| i.label.contains("Process: needle (PID 119)")));
        assert!(!items.iter().any(|i| i.label.contains("Process: hot")));
    }
}
