//! Mode-aware selection resolution and its O(1) cache for the memory table.
//!
//! Selection semantics never leak between modes:
//! - grouped (`group_clones == true`)  -> the whole name group;
//! - ungrouped (`group_clones == false`) -> exactly one PID.

use std::sync::Arc;

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
    /// Returns `Arc<SelectedGroupSummary>` for O(1) clone on cache hits.
    /// In single-PID mode, verifies process identity (creation_time + name) to prevent PID reuse.
    pub fn resolved_selection(
        &mut self,
        scan_id: u64,
        raw_rows: &[RawProcessRow],
    ) -> Option<Arc<SelectedGroupSummary>> {
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
            // In single-PID mode, verify process identity even on cache hit
            if !self.group_clones {
                if let Some(pid) = self.selected_pid {
                    if let Some(row) = raw_rows.iter().find(|r| r.pid == pid) {
                        let creation_ok = self
                            .selected_creation_time
                            .map_or(true, |t| t == row.creation_time.unwrap_or(0));
                        let name_ok = self
                            .selected_name
                            .as_ref()
                            .map_or(true, |n| base_group_key(n) == base_group_key(&row.name));
                        if !creation_ok || !name_ok {
                            // PID reuse detected - clear selection and cancel confirm
                            self.clear_selection_state();
                            return None;
                        }
                    } else {
                        // Process no longer exists
                        self.clear_selection_state();
                        return None;
                    }
                }
            }
            return self.cached_selection.as_ref().map(|(.., summary)| Arc::clone(summary));
        }
        let view = resolve_selection(raw_rows, Some(&group), self.selected_pid, self.group_clones);
        // In single-PID mode, verify process identity against previously stored identity
        // to detect PID reuse across scan boundaries.
        if !self.group_clones {
            if let Some(pid) = self.selected_pid {
                if let Some(row) = raw_rows.iter().find(|r| r.pid == pid) {
                    // Check if previously stored identity matches current process
                    let creation_ok = self
                        .selected_creation_time
                        .map_or(true, |t| t == row.creation_time.unwrap_or(0));
                    let name_ok = self
                        .selected_name
                        .as_ref()
                        .map_or(true, |n| base_group_key(n) == base_group_key(&row.name));
                    if !creation_ok || !name_ok {
                        // PID reuse detected - clear selection and cancel confirm
                        self.clear_selection_state();
                        return None;
                    }
                    // Identity matches - update stored identity for next scan
                    self.selected_creation_time = row.creation_time;
                    self.selected_name = Some(row.name.clone());
                } else {
                    // Process no longer exists
                    self.clear_selection_state();
                    return None;
                }
            }
            // If resolve_selection returns None (e.g., process not found), clear selection
            if view.is_none() {
                self.clear_selection_state();
                return None;
            }
        }
        let view = view?;
        let summary = Arc::new(SelectedGroupSummary {
            name: view.name,
            targets: view.targets,
            pids: view.pids,
            blocked: view.blocked,
            path: view.path,
        });
        self.cached_selection =
            Some((scan_id, group, self.selected_pid, self.group_clones, Arc::clone(&summary)));
        Some(summary)
    }

    /// True when this row belongs to the current selection.
    pub fn is_row_selected(&self, item: &ProcessMemoryItem) -> bool {
        self.selected_group.as_deref() == Some(base_group_key(&item.name))
            && (self.group_clones || self.selected_pid == Some(item.pid))
    }

    /// Clears selection state so single-PID and group semantics never leak between modes.
    pub fn clear_selection(&mut self) {
        self.clear_selection_state();
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
    /// In single-PID mode, captures creation_time and name for identity verification.
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
        // In single-PID mode, capture process identity for PID reuse detection
        if !self.group_clones {
            // Find the raw row to get creation_time
            // Note: we need access to raw_rows, but we only have ProcessMemoryItem here.
            // The creation_time will be captured from the raw row when resolved_selection runs.
            // For now, we store the normalized name.
            self.selected_name = Some(key.to_owned());
            // selected_creation_time will be set when the selection is resolved against raw rows
        } else {
            self.selected_name = None;
            self.selected_creation_time = None;
        }
        self.cached_selection = None;
    }
}
