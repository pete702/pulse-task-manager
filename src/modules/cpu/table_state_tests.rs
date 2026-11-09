//! Unit tests for the CPU table cache and clone grouping.

use super::*;
use crate::modules::cpu::collector::pid_contains;
use crate::modules::cpu::model::CpuSnapshot;

fn raw(pid: u32, name: &str, cpu: f32, ram: u64) -> RawProcessCpuRow {
    RawProcessCpuRow {
        pid,
        name: Arc::from(name),
        cpu_usage: cpu,
        ram_bytes: ram,
        ..Default::default()
    }
}
fn raw_st(pid: u32, name: &str, cpu: f32, ram: u64, st: u64) -> RawProcessCpuRow {
    let mut r = raw(pid, name, cpu, ram);
    r.start_time = st;
    r
}

#[test]
fn default_groups_clones_and_sorts_by_cpu_desc() {
    let state = CpuTableState::default();
    assert!(state.group_clones);
    assert_eq!(state.sort_col, CpuSortCol::CpuPct);
    assert!(!state.sort_asc);
}

#[test]
fn first_render_builds_with_zero_generation() {
    let mut state = CpuTableState::default();
    let rows = vec![raw(10, "alpha.exe", 5.0, 1024)];
    assert!(state.rebuild_if_needed(0, &rows));
    let cached = state.cached_rows().expect("rows cached");
    assert_eq!(cached.len(), 1);
    assert!((cached[0].cpu_pct - 5.0).abs() < f32::EPSILON);
}

#[test]
fn clone_group_sums_cpu_and_ram_once() {
    let mut state = CpuTableState::default();
    let rows = vec![
        raw(1, "chrome.exe", 2.0, 100),
        raw(2, "chrome.exe", 3.0, 200),
        raw(3, "notepad.exe", 1.0, 50),
    ];
    state.rebuild_if_needed(1, &rows);
    let cached = state.cached_rows().expect("rows cached");
    assert_eq!(cached.len(), 2);

    let chrome = cached.iter().find(|item| item.name.as_ref() == "chrome.exe").expect("chrome row");
    assert_eq!(chrome.clone_count, 2);
    assert!((chrome.cpu_pct - 5.0).abs() < f32::EPSILON);
    assert_eq!(chrome.ram_bytes, 300);
}

#[test]
fn bottleneck_true_at_85_total() {
    let snap = CpuSnapshot { total_pct: 85.0, hottest_core_pct: 10.0, ..Default::default() };
    assert!(snap.single_thread_bottleneck());
}

#[test]
fn bottleneck_true_at_85_hottest() {
    let snap = CpuSnapshot { total_pct: 10.0, hottest_core_pct: 85.0, ..Default::default() };
    assert!(snap.single_thread_bottleneck());
}

#[test]
fn bottleneck_false_below_85() {
    let snap = CpuSnapshot { total_pct: 84.9, hottest_core_pct: 84.9, ..Default::default() };
    assert!(!snap.single_thread_bottleneck());
}

#[test]
fn clone_group_is_case_insensitive() {
    let mut state = CpuTableState::default();
    let rows = vec![raw(1, "Chrome.exe", 2.0, 100), raw(2, "chrome.exe", 3.0, 200)];
    state.rebuild_if_needed(1, &rows);
    let cached = state.cached_rows().expect("rows cached");
    assert_eq!(cached.len(), 1);
    assert_eq!(cached[0].clone_count, 2);
}

#[test]
fn clone_group_sums_delta_once() {
    let mut state = CpuTableState::default();
    let rows = vec![raw(1, "a.exe", 1.0, 100), raw(2, "a.exe", 2.0, 200)];
    state.rebuild_if_needed(1, &rows);
    let cached = state.cached_rows().expect("rows cached");
    assert_eq!(cached[0].ram_delta_10s, 0);
}

