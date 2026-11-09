//! App state persistence for PULSE (std only). Stores key=value in %LOCALAPPDATA%\PULSE\state.cfg.
use crate::app_tabs::ActiveTab;
use crate::modules::cpu::table_state::CpuSortCol;
use crate::modules::memory::model::ChartWindow;
use std::io::Read;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
static STAGED_STATE: OnceLock<Mutex<Option<PersistedState>>> = OnceLock::new();
pub fn stage_state(state: &PersistedState) {
    let slot = STAGED_STATE.get_or_init(|| Mutex::new(None));
    *slot.lock().unwrap_or_else(|e| e.into_inner()) = Some(state.clone());
}
pub fn clear_staged_state() {
    if let Some(slot) = STAGED_STATE.get() {
        *slot.lock().unwrap_or_else(|e| e.into_inner()) = None;
    }
}
pub fn flush_staged_state() {
    let staged = STAGED_STATE.get().and_then(|slot| {
        let mut guard = slot.lock().unwrap_or_else(|e| e.into_inner());
        guard.take()
    });
    if let Some(s) = staged {
        let _ = save_state(&s);
    }
}
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
    pub storage_window_secs: u64,
    pub network_window_secs: u64,
    pub network_chart_mode: u8,
    pub network_probe_enabled: bool,
    pub process_quick_filter: u8,
    pub win_w: f32,
    pub win_h: f32,
    pub cpu_col_widths: Option<[f32; 7]>,
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
            storage_window_secs: 300,
            network_window_secs: 300,
            network_chart_mode: 0,
            network_probe_enabled: true,
            process_quick_filter: 0,
            win_w: 1180.0,
            win_h: 760.0,
            cpu_col_widths: None,
        }
    }
}
pub fn state_path() -> Option<PathBuf> {
    let base = std::env::var_os("LOCALAPPDATA")?;
    Some(PathBuf::from(base).join("PULSE").join("state.cfg"))
}
pub fn load_state() -> PersistedState {
    let Some(path) = state_path() else { return PersistedState::default() };
    let Ok(file) = std::fs::File::open(&path) else { return PersistedState::default() };
    let mut buf = Vec::new();
    if file.take(64 * 1024).read_to_end(&mut buf).is_err() {
        return PersistedState::default();
    }
    match String::from_utf8(buf) {
        Ok(text) => parse(&text),
        Err(_) => PersistedState::default(),
    }
}
pub fn save_state(s: &PersistedState) -> std::io::Result<()> {
    let Some(path) = state_path() else { return Ok(()) };
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let tmp_path = path.with_file_name("state.cfg.tmp");
    if std::fs::symlink_metadata(&tmp_path).map(|m| m.file_type().is_symlink()).unwrap_or(false) {
        return Ok(());
    }
    std::fs::write(&tmp_path, serialize(s))?;
    if std::fs::rename(&tmp_path, &path).is_err() {
        let _ = std::fs::remove_file(&tmp_path);
        std::fs::write(&path, serialize(s))?;
    }
    clear_staged_state();
    Ok(())
}
pub fn serialize(s: &PersistedState) -> String {
    let win_w = if s.win_w.is_finite() { s.win_w } else { 1180.0 };
    let win_h = if s.win_h.is_finite() { s.win_h } else { 760.0 };
    let mut out = format!(
        "active_tab={}\ngroup_clones={}\ncompact_rows={}\nsort_col={}\nsort_asc={}\ncpu_show_grid={}\nmem_window_secs={}\ncpu_window_secs={}\nstorage_window_secs={}\nnetwork_window_secs={}\nnetwork_chart_mode={}\nnetwork_probe_enabled={}\nprocess_quick_filter={}\nwin_w={:.1}\nwin_h={:.1}\n",
        s.active_tab.min(5), bool_str(s.group_clones), bool_str(s.compact_rows), sort_col_str(s.sort_col),
        bool_str(s.sort_asc), bool_str(s.cpu_show_grid), s.mem_window_secs, s.cpu_window_secs,
        s.storage_window_secs, s.network_window_secs, s.network_chart_mode.min(2),
        bool_str(s.network_probe_enabled), s.process_quick_filter.min(3), win_w.clamp(860.0, 3840.0),
        win_h.clamp(560.0, 2160.0),
    );
    if let Some(w) = s.cpu_col_widths {
        out.push_str(&format!(
            "cpu_col_widths={:.1},{:.1},{:.1},{:.1},{:.1},{:.1},{:.1}\n",
            w[0], w[1], w[2], w[3], w[4], w[5], w[6]
        ));
    }
    out
}
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
            "storage_window_secs" => {
                s.storage_window_secs = valid_window(v.parse().unwrap_or(300)) as u64
            }
            "network_window_secs" => {
                s.network_window_secs = valid_window(v.parse().unwrap_or(300)) as u64
            }
            "network_chart_mode" => s.network_chart_mode = v.parse::<u8>().unwrap_or(0).min(2),
            "network_probe_enabled" => s.network_probe_enabled = parse_bool(v),
            "process_quick_filter" => s.process_quick_filter = v.parse::<u8>().unwrap_or(0).min(3),
            "win_w" => s.win_w = parse_f32_clamp(v, 1180.0, 860.0, 3840.0),
            "win_h" => s.win_h = parse_f32_clamp(v, 760.0, 560.0, 2160.0),
            "cpu_col_widths" => s.cpu_col_widths = parse_col_widths(v),
            _ => {}
        }
    }
    s
}
fn parse_col_widths(v: &str) -> Option<[f32; 7]> {
    let parts: Vec<f32> =
        v.split(',').map(|p| p.trim().parse::<f32>().unwrap_or(f32::NAN)).collect();
    if parts.len() != 7 || !parts.iter().all(|x| x.is_finite() && *x >= 32.0) {
        return None;
    }
    Some([parts[0], parts[1], parts[2], parts[3], parts[4], parts[5], parts[6]])
}
fn parse_f32_clamp(v: &str, def: f32, lo: f32, hi: f32) -> f32 {
    v.parse::<f32>().ok().filter(|x: &f32| x.is_finite()).unwrap_or(def).clamp(lo, hi)
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
pub fn chart_window_from_secs(secs: u32) -> ChartWindow {
    match secs {
        300 => ChartWindow::FiveMin,
        900 => ChartWindow::FifteenMin,
        _ => ChartWindow::OneMin,
    }
}
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
        CpuSortCol::DiskIo => "disk_io",
    }
}
fn sort_col_from_str(v: &str) -> CpuSortCol {
    match v.trim().to_ascii_lowercase().as_str() {
        "pid" => CpuSortCol::Pid,
        "name" => CpuSortCol::Name,
        "ram" => CpuSortCol::RamBytes,
        "delta" => CpuSortCol::Delta,
        "clones" => CpuSortCol::CloneCount,
        "disk_io" => CpuSortCol::DiskIo,
        _ => CpuSortCol::CpuPct,
    }
}
#[cfg(test)]
#[path = "persist_tests.rs"]
mod tests;
