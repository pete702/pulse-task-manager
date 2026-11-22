//! CPU table tree view: view modes, hierarchy ordering and indentation helpers.
use std::collections::{HashMap, HashSet};

use crate::modules::cpu::model::{ProcessCpuItem, RawProcessCpuRow};
use crate::modules::cpu::table_state::CpuTableState;

/// CPU process table view mode. `Clones` is the persisted default.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CpuViewMode {
    Flat,
    #[default]
    Clones,
    Tree,
}

const MAX_DEPTH: usize = 6;

/// Orders `(pid, parent_pid)` pairs hierarchically. Orphans become roots,
/// cycles are broken and indentation depth is capped at six levels.
pub fn build_tree_order(rows: &[(u32, Option<u32>)]) -> Vec<(u32, usize)> {
    let present: HashSet<u32> = rows.iter().map(|&(pid, _)| pid).collect();
    let mut children: HashMap<u32, Vec<u32>> = HashMap::new();
    let mut roots: Vec<u32> = Vec::new();
    for &(pid, parent) in rows {
        match parent {
            Some(p) if p != pid && present.contains(&p) => {
                children.entry(p).or_default().push(pid);
            }
            _ => roots.push(pid),
        }
    }
    let mut out = Vec::with_capacity(rows.len());
    let mut visited: HashSet<u32> = HashSet::new();
    let mut stack: Vec<(u32, usize)> = roots.into_iter().rev().map(|p| (p, 0usize)).collect();
    while let Some((pid, depth)) = stack.pop() {
        if !visited.insert(pid) {
            continue;
        }
        out.push((pid, depth));
        if depth < MAX_DEPTH {
            if let Some(kids) = children.get(&pid) {
                for &kid in kids.iter().rev() {
                    if !visited.contains(&kid) {
                        stack.push((kid, depth + 1));
                    }
                }
            }
        }
    }
    // Nodes left over (cycles or depth-truncated branches) become roots.
    for &(pid, _) in rows {
        if visited.insert(pid) {
            out.push((pid, 0));
        }
    }
    out
}

/// Indent prefix for a tree row; root rows are not indented.
pub fn tree_indent(depth: usize) -> String {
    if depth == 0 {
        String::new()
    } else {
        format!("{}|- ", "  ".repeat(depth.min(MAX_DEPTH)))
    }
}

/// Reorders display items into tree order and stamps their display depth.
pub(crate) fn apply_tree_order(
    items: &mut [ProcessCpuItem],
    raw: &[RawProcessCpuRow],
    mode: CpuViewMode,
) {
    if mode != CpuViewMode::Tree {
        return;
    }
    let present: HashSet<u32> = items.iter().map(|i| i.pid).collect();
    let pairs: Vec<(u32, Option<u32>)> =
        raw.iter().filter(|r| present.contains(&r.pid)).map(|r| (r.pid, r.parent_pid)).collect();
    let order = build_tree_order(&pairs);
    let rank: HashMap<u32, usize> =
        order.iter().enumerate().map(|(i, &(pid, _))| (pid, i)).collect();
    let depth: HashMap<u32, usize> = order.into_iter().collect();
    items.sort_by_key(|i| rank.get(&i.pid).copied().unwrap_or(usize::MAX));
    for item in items.iter_mut() {
        item.tree_depth = depth.get(&item.pid).copied().unwrap_or(0).min(MAX_DEPTH);
    }
}

impl CpuTableState {
    /// Switches between Flat, Clones and Tree display modes.
    pub fn set_view_mode(&mut self, mode: CpuViewMode) {
        if self.view_mode == mode {
            return;
        }
        self.view_mode = mode;
        self.group_clones = mode == CpuViewMode::Clones;
        self.mark_dirty();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parent_child_hierarchy_is_ordered_and_indented() {
        let rows = [(10, None), (30, Some(20)), (20, Some(10)), (11, None)];
        let order = build_tree_order(&rows);
        assert_eq!(order, vec![(10, 0), (20, 1), (30, 2), (11, 0)]);
        assert_eq!(tree_indent(0), "");
        assert_eq!(tree_indent(1), "  |- ");
        assert_eq!(tree_indent(3), "      |- ");
    }

    #[test]
    fn orphan_parents_are_promoted_to_roots() {
        let rows = [(5, Some(999)), (6, Some(5)), (7, Some(999))];
        let order = build_tree_order(&rows);
        assert_eq!(order, vec![(5, 0), (6, 1), (7, 0)]);
    }

    #[test]
    fn indentation_depth_is_capped_at_six() {
        let rows: Vec<(u32, Option<u32>)> = (0u32..10).map(|i| (i, i.checked_sub(1))).collect();
        let order = build_tree_order(&rows);
        assert_eq!(order.len(), 10);
        assert!(order.iter().all(|&(_, d)| d <= 6));
        assert_eq!(order[6].1, 6);
    }

    #[test]
    fn cyclic_parent_links_do_not_loop_forever() {
        let rows = [(1, Some(2)), (2, Some(3)), (3, Some(1)), (4, None)];
        let order = build_tree_order(&rows);
        assert_eq!(order.len(), 4);
        assert_eq!(order.iter().filter(|&&(_, d)| d == 0).count(), 4);
    }
}