#[test]
fn sort_by_ram_desc_then_toggle() {
    let mut state = CpuTableState::default();
    let rows = vec![raw(1, "a.exe", 1.0, 100), raw(2, "b.exe", 2.0, 900)];
    state.rebuild_if_needed(1, &rows);
    state.set_sort(CpuSortCol::RamBytes);
    let cached = state.cached_rows().expect("rows cached");
    assert_eq!(cached[0].ram_bytes, 900);
    assert!(!state.sort_asc);
}

#[test]
fn sort_switch_name_sets_asc_true() {
    let mut state = CpuTableState::default();
    state.set_sort(CpuSortCol::Name);
    assert_eq!(state.sort_col, CpuSortCol::Name);
    assert!(state.sort_asc);
}

#[test]
fn sort_toggle_same_col_flips() {
    let mut state = CpuTableState::default();
    state.set_sort(CpuSortCol::Name);
    assert!(state.sort_asc);
    state.set_sort(CpuSortCol::Name);
    assert!(!state.sort_asc);
}

#[test]
fn sort_by_pid_asc_after_switch() {
    let mut state = CpuTableState::default();
    let rows = vec![raw(30, "c.exe", 1.0, 10), raw(10, "a.exe", 1.0, 10)];
    state.rebuild_if_needed(1, &rows);
    state.set_sort(CpuSortCol::Pid);
    let cached = state.cached_rows().expect("rows cached");
    assert_eq!(cached[0].pid, 10);
    assert_eq!(cached[1].pid, 30);
}

#[test]
fn search_by_pid_substring_zero_alloc() {
    let mut state = CpuTableState::default();
    let rows = vec![raw(123, "a.exe", 1.0, 10), raw(45, "b.exe", 1.0, 10)];
    state.search_query = "12".to_owned();
    state.rebuild_if_needed(1, &rows);
    let cached = state.cached_rows().expect("rows cached");
    assert_eq!(cached.len(), 1);
    assert_eq!(cached[0].pid, 123);
}

#[test]
fn search_by_name_case_insensitive() {
    let mut state = CpuTableState::default();
    let rows = vec![raw(1, "Chrome.exe", 1.0, 10), raw(2, "other.exe", 1.0, 10)];
    state.search_query = "CHROME".to_owned();
    state.rebuild_if_needed(1, &rows);
    let cached = state.cached_rows().expect("rows cached");
    assert_eq!(cached.len(), 1);
    assert_eq!(cached[0].name.as_ref(), "Chrome.exe");
}

#[test]
fn pid_contains_edges() {
    assert!(pid_contains(123, "12"));
    assert!(!pid_contains(12, "123"));
    assert!(pid_contains(0, "0"));
    assert!(!pid_contains(100, "9"));
    assert!(!pid_contains(99999, ""));
}

#[test]
fn empty_filter_result_caches_without_rebuild_storm() {
    let mut state =
        CpuTableState { search_query: "zzzz_no_match".to_owned(), ..Default::default() };
    let rows = vec![raw(1, "a.exe", 1.0, 10)];
    assert!(state.rebuild_if_needed(1, &rows));
    let cached = state.cached_rows().expect("rows cached");
    assert!(cached.is_empty());
    assert!(!state.rebuild_if_needed(1, &rows));
}

#[test]
fn normalized_search_key_caches_whitespace_and_case() {
    let mut state = CpuTableState::default();
    let rows = vec![raw(1, "Chrome.exe", 1.0, 10), raw(2, "other.exe", 1.0, 10)];
    state.search_query = "Chrome".to_owned();
    assert!(state.rebuild_if_needed(1, &rows));
    state.search_query = " chrome ".to_owned();
    state.mark_dirty();
    assert!(!state.rebuild_if_needed(1, &rows));
}

#[test]
fn ensure_sort_keeps_direction() {
    let mut state = CpuTableState::default();
    state.ensure_sort(CpuSortCol::CpuPct, false);
    assert!(!state.sort_asc);
    state.ensure_sort(CpuSortCol::CpuPct, false);
    assert!(!state.sort_asc);
    state.ensure_sort(CpuSortCol::CpuPct, true);
    assert!(state.sort_asc);
}

