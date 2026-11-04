//! Unit tests for table view (process table with 1 Hz cache).
#[cfg(test)]
mod tests {
    use crate::modules::memory::model::{MemorySnapshot, RawProcessRow};
    use crate::modules::memory::table_state::MemoryTableState;
    use std::sync::Arc;
    fn r(p: u32, n: &str, rb: u64, vb: u64, d: i64) -> RawProcessRow {
        RawProcessRow {
            pid: p,
            name: n.into(),
            ram_bytes: rb,
            virtual_bytes: vb,
            delta_bytes_per_sec: d,
            exe_path: None,
            creation_time: None,
        }
    }
    fn sh(x: Vec<RawProcessRow>) -> Arc<Vec<RawProcessRow>> {
        Arc::new(x)
    }
    fn sp(p: Arc<Vec<RawProcessRow>>) -> MemorySnapshot {
        MemorySnapshot {
            ram_total_bytes: 16_000_000_000,
            processes: p,
            latest_spike: None,
            kernel_metrics_valid: true,
            ..Default::default()
        }
    }
    #[test]
    fn t_defaults() {
        let s = MemoryTableState::default();
        assert!(s.search_query.is_empty());
        assert!(s.group_clones);
        assert_eq!(s.sort_col, crate::modules::memory::process_model::MemorySortCol::RamBytes);
        assert!(!s.sort_asc);
        assert!(s.selected_pid.is_none());
    }
    #[test]
    fn t_render() {
        let s = sp(sh(vec![r(100, "c", 1, 2, 0), r(200, "g", 3, 4, 0)]));
        let mut st = MemoryTableState::default();
        egui::__run_test_ui(|ui| {
            crate::modules::memory::table_view::render_memory_table(ui, &s, &mut st, 0)
        });
    }
    #[test]
    fn t_sort() {
        let s = sp(sh(vec![r(100, "c", 1, 2, 0), r(200, "g", 3, 4, 0)]));
        let mut st = MemoryTableState::default();
        egui::__run_test_ui(|ui| {
            crate::modules::memory::table_view::render_memory_table(ui, &s, &mut st, 0)
        });
        assert!(st.has_cached_rows());
        st.sort_col = crate::modules::memory::process_model::MemorySortCol::Name;
        egui::__run_test_ui(|ui| {
            crate::modules::memory::table_view::render_memory_table(ui, &s, &mut st, 0)
        });
        assert!(st.has_cached_rows());
    }
    #[test]
    fn t_delta() {
        let mut st = MemoryTableState::default();
        let mut s1 = sp(sh(vec![r(100, "c", 1, 2, 0), r(200, "g", 3, 4, 0)]));
        s1.ram_used_bytes = 8_000_000_000;
        s1.ram_available_bytes = 8_000_000_000;
        s1.swap_used_bytes = 1_000_000_000;
        st.rebuild_for_scan(1, &s1.processes, s1.ram_total_bytes);
        assert!(st.has_cached_rows());
        assert_eq!(st.cached_rows().unwrap().len(), 2);
        let mut s2 = sp(sh(vec![r(100, "c", 1, 2, 50_000_000), r(200, "g", 3, 4, 0)]));
        s2.ram_used_bytes = 8_500_000_000;
        s2.ram_available_bytes = 7_500_000_000;
        s2.swap_used_bytes = 1_000_000_000;
        s2.process_scan_id = 2;
        st.rebuild_for_scan(2, &s2.processes, s2.ram_total_bytes);
        assert_eq!(st.cached_rows().unwrap().len(), 2);
        assert_eq!(
            st.cached_rows().unwrap().iter().find(|r| r.name == "c").unwrap().delta_bytes_per_sec,
            50_000_000
        );
    }
    #[test]
    fn t_scan_id() {
        let mut st = MemoryTableState::default();
        let r = vec![r(100, "c", 1, 2, 0), r(200, "g", 3, 4, 0)];
        st.rebuild_for_scan(1, &r, 16_000_000_000);
        assert_eq!(st.cached_scan_id(), 1);
        assert!(st.cached_rows().is_some());
        let b = st.cached_rows();
        st.rebuild_for_scan(1, &r, 16_000_000_000);
        assert_eq!(st.cached_scan_id(), 1);
        assert!(Arc::ptr_eq(b.as_ref().unwrap(), st.cached_rows().as_ref().unwrap()));
        st.rebuild_for_scan(2, &r, 16_000_000_000);
        assert_eq!(st.cached_scan_id(), 2);
    }
    #[test]
    fn t_sel_cache() {
        let mut st = MemoryTableState::default();
        let r = vec![r(100, "c [2x]", 1, 2, 0), r(101, "c", 500_000_000, 1_000_000_000, 0)];
        st.selected_group = Some("c".into());
        let s1 = st.resolved_selection(1, &r);
        assert!(s1.is_some());
        let p = s1.as_ref().unwrap().pids.clone();
        assert_eq!(p, vec![100, 101]);
        let s2 = st.resolved_selection(1, &r);
        assert!(s2.is_some());
        assert!(Arc::ptr_eq(s1.as_ref().unwrap(), s2.as_ref().unwrap()));
        assert_eq!(s2.as_ref().unwrap().pids, p);
        let s3 = st.resolved_selection(2, &r);
        assert!(s3.is_some());
        assert_eq!(s3.as_ref().unwrap().pids, p);
    }
    #[test]
    fn t_fallback() {
        let mut st = MemoryTableState::default();
        let r = vec![r(100, "c", 1, 2, 0)];
        st.rebuild_if_needed(&r);
        assert!(st.cached_rows().is_some());
    }
}
