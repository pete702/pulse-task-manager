use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use crate::modules::cpu::model::{ProcessCpuItem, RawProcessCpuRow};
use crate::modules::cpu::table_sort::{filter_group_and_sort, sort_slice};
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ProcessQuickFilter {
    #[default]
    All,
    ActiveLoad,
    HighRam,
    EcoQos,
}

impl ProcessQuickFilter {
    pub fn from_u8(v: u8) -> Self {
        match v {
            1 => ProcessQuickFilter::ActiveLoad,
            2 => ProcessQuickFilter::HighRam,
            3 => ProcessQuickFilter::EcoQos,
            _ => ProcessQuickFilter::All,
        }
    }
    pub fn to_u8(self) -> u8 {
        match self {
            ProcessQuickFilter::All => 0,
            ProcessQuickFilter::ActiveLoad => 1,
            ProcessQuickFilter::HighRam => 2,
            ProcessQuickFilter::EcoQos => 3,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            ProcessQuickFilter::All => "[ All ]",
            ProcessQuickFilter::ActiveLoad => "[ Active CPU/Disk ]",
            ProcessQuickFilter::HighRam => "[ RAM > 200 MB ]",
            ProcessQuickFilter::EcoQos => "[ EcoQoS ]",
        }
    }
    pub fn matches(
        self,
        cpu_pct: f32,
        ram_bytes: u64,
        io_mbps: f32,
        eco_qos: Option<bool>,
    ) -> bool {
        match self {
            ProcessQuickFilter::All => true,
            ProcessQuickFilter::ActiveLoad => cpu_pct >= 1.0 || io_mbps > 0.0,
            ProcessQuickFilter::HighRam => ram_bytes >= 200 * 1024 * 1024,
            ProcessQuickFilter::EcoQos => eco_qos == Some(true),
        }
    }
}

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
#[derive(Debug, Clone)]
pub struct CpuTableState {
    pub comfort_density: bool,
    pub search_query: String,
    pub focus_search_next_frame: bool,
    pub quick_filter: ProcessQuickFilter,
    pub group_clones: bool,
    pub sort_col: CpuSortCol,
    pub sort_asc: bool,
    pub selected_pid: Option<u32>,
    pub selected_name: Option<String>,
    pub paused: bool,
    pub last_action_msg: String,
    pub confirm_kill: Option<(u32, Instant)>,
    dirty: bool,
    cached_rows: Option<Arc<[ProcessCpuItem]>>,
    cache_key: Option<CpuTableCacheKey>,
    cached_scan_generation: u64,
    ram_baseline: HashMap<(u32, u64), (u64, Instant)>,
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
            selected_name: None,
            paused: false,
            last_action_msg: String::new(),
            confirm_kill: None,
            dirty: false,
            cached_rows: None,
            cache_key: None,
            cached_scan_generation: 0,
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
        let cached_empty = self.cached_rows.as_ref().is_some_and(|r| r.is_empty());
        let needs_data = cached_empty && !raw_rows.is_empty();
        if raw_rows.is_empty() && !first_build {
            return false;
        }
        if !first_build
            && !needs_data
            && scan_generation != 0
            && self.cached_scan_generation == scan_generation
            && !self.dirty
        {
            return false;
        }
        let new_key = CpuTableCacheKey(
            scan_generation,
            self.search_query.clone(),
            self.sort_col,
            self.sort_asc,
            self.group_clones,
            self.quick_filter.to_u8(),
        );
        if !first_build && !needs_data && self.cache_key.as_ref() == Some(&new_key) {
            self.cached_scan_generation = scan_generation;
            self.dirty = false;
            return false;
        }
        let query_lower = self.search_query.trim().to_ascii_lowercase();
        let deltas = self.compute_ram_deltas(raw_rows);
        let rows: Arc<[ProcessCpuItem]> = filter_group_and_sort(
            raw_rows,
            &query_lower,
            self.quick_filter,
            self.group_clones,
            self.sort_col,
            self.sort_asc,
            &deltas,
        )
        .into();
        self.preserve_selection(&rows);
        self.cached_rows = Some(Arc::clone(&rows));
        self.cache_key = Some(new_key);
        self.cached_scan_generation = scan_generation;
        self.dirty = false;
        true
    }

    fn preserve_selection(&mut self, rows: &[ProcessCpuItem]) {
        self.selected_pid = self.selected_pid.filter(|&pid| rows.iter().any(|r| r.pid == pid));
        if self.selected_pid.is_none() {
            // No auto-select: only follow a name match, never rows[0].
            self.selected_pid = self
                .selected_name
                .as_ref()
                .and_then(|n| rows.iter().find(|r| r.name.eq_ignore_ascii_case(n)))
                .map(|r| r.pid);
            if self.selected_pid.is_none() {
                self.selected_name = None;
            }
        }
        self.selected_name = self
            .selected_pid
            .and_then(|pid| rows.iter().find(|r| r.pid == pid))
            .map(|r| r.name.to_string());
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

    fn compute_ram_deltas(&mut self, rows: &[RawProcessCpuRow]) -> HashMap<(u32, u64), i64> {
        let now = Instant::now();
        let mut deltas = HashMap::with_capacity(rows.len());
        for row in rows {
            let delta = self.ram_baseline.get(&(row.pid, row.start_time)).map_or(0, |prev| {
                let dt = now.saturating_duration_since(prev.1).as_secs_f32().max(0.25);
                let diff = row.ram_bytes as i64 - prev.0 as i64;
                (diff as f32 * (10.0 / dt)).round() as i64
            });
            deltas.insert((row.pid, row.start_time), delta);
        }
        self.ram_baseline =
            rows.iter().map(|row| ((row.pid, row.start_time), (row.ram_bytes, now))).collect();
        deltas
    }
}

#[cfg(test)]
#[path = "table_state_tests.rs"]
mod table_state_tests;
