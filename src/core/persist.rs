//! App state persistence for PULSE (zero external crates, std only).
//! Stores key=value lines in %LOCALAPPDATA%\PULSE\state.cfg.
use crate::app_tabs::ActiveTab;
use crate::modules::cpu::table_state::CpuSortCol;
use crate::modules::memory::model::ChartWindow;
use std::path::PathBuf;

/// Persisted UI state across restarts.
#[derive(Debug, Clone, PartialEq)]
pub struct PersistedState {
    pub active_tab: u8,
    pub group_clones: bool,
    pub compact_rows: bool,
    pub sort_col: CpuSortCol,
    pub sort_asc: bool,
    pub cpu_show_grid: bool,
    pub mem_window_secs: u32,
    pub cpu_window_secs: u32,
    pub win_w: f32,
    pub win_h: f32,
}

impl Default for PersistedState {
    fn default() -> Self {
        Self {
            active_tab: 0,
            group_clones: true,
            compact_rows: true,
            sort_col: CpuSortCol::CpuPct,
            sort_asc: false,
            cpu_show_grid: false,
            mem_window_secs: 300,
            cpu_window_secs: 300,
            win_w: 1180.0,
            win_h: 760.0,
        }
    }
}

/// Path to state file: %LOCALAPPDATA%\PULSE\state.cfg, None when unavailable.
pub fn state_path() -> Option<PathBuf> {
    let base = std::env::var_os("LOCALAPPDATA")?;
    Some(PathBuf::from(base).join("PULSE").join("state.cfg"))
}

/// Loads persisted state, or defaults when the file is missing or unreadable.
pub fn load_state() -> PersistedState {
    let Some(path) = state_path() else { return PersistedState::default() };
    match std::fs::read_to_string(path) {
        Ok(text) => parse(&text),
        Err(_) => PersistedState::default(),
    }
}

/// Saves state to disk. No-op when LOCALAPPDATA is unavailable.
pub fn save_state(s: &PersistedState) -> std::io::Result<()> {
    let Some(path) = state_path() else { return Ok(()) };
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, serialize(s))
}

/// Serializes state to key=value text (fixed key order).
pub fn serialize(s: &PersistedState) -> String {
    format!(
        "active_tab={}\ngroup_clones={}\ncompact_rows={}\nsort_col={}\nsort_asc={}\ncpu_show_grid={}\nmem_window_secs={}\ncpu_window_secs={}\nwin_w={:.1}\nwin_h={:.1}\n",
        s.active_tab,
        bool_str(s.group_clones),
        bool_str(s.compact_rows),
        sort_col_str(s.sort_col),
        bool_str(s.sort_asc),
        bool_str(s.cpu_show_grid),
        s.mem_window_secs,
        s.cpu_window_secs,
        s.win_w.clamp(860.0, 3840.0),
        s.win_h.clamp(560.0, 2160.0),
    )
}

/// Parses key=value text into state. Unknown or malformed lines are ignored.
pub fn parse(text: &str) -> PersistedState {
    let mut s = PersistedState::default();
    for line in text.lines() {
        let Some((k, v)) = line.split_once('=') else { continue };
        let (k, v) = (k.trim(), v.trim());
        match k {
            "active_tab" => s.active_tab = v.parse::<u8>().unwrap_or(0).min(5),
            "group_clones" => s.group_clones = parse_bool(v),
            "compact_rows" => s.compact_rows = parse_bool(v),
            "sort_col" => s.sort_col = sort_col_from_str(v),
            "sort_asc" => s.sort_asc = parse_bool(v),
            "cpu_show_grid" => s.cpu_show_grid = parse_bool(v),
            "mem_window_secs" => s.mem_window_secs = valid_window(v.parse().unwrap_or(60)),
            "cpu_window_secs" => s.cpu_window_secs = valid_window(v.parse().unwrap_or(60)),
            "win_w" => s.win_w = v.parse::<f32>().unwrap_or(1180.0).clamp(860.0, 3840.0),
            "win_h" => s.win_h = v.parse::<f32>().unwrap_or(760.0).clamp(560.0, 2160.0),
            _ => {}
        }
    }
    s
}