#[test]
fn pid_reuse_clears_selection() {
    let mut state = CpuTableState {
        selected_pid: Some(100),
        selected_start_time: Some(10),
        selected_name: Some("app.exe".to_owned()),
        ..Default::default()
    };
    let rows = vec![raw_st(100, "app.exe", 1.0, 10, 99)];
    assert!(state.rebuild_if_needed(1, &rows));
    assert_eq!(state.selected_pid, None);
    assert_eq!(state.selected_start_time, None);
}

#[test]
fn disconnected_action_channel_recovers() {
    let mut state = CpuTableState::default();
    let (tx, rx) = std::sync::mpsc::channel();
    drop(tx);
    state.action_rx = Some(rx);
    state.poll_menu_action();
    assert!(state.action_rx.is_none());
    let msg = state.last_action_msg.as_ref().expect("msg set");
    assert!(msg.0.contains("disconnected"));
}

#[test]
fn context_menu_kill_arms_confirmation_without_spawning() {
    use crate::modules::cpu::model::ProcessCpuItem;
    use crate::modules::cpu::table_menu::arm_context_menu_kill;
    let mut state = CpuTableState::default();
    let mut item = ProcessCpuItem::new(42, Arc::from("app.exe"), 1.0, 100, None);
    item.start_time = 7;
    arm_context_menu_kill(&mut state, &item);
    assert_eq!(state.selected_pid, Some(42));
    assert_eq!(state.selected_start_time, Some(7));
    assert!(state.confirm_kill.is_some());
    assert!(state.action_rx.is_none());
}

#[test]
fn resolve_group_targets_collects_all_clones() {
    use crate::modules::cpu::model::ProcessCpuItem;
    use crate::modules::cpu::table_menu::resolve_group_targets;
    let mut item = ProcessCpuItem::new(10, Arc::from("chrome.exe"), 1.0, 100, None);
    item.start_time = 1;
    item.clone_count = 3;
    let rows = vec![
        raw_st(10, "chrome.exe", 1.0, 100, 1),
        raw_st(11, "Chrome.exe", 1.0, 100, 2),
        raw_st(12, "chrome.exe", 1.0, 100, 3),
        raw_st(13, "other.exe", 1.0, 100, 4),
    ];
    let grouped = resolve_group_targets(&item, &rows, true);
    assert_eq!(grouped.len(), 3);
    let mut pids: Vec<_> = grouped.iter().map(|(pid, _, _, _)| *pid).collect();
    pids.sort_unstable();
    assert_eq!(pids, vec![10, 11, 12]);
    let single = resolve_group_targets(&item, &rows, false);
    assert_eq!(single.len(), 1);
    assert_eq!(single[0].0, 10);
}

#[test]
fn grouped_idle_clones_not_cpu_hog_but_hot_clone_is() {
    let mut state = CpuTableState::default();
    let rows: Vec<RawProcessCpuRow> = (0..20).map(|i| raw(100 + i, "clone.exe", 1.5, 10)).collect();
    state.rebuild_if_needed(1, &rows);
    let cached = state.cached_rows().expect("rows cached");
    assert_eq!(cached.len(), 1);
    assert!((cached[0].cpu_pct - 30.0).abs() < 0.001);
    assert_eq!(cached[0].clone_count, 20);
    assert!(!cached[0].is_cpu_hog);
    let mut state2 = CpuTableState::default();
    let rows2 = vec![raw(1, "hot.exe", 1.0, 10), raw(2, "hot.exe", 26.0, 10)];
    state2.rebuild_if_needed(1, &rows2);
    let cached2 = state2.cached_rows().expect("rows cached");
    assert_eq!(cached2.len(), 1);
    assert!(cached2[0].is_cpu_hog);
}
