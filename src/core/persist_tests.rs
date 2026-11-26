//! Persisted state tests.

use super::*;
use crate::app_tabs::ActiveTab;
use crate::modules::cpu::table_state::CpuSortCol;
use crate::modules::memory::model::ChartWindow;
use std::time::{SystemTime, UNIX_EPOCH};

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
        storage_window_secs: 300,
        network_window_secs: 300,
        network_chart_mode: 0,
        network_probe_enabled: true,
        process_quick_filter: 0,
        global_hotkey_enabled: true,
        sidebar_hidden: false,
        win_w: 1920.0,
        win_h: 1080.0,
        cpu_col_widths: None,
    };
    assert_eq!(parse(&serialize(&s)), s);
}
#[test]
fn roundtrip_new_fields() {
    let s = PersistedState {
        storage_window_secs: 900,
        network_window_secs: 60,
        network_chart_mode: 1,
        network_probe_enabled: false,
        process_quick_filter: 3,
        ..PersistedState::default()
    };
    let r = parse(&serialize(&s));
    assert_eq!(r, s);
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
#[test]
fn serialize_nan_inf_win_fallback() {
    let s = PersistedState { win_w: f32::NAN, win_h: f32::INFINITY, ..Default::default() };
    let out = serialize(&s);
    assert!(out.contains("win_w=1180.0") && out.contains("win_h=760.0"));
}
#[test]
fn parse_nan_inf_win_fallback() {
    let s = parse("win_w=NaN\nwin_h=inf\nwin_w=-inf\n");
    assert_eq!((s.win_w, s.win_h), (1180.0, 760.0));
}
#[test]
fn roundtrip_cpu_col_widths() {
    let s = PersistedState {
        cpu_col_widths: Some([56.0, 210.0, 64.0, 78.0, 140.0, 96.0, 88.0]),
        ..Default::default()
    };
    assert_eq!(parse(&serialize(&s)).cpu_col_widths, s.cpu_col_widths);
}
#[test]
fn version1_roundtrip_and_v0_compat() {
    let s = PersistedState {
        cpu_col_widths: Some([56.0, 210.0, 64.0, 78.0, 140.0, 96.0, 88.0]),
        ..Default::default()
    };
    let text = serialize(&s);
    assert!(text.starts_with("version=1\n"));
    assert_eq!(parse(&text), s);
    let old = text.replacen("version=1\n", "", 1);
    assert_eq!(parse(&old), s);
}
#[test]
fn parse_unknown_key_and_coercion_handling() {
    let s = PersistedState::default();
    let base = serialize(&s);
    let bad = format!("version=9\n{base}unknown_key=1\nactive_tab=99\n");
    let c = parse(&bad);
    assert_eq!(c.active_tab, 5);
    assert_eq!(c.cpu_col_widths, s.cpu_col_widths);
}
#[test]
fn parse_col_widths_accepts_valid() {
    let mut iss = 0u32;
    assert_eq!(
        parse_col_widths("56,210,64,78,140,96,88", &mut iss),
        Some([56.0, 210.0, 64.0, 78.0, 140.0, 96.0, 88.0])
    );
    assert_eq!(iss, 0);
}
#[test]
fn parse_col_widths_rejects_invalid() {
    let mut iss = 0u32;
    assert!(parse_col_widths("56,NaN,64,78,140,96,88", &mut iss).is_none());
    assert!(parse_col_widths("56,inf,64,78,140,96,88", &mut iss).is_none());
    assert!(parse_col_widths("56,1600.1,64,78,140,96,88", &mut iss).is_none());
    assert!(parse_col_widths("1500,1500,1500,1500,10,10,10", &mut iss).is_none());
    assert!(parse_col_widths("56,210,64,78,140,96", &mut iss).is_none());
    assert!(parse_col_widths("23.9,210,64,78,140,96,88", &mut iss).is_none());
    assert!(iss > 0);
}
#[test]
fn write_config_file_rejects_tmp_collision() {
    let uniq = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let dir = std::env::temp_dir().join(format!("pulse_persist_test_{uniq}"));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("state.cfg");
    let tmp = dir.join("state.cfg.tmp");
    std::fs::create_dir_all(&tmp).unwrap();
    let res = write_config_file(&path, "version=1\nactive_tab=0\n");
    assert!(res.is_err(), "expected fail-closed on tmp collision");
    assert!(!path.exists(), "destination must not be written on collision");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn save_worker_join_drains_queued_writes() {
    let (tx, rx) = std::sync::mpsc::sync_channel::<(PathBuf, String)>(2);
    let (seen_tx, seen_rx) = std::sync::mpsc::channel();
    let handle = std::thread::spawn(move || {
        while let Ok((_p, s)) = rx.recv() {
            let _ = seen_tx.send(s);
        }
    });
    for i in 0..4 {
        tx.send((PathBuf::from("state.cfg"), format!("v{i}"))).unwrap();
    }
    drop(tx);
    handle.join().unwrap();
    let got: Vec<String> = seen_rx.try_iter().collect();
    assert_eq!(got, vec!["v0", "v1", "v2", "v3"]);
}

#[test]
fn join_save_worker_is_idempotent() {
    join_save_worker();
    join_save_worker();
}
