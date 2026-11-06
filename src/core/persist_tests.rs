//! Persisted state tests.

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
        storage_window_secs: 300,
        network_window_secs: 300,
        network_chart_mode: 0,
        network_probe_enabled: true,
        process_quick_filter: 0,
        win_w: 1920.0,
        win_h: 1080.0,
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
    assert_eq!(r.storage_window_secs, 900);
    assert_eq!(r.network_window_secs, 60);
    assert_eq!(r.network_chart_mode, 1);
    assert!(!r.network_probe_enabled);
    assert_eq!(r.process_quick_filter, 3);
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
