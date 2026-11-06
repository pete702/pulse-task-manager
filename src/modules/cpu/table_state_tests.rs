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
        exe_path: None,
        creation_time: None,
        start_time: 0,
    }
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
    assert!(pid_contains(99999, ""));
}
