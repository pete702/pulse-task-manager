//! Unit tests for the CPU table cache and clone grouping.

use super::*;

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
