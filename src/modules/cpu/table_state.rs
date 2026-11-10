use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use crate::modules::cpu::model::{ProcessCpuItem, RawProcessCpuRow};
use crate::modules::cpu::table_sort::{filter_group_and_sort, sort_slice};
pub(crate) type ActionMsg = (String, Option<(u32, u64, bool)>, Vec<((u32, u64), Option<bool>)>);
pub use super::quick_filter::ProcessQuickFilter;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CpuSortCol {
    #[default]
    CpuPct,
    Name,
    Pid,
    RamBytes,
    CloneCount,
    Delta,
    DiskIo,
}
#[derive(Debug, Clone, PartialEq, Eq)]
struct CpuTableCacheKey(u64, String, CpuSortCol, bool, bool, u8);
#[derive(Debug)]
pub struct CpuTableState {
    pub comfort_density: bool,
    pub search_query: String,
    pub focus_search_next_frame: bool,
    pub quick_filter: ProcessQuickFilter,
    pub group_clones: bool,
    pub sort_col: CpuSortCol,
    pub sort_asc: bool,
    pub selected_pid: Option<u32>,
    pub selected_start_time: Option<u64>,
    pub selected_name: Option<String>,
    pub paused: bool,
    pub last_action_msg: Option<(String, Instant)>,
    pub action_rx: Option<std::sync::mpsc::Receiver<ActionMsg>>,
    pub eco_overrides: HashMap<(u32, u64), bool>,
    pub confirm_kill: Option<(u32, Instant)>,
    pub baseline_scan_gen: u64,
    dirty: bool,
    cached_rows: Option<Arc<[ProcessCpuItem]>>,
    cache_key: Option<CpuTableCacheKey>,
    cached_scan_generation: u64,
    ram_baseline: HashMap<(u32, u64), (u64, Instant, i64)>,
}

impl Default for CpuTableState {
    fn default() -> Self {
        Self {
            comfort_density: false,
            search_query: String::new(),
            focus_search_next_frame: false,
            quick_filter: ProcessQuickFilter::All,
            group_clones: true,
            sort_col: CpuSortCol::CpuPct,
            sort_asc: false,
            selected_pid: None,
            selected_start_time: None,
            selected_name: None,
            paused: false,
            last_action_msg: None,
            action_rx: None,
            eco_overrides: HashMap::new(),
            confirm_kill: None,
            dirty: false,
            cached_rows: None,
            cache_key: None,
            cached_scan_generation: 0,
            baseline_scan_gen: 0,
            ram_baseline: HashMap::new(),
        }
    }
}
impl CpuTableState {
    pub fn rebuild_if_needed(
        &mut self,
        scan_generation: u64,
        raw_rows: &[RawProcessCpuRow],
    ) -> bool {
        if self.paused {
            return false;
        }
        let first_build = self.cached_rows.is_none();
        if raw_rows.is_empty() && !first_build && scan_generation == 0 {
            return false;
        }
        let query_lower = self.search_query.trim().to_ascii_lowercase();
        let new_key = CpuTableCacheKey(
            scan_generation,
            query_lower.clone(),
            self.sort_col,
            self.sort_asc,
            self.group_clones,
            self.quick_filter.to_u8(),
        );
        if !first_build
            && !self.dirty
            && scan_generation != 0
            && self.cached_scan_generation == scan_generation
        {
            return false;
        }
        if !first_build && !self.dirty && self.cache_key.as_ref() == Some(&new_key) {
            return false;
        }
        if !first_build && self.cache_key.as_ref() == Some(&new_key) {
            self.dirty = false;
            return false;
        }
        let scan_changed = first_build
            || self.baseline_scan_gen != scan_generation
            || self.ram_baseline.is_empty();
        let ram_deltas =
            self.compute_ram_deltas(raw_rows, Instant::now(), scan_changed, scan_generation);
        let rows: Arc<[ProcessCpuItem]> = filter_group_and_sort(
            raw_rows,
            &ram_deltas,
            &query_lower,
            self.quick_filter,
            self.group_clones,
            self.sort_col,
            self.sort_asc,
            &self.eco_overrides,
        )
        .into();
        self.preserve_selection(&rows);
        self.cached_rows = Some(Arc::clone(&rows));
        self.cache_key = Some(new_key);
        self.cached_scan_generation = scan_generation;
        self.dirty = raw_rows.is_empty() && first_build;
        true
    }

