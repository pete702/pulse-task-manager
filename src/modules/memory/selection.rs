//! Raw-process selection resolution: clone markers, action targets, and PID gating.
//!
//! Selection is resolved against the raw process table with direct filtering:
//! - `group_clones == true`  → all processes sharing the base group key.
//! - `group_clones == false` → only the single process matching `selected_pid`.

use crate::modules::memory::model::RawProcessRow;

#[cfg(test)]
use crate::modules::memory::model::MemorySnapshot;
use crate::modules::memory::process_target::ProcessTarget;
use crate::modules::memory::safe_handle::verify_pid_safety_gate;

/// Strips a trailing clone marker such as `[60x]` or `(60x)` from a display name.
pub fn base_group_key(name: &str) -> &str {
    let trimmed = name.trim();
    let mut base = trimmed;
    for (open, close) in [('[', ']'), ('(', ')')] {
        let inner = trimmed.strip_suffix(close).and_then(|s| s.rsplit_once(open));
        if let Some((name, _)) = inner.filter(|(_, count)| is_clone_count(count)) {
            base = name.trim_end();
        }
    }
    base
}

fn is_clone_count(text: &str) -> bool {
    let digits = text
        .strip_suffix('x')
        .or_else(|| text.strip_suffix('X'))
        .or_else(|| text.strip_suffix('×'));
    digits.is_some_and(|d| !d.is_empty() && d.bytes().all(|b| b.is_ascii_digit()))
}

/// Everything the action bar needs to know about the current row selection.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SelectionView {
    /// Clean base process name without the `[Nx]` clone marker.
    pub name: String,
    /// Raw processes belonging to the selected group (never filtered display rows).
    pub targets: Vec<ProcessTarget>,
    /// All live PIDs in the selected group.
    pub pids: Vec<u32>,
    /// True when the PID safety gate blocks any member of the group.
    pub blocked: bool,
    /// Executable path of the first raw process that has one.
    pub path: Option<String>,
}

impl SelectionView {
    /// True when trim/end actions may target this selection.
    #[allow(dead_code)]
    pub fn can_modify(&self) -> bool {
        !self.blocked && !self.pids.is_empty() && !self.targets.is_empty()
    }
}

/// Builds action targets from every raw process row (test helper only).
#[cfg(test)]
pub fn raw_targets(mem: &MemorySnapshot) -> Vec<ProcessTarget> {
    mem.processes.iter().map(ProcessTarget::from_raw).collect()
}

/// Resolves selection against raw process rows with direct filtering.
/// When group_clones is true, all processes matching selected_group are included.
/// When group_clones is false, only the single process matching selected_pid is included.
pub fn resolve_selection(
    raw_rows: &[RawProcessRow],
    selected_group: Option<&str>,
    selected_pid: Option<u32>,
    group_clones: bool,
) -> Option<SelectionView> {
    let key = base_group_key(selected_group?);
    if key.is_empty() {
        return None;
    }
    let targets: Vec<ProcessTarget> = if group_clones {
        raw_rows
            .iter()
            .filter(|row| base_group_key(&row.name) == key)
            .map(ProcessTarget::from_raw)
            .collect()
    } else {
        let pid = selected_pid?;
        raw_rows.iter().filter(|row| row.pid == pid).map(ProcessTarget::from_raw).collect()
    };
    if targets.is_empty() {
        return None;
    }
    let pids: Vec<u32> = targets.iter().map(|item| item.pid).collect();
    let blocked =
        pids.is_empty() || pids.iter().any(|pid| verify_pid_safety_gate(*pid, key).is_err());
    let path = targets.iter().find_map(|t| t.exe_path.clone().filter(|p| !p.is_empty()));
    Some(SelectionView { name: key.to_owned(), targets, pids, blocked, path })
}
