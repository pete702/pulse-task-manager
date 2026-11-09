//! Unit tests for the CPU table cache and clone grouping.

use super::*;

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
fn defaults_first_and_group() {
    let mut state = CpuTableState::default();
    assert!(state.group_clones);
    assert_eq!(state.sort_col, CpuSortCol::CpuPct);
    assert!(!state.sort_asc);
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
fn clone_group_variants() {
    let mut state = CpuTableState::default();
    let rows = vec![raw(1, "Chrome.exe", 2.0, 100), raw(2, "chrome.exe", 3.0, 200)];
    state.rebuild_if_needed(1, &rows);
    let cached = state.cached_rows().expect("rows cached");
    assert_eq!(cached.len(), 1);
    assert_eq!(cached[0].clone_count, 2);
    assert_eq!(cached[0].ram_delta_10s, 0);
}

#[test]
fn sorts() {
    let mut state = CpuTableState::default();
    let rows = vec![raw(1, "a.exe", 1.0, 100), raw(2, "b.exe", 2.0, 900)];
    state.rebuild_if_needed(1, &rows);
    state.set_sort(CpuSortCol::RamBytes);
    let cached = state.cached_rows().expect("rows cached");
    assert_eq!(cached[0].ram_bytes, 900);
    assert!(!state.sort_asc);
    let mut s2 = CpuTableState::default();
    s2.set_sort(CpuSortCol::Name);
    assert_eq!(s2.sort_col, CpuSortCol::Name);
    assert!(s2.sort_asc);
    s2.set_sort(CpuSortCol::Name);
    assert!(!s2.sort_asc);
    let mut s3 = CpuTableState::default();
    let rows3 = vec![raw(30, "c.exe", 1.0, 10), raw(10, "a.exe", 1.0, 10)];
    s3.rebuild_if_needed(1, &rows3);
    s3.set_sort(CpuSortCol::Pid);
    let cached3 = s3.cached_rows().expect("rows cached");
    assert_eq!(cached3[0].pid, 10);
    assert_eq!(cached3[1].pid, 30);
}

#[test]
fn search_filters() {
    let mut state = CpuTableState::default();
    let rows = vec![raw(123, "a.exe", 1.0, 10), raw(45, "b.exe", 1.0, 10)];
    state.search_query = "12".to_owned();
    state.rebuild_if_needed(1, &rows);
    let cached = state.cached_rows().expect("rows cached");
    assert_eq!(cached.len(), 1);
    assert_eq!(cached[0].pid, 123);
    let mut s2 = CpuTableState::default();
    let rows2 = vec![raw(1, "Chrome.exe", 1.0, 10), raw(2, "other.exe", 1.0, 10)];
    s2.search_query = "CHROME".to_owned();
    s2.rebuild_if_needed(1, &rows2);
    let cached2 = s2.cached_rows().expect("rows cached");
    assert_eq!(cached2.len(), 1);
    assert_eq!(cached2[0].name.as_ref(), "Chrome.exe");
}

#[test]
fn search_cache_no_storm() {
    let mut state =
        CpuTableState { search_query: "zzzz_no_match".to_owned(), ..Default::default() };
    let rows = vec![raw(1, "a.exe", 1.0, 10)];
    assert!(state.rebuild_if_needed(1, &rows));
    assert!(state.cached_rows().expect("rows cached").is_empty());
    assert!(!state.rebuild_if_needed(1, &rows));
    let mut s2 = CpuTableState::default();
    let rows2 = vec![raw(1, "Chrome.exe", 1.0, 10), raw(2, "other.exe", 1.0, 10)];
    s2.search_query = "Chrome".to_owned();
    assert!(s2.rebuild_if_needed(1, &rows2));
    s2.search_query = " chrome ".to_owned();
    s2.mark_dirty();
    assert!(!s2.rebuild_if_needed(1, &rows2));
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
    let mut r10 = raw_st(10, "chrome.exe", 1.0, 100, 1);
    r10.creation_time = Some(100);
    let mut r11 = raw_st(11, "Chrome.exe", 1.0, 100, 2);
    r11.creation_time = Some(200);
    r11.exe_path = Some(Arc::from("C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe"));
    let mut r12 = raw_st(12, "chrome.exe", 1.0, 100, 3);
    r12.creation_time = Some(300);
    let rows = vec![r10, r11, r12, raw_st(13, "other.exe", 1.0, 100, 4)];
    let grouped = resolve_group_targets(&item, true, &rows);
    assert_eq!(grouped.len(), 3);
    let mut pids: Vec<_> = grouped.iter().map(|t| t.pid).collect();
    pids.sort_unstable();
    assert_eq!(pids, vec![10, 11, 12]);
    for t in &grouped {
        assert_eq!(t.start_time, t.creation_time.unwrap_or(0) / 100);
    }
    assert!(grouped.iter().any(|t| t.pid == 11
        && t.exe_path.as_deref()
            == Some("C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe")));
    let single = resolve_group_targets(&item, false, &rows);
    assert_eq!(single.len(), 1);
    assert_eq!(single[0].pid, 10);
    assert_eq!(single[0].start_time, 1);
    assert_eq!(single[0].creation_time, Some(100));
}
#[test]
fn resolve_excludes_same_name_different_exe() {
    use crate::modules::cpu::model::ProcessCpuItem;
    use crate::modules::cpu::table_menu::resolve_group_targets;
    let mut item = ProcessCpuItem::new(10, Arc::from("chrome.exe"), 1.0, 100, None);
    item.start_time = 1;
    item.clone_count = 2;
    item.exe_path = Arc::from("C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe");
    let mut legit = raw_st(10, "chrome.exe", 1.0, 100, 1);
    legit.exe_path = Some(Arc::from("C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe"));
    legit.creation_time = Some(10);
    let mut impostor = raw_st(99, "chrome.exe", 1.0, 100, 2);
    impostor.exe_path = Some(Arc::from("C:\\Temp\\chrome.exe"));
    impostor.creation_time = Some(20);
    let rows = vec![legit, impostor];
    let v = resolve_group_targets(&item, true, &rows);
    assert_eq!(v.len(), 1);
    assert_eq!(v[0].pid, 10);
    assert_eq!(v[0].creation_time, Some(10));
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
#[test]
fn dead_entries_pruned_from_ram_baseline() {
    let mut state = CpuTableState::default();
    let rows = vec![raw(1, "a.exe", 1.0, 100), raw(2, "b.exe", 1.0, 100)];
    let now = std::time::Instant::now();
    let _ = state.compute_ram_deltas(&rows, now, true, 1);
    assert!(state.ram_baseline.contains_key(&(1, 0)));
    let rows2 = vec![raw(1, "a.exe", 1.0, 100)];
    let _ = state.compute_ram_deltas(&rows2, now, false, 1);
    assert!(state.ram_baseline.contains_key(&(1, 0)));
    for i in 0..40 {
        state.ram_baseline.insert((1000 + i, 0), (10, now, 0));
    }
    let _ = state.compute_ram_deltas(&rows2, now, false, 1);
    assert!(!state.ram_baseline.contains_key(&(1000, 0)));
    assert!(!state.ram_baseline.contains_key(&(2, 0)));
    let empty: Vec<RawProcessCpuRow> = vec![];
    let _ = state.compute_ram_deltas(&empty, now, true, 2);
    assert!(state.ram_baseline.is_empty());
}
#[test]
fn eco_rollback_restores_prev_and_invalidates() {
    let mut state = CpuTableState::default();
    state.eco_overrides.insert((1, 0), true);
    let (tx, rx) = std::sync::mpsc::channel();
    tx.send(("blocked".to_owned(), None, vec![((1, 0), None), ((2, 0), Some(false))])).unwrap();
    state.action_rx = Some(rx);
    state.poll_menu_action();
    assert!(!state.eco_overrides.contains_key(&(1, 0)));
    assert_eq!(state.eco_overrides.get(&(2, 0)), Some(&false));
    assert!(state.dirty);
}