    fn preserve_selection(&mut self, rows: &[ProcessCpuItem]) {
        if let Some(pid) = self.selected_pid {
            if let Some(found) = rows.iter().find(|r| r.pid == pid) {
                if self.selected_start_time.is_some_and(|st| st != found.start_time) {
                    self.selected_pid = None;
                    self.selected_start_time = None;
                    self.selected_name = None;
                    self.confirm_kill = None;
                    return;
                }
                self.selected_pid = Some(found.pid);
                self.selected_start_time = Some(found.start_time);
                self.selected_name = Some(found.name.to_string());
                return;
            }
        }
        if self.group_clones {
            if let Some(found) = self
                .selected_name
                .as_ref()
                .and_then(|n| rows.iter().find(|r| r.name.eq_ignore_ascii_case(n)))
            {
                self.selected_pid = Some(found.pid);
                self.selected_start_time = Some(found.start_time);
                self.selected_name = Some(found.name.to_string());
                return;
            }
        }
        self.selected_pid = None;
        self.selected_start_time = None;
        self.selected_name = None;
    }
    pub fn cached_rows(&self) -> Option<Arc<[ProcessCpuItem]>> {
        self.cached_rows.clone()
    }
    pub fn set_sort(&mut self, col: CpuSortCol) {
        if self.sort_col == col {
            self.sort_asc = !self.sort_asc;
        } else {
            self.sort_col = col;
            self.sort_asc = matches!(col, CpuSortCol::Name | CpuSortCol::Pid);
        }
        self.apply_sort_in_place();
    }
    pub fn ensure_sort(&mut self, col: CpuSortCol, asc: bool) {
        self.sort_col = col;
        self.sort_asc = asc;
        self.apply_sort_in_place();
    }
    pub fn apply_sort_in_place(&mut self) {
        if let Some(rows) = &mut self.cached_rows {
            sort_slice(Arc::make_mut(rows), self.sort_col, self.sort_asc);
        }
        if let Some(key) = &mut self.cache_key {
            key.2 = self.sort_col;
            key.3 = self.sort_asc;
        }
    }
    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
        if !self.paused {
            self.ram_baseline.clear();
            self.dirty = true;
        }
    }
    pub fn set_group_clones(&mut self, value: bool) {
        if self.group_clones != value {
            self.group_clones = value;
            self.dirty = true;
        }
    }
    pub fn clear_search(&mut self) {
        if !self.search_query.is_empty() {
            self.search_query.clear();
            self.dirty = true;
        }
    }
    pub fn set_quick_filter(&mut self, f: ProcessQuickFilter) {
        if self.quick_filter != f {
            self.quick_filter = f;
            self.dirty = true;
        }
    }
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }
    pub(crate) fn apply_eco_rollbacks(&mut self, rollbacks: &[((u32, u64), Option<bool>)]) {
        if rollbacks.is_empty() {
            return;
        }
        for &(key, prev) in rollbacks {
            if let Some(old) = prev {
                if self.eco_overrides.len() >= 512 && !self.eco_overrides.contains_key(&key) {
                    if let Some(k) = self.eco_overrides.keys().next().copied() {
                        self.eco_overrides.remove(&k);
                    }
                }
                self.eco_overrides.insert(key, old);
            } else {
                self.eco_overrides.remove(&key);
            }
        }
        self.dirty = true;
    }
}

#[path = "table_state_deltas.rs"]
mod table_state_deltas;

#[cfg(test)]
#[path = "table_state_tests.rs"]
mod table_state_tests;
