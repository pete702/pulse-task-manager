//! Selection cache and PID-reuse tests for the memory process table.

#[cfg(test)]
mod tests {
    use crate::modules::memory::model::RawProcessRow;
    use crate::modules::memory::table_state::MemoryTableState;

    fn rc(p: u32, n: &str, c: u64) -> RawProcessRow {
        RawProcessRow {
            pid: p,
            name: n.into(),
            ram_bytes: 1_000_000_000,
            virtual_bytes: 2_000_000_000,
            delta_bytes_per_sec: 0,
            exe_path: None,
            creation_time: Some(c),
        }
    }

    #[test]
    fn t_pid_id() {
        let mut st = MemoryTableState::default();
        st.group_clones = false;
        let r = vec![RawProcessRow { pid: 100, name: "c".into(), ..Default::default() }];
        st.selected_group = Some("c".into());
        st.selected_pid = Some(100);
        let s = st.resolved_selection(1, &r);
        assert!(s.is_some());
        assert_eq!(st.selected_creation_time, None);
        assert_eq!(st.selected_name, Some("c".into()));
    }

    #[test]
    fn t_pid_reuse() {
        let mut st = MemoryTableState::default();
        st.group_clones = false;
        let r1 = vec![rc(100, "c", 1000)];
        st.selected_group = Some("c".into());
        st.selected_pid = Some(100);
        let s1 = st.resolved_selection(1, &r1);
        assert!(s1.is_some());
        assert_eq!(st.selected_creation_time, Some(1000));
        assert_eq!(st.selected_name, Some("c".into()));
        let r2 = vec![rc(100, "n", 2000)];
        let s2 = st.resolved_selection(2, &r2);
        assert!(s2.is_none());
        assert!(st.selected_pid.is_none());
        assert!(st.selected_group.is_none());
    }

    #[test]
    fn t_pid_reuse_ct() {
        let mut st = MemoryTableState::default();
        st.group_clones = false;
        let r1 = vec![rc(100, "c", 1000)];
        st.selected_group = Some("c".into());
        st.selected_pid = Some(100);
        let s1 = st.resolved_selection(1, &r1);
        assert!(s1.is_some());
        assert_eq!(st.selected_creation_time, Some(1000));
        let r2 = vec![rc(100, "c", 2000)];
        let s2 = st.resolved_selection(2, &r2);
        assert!(s2.is_none());
        assert!(st.selected_pid.is_none());
    }

    #[test]
    fn t_pid_reuse_name() {
        let mut st = MemoryTableState::default();
        st.group_clones = false;
        let r1 = vec![rc(100, "c", 1000)];
        st.selected_group = Some("c".into());
        st.selected_pid = Some(100);
        let s1 = st.resolved_selection(1, &r1);
        assert!(s1.is_some());
        assert_eq!(st.selected_name, Some("c".into()));
        let r2 = vec![rc(100, "n", 1000)];
        let s2 = st.resolved_selection(2, &r2);
        assert!(s2.is_none());
        assert!(st.selected_pid.is_none());
    }

    #[test]
    fn t_pid_gone() {
        let mut st = MemoryTableState::default();
        st.group_clones = false;
        let r1 = vec![rc(100, "c", 1000)];
        st.selected_group = Some("c".into());
        st.selected_pid = Some(100);
        let s1 = st.resolved_selection(1, &r1);
        assert!(s1.is_some());
        let r2: Vec<RawProcessRow> = vec![];
        let s2 = st.resolved_selection(2, &r2);
        assert!(s2.is_none());
        assert!(st.selected_pid.is_none());
    }

    #[test]
    fn t_grouped() {
        let mut st = MemoryTableState::default();
        let r1 = vec![rc(100, "c", 1000)];
        st.selected_group = Some("c".into());
        let s1 = st.resolved_selection(1, &r1);
        assert!(s1.is_some());
        let r2 = vec![rc(100, "c", 2000)];
        let s2 = st.resolved_selection(2, &r2);
        assert!(s2.is_some());
        assert_eq!(s2.as_ref().unwrap().pids, vec![100]);
    }
}
