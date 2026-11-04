//! Mode-aware selection resolution and its O(1) cache for the memory table.
//!
//! Selection semantics never leak between modes:
//! - grouped (`group_clones == true`)  -> the whole name group;
//! - ungrouped (`group_clones == false`) -> exactly one PID.

use crate::modules::memory::model::RawProcessRow;
use crate::modules::memory::process_model::ProcessMemoryItem;
use crate::modules::memory::process_target::ProcessTarget;
use crate::modules::memory::selection::{base_group_key, resolve_selection};

use super::table_state::MemoryTableState;

/// Cached, O(1) summary of the selected group for the action bar.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SelectedGroupSummary {
    pub name: String,
    pub targets: Vec<ProcessTarget>,
    pub pids: Vec<u32>,
    pub blocked: bool,
    pub path: Option<String>,
}

impl MemoryTableState {
    /// Resolves the current selection against raw rows, cached per scan id and mode.
    pub fn resolved_selection(
        &mut self,
        scan_id: u64,
        raw_rows: &[RawProcessRow],
    ) -> Option<SelectedGroupSummary> {
        let group = self.selected_group.clone()?;
        let cache_hit = self.cached_selection.as_ref().is_some_and(
            |(cached_scan, cached_key, cached_pid, cached_grouped, _)| {
                *cached_scan == scan_id
                    && cached_key == &group
                    && *cached_pid == self.selected_pid
                    && *cached_grouped == self.group_clones
            },
        );
        if cache_hit {
            return self.cached_selection.as_ref().map(|(.., summary)| summary.clone());
        }
        let view = resolve_selection(raw_rows, Some(&group), self.selected_pid, self.group_clones)?;
        let summary = SelectedGroupSummary {
            name: view.name,
            targets: view.targets,
            pids: view.pids,
            blocked: view.blocked,
            path: view.path,
        };
        self.cached_selection =
            Some((scan_id, group, self.selected_pid, self.group_clones, summary.clone()));
        Some(summary)
    }

    /// True when this row belongs to the current selection.
    pub fn is_row_selected(&self, item: &ProcessMemoryItem) -> bool {
        self.selected_group.as_deref() == Some(base_group_key(&item.name))
            && (self.group_clones || self.selected_pid == Some(item.pid))
    }

    /// Clears selection state so single-PID and group semantics never leak between modes.
    pub fn clear_selection(&mut self) {
        self.selected_pid = None;
        self.selected_group = None;
        self.cached_selection = None;
        self.group_clones_toggled = false;
    }

    /// Toggles clone grouping; a real mode change clears the selection and the row cache.
    pub fn set_group_clones(&mut self, enabled: bool) {
        if self.group_clones != enabled {
            self.group_clones = enabled;
            self.clear_selection();
            self.invalidate_cache();
            self.group_clones_toggled = true;
        }
    }

    /// Consumes the pending "grouping mode changed" flag.
    pub fn take_group_clones_toggled(&mut self) -> bool {
        std::mem::replace(&mut self.group_clones_toggled, false)
    }

    /// Toggles row selection under the current grouping mode.
    pub fn toggle_selection(&mut self, item: &ProcessMemoryItem) {
        let key = base_group_key(&item.name);
        let same_row = self.selected_group.as_deref() == Some(key)
            && (self.group_clones || self.selected_pid == Some(item.pid));
        if same_row {
            self.clear_selection();
            return;
        }
        self.selected_group = Some(key.to_owned());
        self.selected_pid = if self.group_clones { None } else { Some(item.pid) };
        self.cached_selection = None;
    }
}