fn bool_str(b: bool) -> &'static str {
    if b {
        "1"
    } else {
        "0"
    }
}

fn parse_bool(v: &str) -> bool {
    matches!(v.trim().to_ascii_lowercase().as_str(), "1" | "true" | "yes")
}

fn valid_window(secs: u32) -> u32 {
    if secs == 60 || secs == 300 || secs == 900 {
        secs
    } else {
        60
    }
}

/// Tab index: 0 Processes .. 5 System.
pub fn active_tab_index(tab: ActiveTab) -> u8 {
    match tab {
        ActiveTab::Processes => 0,
        ActiveTab::Memory => 1,
        ActiveTab::Cpu => 2,
        ActiveTab::Storage => 3,
        ActiveTab::Network => 4,
        ActiveTab::System => 5,
    }
}

/// Tab from clamped index.
pub fn active_tab_from_index(idx: u8) -> ActiveTab {
    match idx.min(5) {
        1 => ActiveTab::Memory,
        2 => ActiveTab::Cpu,
        3 => ActiveTab::Storage,
        4 => ActiveTab::Network,
        5 => ActiveTab::System,
        _ => ActiveTab::Processes,
    }
}

/// Chart window from seconds.
pub fn chart_window_from_secs(secs: u32) -> ChartWindow {
    match secs {
        300 => ChartWindow::FiveMin,
        900 => ChartWindow::FifteenMin,
        _ => ChartWindow::OneMin,
    }
}

/// Seconds from chart window.
pub fn secs_from_chart_window(w: ChartWindow) -> u32 {
    w.seconds()
}

fn sort_col_str(col: CpuSortCol) -> &'static str {
    match col {
        CpuSortCol::Pid => "pid",
        CpuSortCol::Name => "name",
        CpuSortCol::CpuPct => "cpu",
        CpuSortCol::RamBytes => "ram",
        CpuSortCol::Delta => "delta",
        CpuSortCol::CloneCount => "clones",
    }
}

fn sort_col_from_str(v: &str) -> CpuSortCol {
    match v.trim().to_ascii_lowercase().as_str() {
        "pid" => CpuSortCol::Pid,
        "name" => CpuSortCol::Name,
        "ram" => CpuSortCol::RamBytes,
        "delta" => CpuSortCol::Delta,
        "clones" => CpuSortCol::CloneCount,
        _ => CpuSortCol::CpuPct,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn roundtrip_default() {
        let s = PersistedState::default();
        assert_eq!(parse(&serialize(&s)), s);
    }
    #[test]
    fn roundtrip_custom() {
        let s = PersistedState {
            active_tab: 2,
            group_clones: false,
            compact_rows: false,
            sort_col: CpuSortCol::RamBytes,
            sort_asc: true,
            cpu_show_grid: true,
            mem_window_secs: 900,
            cpu_window_secs: 300,
            win_w: 1920.0,
            win_h: 1080.0,
        };
        assert_eq!(parse(&serialize(&s)), s);
    }
    #[test]
    fn parse_clamps_invalid() {
        let s = parse("active_tab=99\nmem_window_secs=42\nwin_w=1.0\nwin_h=9999.0\n");
        assert_eq!(s.active_tab, 5);
        assert_eq!(s.mem_window_secs, 60);
        assert_eq!(s.win_w, 860.0);
        assert_eq!(s.win_h, 2160.0);
    }
    #[test]
    fn parse_ignores_garbage() {
        let s = parse("");
        assert_eq!(s, PersistedState::default());
        let s2 = parse("unknown=1\nbadline\nsort_col=delta\nsort_asc=1\n");
        assert_eq!(s2.sort_col, CpuSortCol::Delta);
        assert!(s2.sort_asc);
    }
    #[test]
    fn tab_and_window_maps() {
        assert_eq!(active_tab_index(ActiveTab::Processes), 0);
        assert_eq!(active_tab_from_index(4), ActiveTab::Network);
        assert_eq!(chart_window_from_secs(300), ChartWindow::FiveMin);
        assert_eq!(secs_from_chart_window(ChartWindow::FifteenMin), 900);
    }
}
