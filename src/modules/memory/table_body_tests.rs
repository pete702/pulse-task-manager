//! Unit tests for table body (virtualized process table rendering).

#[cfg(test)]
mod tests {
    use crate::modules::memory::process_model::MemorySortCol;
    use crate::modules::memory::table_sort::toggle_sort;
    use crate::modules::memory::table_state::MemoryTableState;

    #[test]
    fn test_toggle_sort() {
        let mut s = MemoryTableState::default();
        toggle_sort(&mut s, MemorySortCol::RamBytes);
        assert!(s.sort_asc);
        toggle_sort(&mut s, MemorySortCol::Name);
        assert_eq!(s.sort_col, MemorySortCol::Name);
        assert!(s.sort_asc);
        toggle_sort(&mut s, MemorySortCol::Name);
        assert!(!s.sort_asc);
    }
}
