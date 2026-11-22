//! Serialization/parsing helpers for persisted PULSE state.
use crate::app_tabs::ActiveTab;
use crate::modules::cpu::table_state::CpuSortCol;
use crate::modules::memory::model::ChartWindow;

pub fn active_tab_index(tab: ActiveTab) -> u8 {
    tab as u8
}
pub fn active_tab_from_index(idx: u8) -> ActiveTab {
    [
        ActiveTab::Processes,
        ActiveTab::Memory,
        ActiveTab::Cpu,
        ActiveTab::Storage,
        ActiveTab::Network,
        ActiveTab::System,
    ][idx.min(5) as usize]
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
pub fn serialize(s: &crate::core::persist::PersistedState) -> String {
    let b = |x: bool| if x { "1" } else { "0" };
    let ww = if s.win_w.is_finite() { s.win_w } else { 1180.0 }
        .clamp(crate::core::persist::MIN_WINDOW_WIDTH, 3840.0);
    let wh = if s.win_h.is_finite() { s.win_h } else { 760.0 }.clamp(560.0, 2160.0);
    let mut out = format!(
        "version=1\nactive_tab={}\ngroup_clones={}\ncompact_rows={}\nsort_col={}\nsort_asc={}\ncpu_show_grid={}\nmem_window_secs={}\ncpu_window_secs={}\nstorage_window_secs={}\nnetwork_window_secs={}\nnetwork_chart_mode={}\nnetwork_probe_enabled={}\nprocess_quick_filter={}\nwin_w={:.1}\nwin_h={:.1}\n",
        s.active_tab.min(5), b(s.group_clones), b(s.compact_rows), sort_col_str(s.sort_col),
        b(s.sort_asc), b(s.cpu_show_grid), s.mem_window_secs, s.cpu_window_secs,
        s.storage_window_secs, s.network_window_secs, s.network_chart_mode.min(2),
        b(s.network_probe_enabled), s.process_quick_filter.min(3), ww, wh,
    );
    if let Some(w) = s.cpu_col_widths {
        out.push_str(&format!(
            "cpu_col_widths={:.1},{:.1},{:.1},{:.1},{:.1},{:.1},{:.1}\n",
            w[0], w[1], w[2], w[3], w[4], w[5], w[6]
        ));
    }
    out.push_str(&format!("global_hotkey_enabled={}\n", b(s.global_hotkey_enabled)));
    out
}
pub fn parse(text: &str) -> crate::core::persist::PersistedState {
    let (mut s, mut i) = (crate::core::persist::PersistedState::default(), 0u32);
    for line in text.lines() {
        if let Some((k, v)) = line.split_once('=') {
            parse_kv(&mut s, k.trim(), v.trim(), &mut i);
        }
    }
    if i > 0 {
        eprintln!("pulse: state parse adjusted ({i})");
    }
    s
}
pub(crate) fn parse_kv(
    s: &mut crate::core::persist::PersistedState,
    k: &str,
    v: &str,
    i: &mut u32,
) {
    match k {
        "version" => {
            if v != "1" {
                *i += 1;
            }
        }
        "active_tab" => s.active_tab = clamp_u8(v, 5, i),
        "group_clones" => s.group_clones = parse_bool(v),
        "compact_rows" => s.compact_rows = parse_bool(v),
        "sort_col" => s.sort_col = sort_col_from_str(v),
        "sort_asc" => s.sort_asc = parse_bool(v),
        "cpu_show_grid" => s.cpu_show_grid = parse_bool(v),
        "mem_window_secs" => s.mem_window_secs = clamp_window(v, 60, i),
        "cpu_window_secs" => s.cpu_window_secs = clamp_window(v, 60, i),
        "storage_window_secs" => s.storage_window_secs = clamp_window(v, 300, i) as u64,
        "network_window_secs" => s.network_window_secs = clamp_window(v, 300, i) as u64,
        "network_chart_mode" => s.network_chart_mode = clamp_u8(v, 2, i),
        "network_probe_enabled" => s.network_probe_enabled = parse_bool(v),
        "process_quick_filter" => s.process_quick_filter = clamp_u8(v, 3, i),
        "win_w" => {
            s.win_w = parse_f32(v, 1180.0, crate::core::persist::MIN_WINDOW_WIDTH, 3840.0, i)
        }
        "win_h" => s.win_h = parse_f32(v, 760.0, 560.0, 2160.0, i),
        "cpu_col_widths" => s.cpu_col_widths = parse_col_widths(v, i),
        "global_hotkey_enabled" => s.global_hotkey_enabled = parse_bool(v),
        _ => *i += 1,
    }
}
fn clamp_u8(v: &str, max: u8, i: &mut u32) -> u8 {
    let raw = v.parse::<u8>().inspect_err(|_| *i += 1).unwrap_or(0);
    let c = raw.min(max);
    if c != raw {
        *i += 1;
    }
    c
}
fn clamp_window(v: &str, def: u32, i: &mut u32) -> u32 {
    let raw = v.parse::<u32>().inspect_err(|_| *i += 1).unwrap_or(def);
    let c = if matches!(raw, 60 | 300 | 900) { raw } else { 60 };
    if c != raw {
        *i += 1;
    }
    c
}
pub fn parse_col_widths(v: &str, i: &mut u32) -> Option<[f32; 7]> {
    let p: Vec<f32> = v.split(',').map(|x| x.trim().parse::<f32>().unwrap_or(f32::NAN)).collect();
    if p.len() != 7
        || !p.iter().all(|x| x.is_finite() && *x >= 24.0 && *x <= 1600.0)
        || p.iter().sum::<f32>() > 6000.0
    {
        *i += 1;
        return None;
    }
    Some([p[0], p[1], p[2], p[3], p[4], p[5], p[6]])
}
fn parse_f32(v: &str, def: f32, lo: f32, hi: f32, i: &mut u32) -> f32 {
    let raw = v.parse::<f32>().ok().filter(|x| x.is_finite());
    let val = raw.unwrap_or(def);
    let c = val.clamp(lo, hi);
    if raw.is_none() || (c - val).abs() > f32::EPSILON {
        *i += 1;
    }
    c
}
fn parse_bool(v: &str) -> bool {
    matches!(v.trim().to_ascii_lowercase().as_str(), "1" | "true" | "yes")
}
fn sort_col_str(col: CpuSortCol) -> &'static str {
    ["cpu", "name", "pid", "ram", "clones", "delta", "disk_io"][col as usize]
}
fn sort_col_from_str(v: &str) -> CpuSortCol {
    use CpuSortCol::*;
    [CpuPct, Name, Pid, RamBytes, CloneCount, Delta, DiskIo]
        .into_iter()
        .find(|&c| sort_col_str(c) == v.trim().to_ascii_lowercase())
        .unwrap_or(CpuPct)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn global_hotkey_enabled_key_roundtrips_and_defaults() {
        let text = serialize(&crate::core::persist::PersistedState::default());
        assert!(text.contains("global_hotkey_enabled=1"));
        assert!(!text.contains("hotkey_preset"));
        assert!(parse(&text).global_hotkey_enabled);

        let off = crate::core::persist::PersistedState {
            global_hotkey_enabled: false,
            ..Default::default()
        };
        let text = serialize(&off);
        assert!(text.contains("global_hotkey_enabled=0"));
        assert!(!parse(&text).global_hotkey_enabled);
        assert!(parse("version=1\nactive_tab=0\n").global_hotkey_enabled);
    }
}
