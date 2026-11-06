use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use crate::modules::cpu::model::{ProcessCpuItem, RawProcessCpuRow};
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CpuSortCol {
    #[default]
    CpuPct,
    Name,
    Pid,
    RamBytes,
    CloneCount,
    Delta,
}
#[derive(Debug, Clone, PartialEq, Eq)]
struct CpuTableCacheKey(u64, String, CpuSortCol, bool, bool);
#[derive(Debug, Clone)]
pub struct CpuTableState {
    pub comfort_density: bool,
    pub search_query: String,
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
        );
        if !first_build && !needs_data && self.cache_key.as_ref() == Some(&new_key) {
            self.cached_scan_generation = scan_generation;
            self.dirty = false;
            return false;
        }
        let deltas = self.compute_ram_deltas(raw_rows);
        let rows: Arc<[ProcessCpuItem]> = Self::filter_group_and_sort(
            raw_rows,
            &self.search_query,
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
            self.selected_pid = self
                .selected_name
                .as_ref()
                .and_then(|n| rows.iter().find(|r| r.name.eq_ignore_ascii_case(n)))
                .map(|r| r.pid)
                .or_else(|| (!rows.is_empty()).then(|| rows[0].pid));
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
            Self::sort_slice(Arc::make_mut(rows), self.sort_col, self.sort_asc);
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

    #[inline]
    fn contains_ci(haystack: &str, needle: &str) -> bool {
        needle.is_empty()
            || haystack
                .as_bytes()
                .windows(needle.len())
                .any(|w| w.eq_ignore_ascii_case(needle.as_bytes()))
    }

    fn filter_group_and_sort(
        processes: &[RawProcessCpuRow],
        search: &str,
        group_clones: bool,
        sort_col: CpuSortCol,
        sort_asc: bool,
        deltas: &HashMap<(u32, u64), i64>,
    ) -> Vec<ProcessCpuItem> {
        let search_trimmed = search.trim();
        let mut items: Vec<ProcessCpuItem> = processes
            .iter()
            .filter(|row| {
                search_trimmed.is_empty()
                    || Self::contains_ci(&row.name, search_trimmed)
                    || row.pid.to_string().contains(search_trimmed)
            })
            .map(|row| {
                let delta = deltas.get(&(row.pid, row.start_time)).copied().unwrap_or(0);
                Self::item_from_raw_cpu(row, delta)
            })
            .collect();
        if group_clones {
            let mut grouped: HashMap<Arc<str>, ProcessCpuItem> = HashMap::new();
            for item in items {
                let key: Arc<str> = Arc::clone(&item.name).to_ascii_lowercase().into();
                grouped
                    .entry(key)
                    .and_modify(|ex| {
                        ex.cpu_pct += item.cpu_pct;
                        ex.ram_bytes = ex.ram_bytes.saturating_add(item.ram_bytes);
                        ex.ram_delta_10s = ex.ram_delta_10s.saturating_add(item.ram_delta_10s);
                        ex.clone_count += 1;
                        ex.update_formatted();
                    })
                    .or_insert(item);
            }
            items = grouped.into_values().collect();
        }
        Self::sort_slice(&mut items, sort_col, sort_asc);
        items
    }

    fn sort_slice(items: &mut [ProcessCpuItem], col: CpuSortCol, asc: bool) {
        items.sort_by(|a, b| {
            let ord = match col {
                CpuSortCol::Name => a.name.cmp(&b.name),
                CpuSortCol::Pid => a.pid.cmp(&b.pid),
                CpuSortCol::RamBytes => a.ram_bytes.cmp(&b.ram_bytes),
                CpuSortCol::CloneCount => a.clone_count.cmp(&b.clone_count),
                CpuSortCol::Delta => a.ram_delta_10s.cmp(&b.ram_delta_10s),
                CpuSortCol::CpuPct => {
                    a.cpu_pct.partial_cmp(&b.cpu_pct).unwrap_or(std::cmp::Ordering::Equal)
                }
            };
            if asc {
                ord
            } else {
                ord.reverse()
            }
        });
    }

    fn item_from_raw_cpu(row: &RawProcessCpuRow, delta: i64) -> ProcessCpuItem {
        let mut item = ProcessCpuItem::new(
            row.pid,
            Arc::clone(&row.name),
            row.cpu_usage.clamp(0.0, 100.0),
            row.ram_bytes,
            row.creation_time,
        );
        item.start_time = row.start_time;
        item.exe_path = row.exe_path.clone().unwrap_or_default();
        item.ram_delta_10s = delta;
        item.update_formatted();
        item
    }
}

#[cfg(test)]
#[path = "table_state_tests.rs"]
mod table_state_tests;
